use alloc::{boxed::Box, vec::Vec};
use core::{
    fmt::{self, Debug, Formatter},
    iter,
    mem::size_of,
};

use bytemuck::{Pod, Zeroable};

use jvm::{ClassDefinition, ClassInstance, Field, JavaType, JavaValue, JvmResult};

use wie_common::util::{read_generic, write_generic, ByteWrite};
use wie_core_arm::{Allocator, ArmCore};

use super::{class_definition::JavaClassDefinition, context_data::JavaContextData, field::JavaField, value::JavaValueExt, KtfJvmWord};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct RawJavaClassInstance {
    ptr_fields: u32,
    ptr_class: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct RawJavaClassInstanceFields {
    vtable_index: u32, // left shifted by 5
    fields: [u32; 1],
}

#[derive(Clone)]
pub struct JavaClassInstance {
    pub(crate) ptr_raw: u32,
    core: ArmCore,
}

impl JavaClassInstance {
    pub fn from_raw(ptr_raw: u32, core: &ArmCore) -> Self {
        Self { ptr_raw, core: core.clone() }
    }

    pub fn new(core: &mut ArmCore, class: &JavaClassDefinition) -> JvmResult<Self> {
        let field_size = class.field_size()?;

        let instance = Self::instantiate(core, class, field_size)?;

        tracing::trace!("Instantiated {} at {:#x}", class.name()?, instance.ptr_raw);

        Ok(instance)
    }

    pub fn destroy(mut self) -> JvmResult<()> {
        let raw = self.read_raw()?;

        Allocator::free(&mut self.core, raw.ptr_fields)?;
        Allocator::free(&mut self.core, self.ptr_raw)?;

        Ok(())
    }

    pub fn class(&self) -> JvmResult<JavaClassDefinition> {
        let raw = self.read_raw()?;

        Ok(JavaClassDefinition::from_raw(raw.ptr_class, &self.core))
    }

    pub fn read_field(&self, field: &JavaField) -> JvmResult<KtfJvmWord> {
        let offset = field.offset()?;

        let address = self.field_address(offset)?;

        let value: KtfJvmWord = read_generic(&self.core, address)?;

        Ok(value)
    }

    pub fn write_field(&mut self, field: &JavaField, value: KtfJvmWord) -> JvmResult<()> {
        let offset = field.offset()?;

        let address = self.field_address(offset)?;

        write_generic(&mut self.core, address, value)
    }

    pub(super) fn field_address(&self, offset: u32) -> JvmResult<u32> {
        let raw = self.read_raw()?;

        Ok(raw.ptr_fields + offset + 4)
    }

    pub(super) fn instantiate(core: &mut ArmCore, class: &JavaClassDefinition, field_size: usize) -> JvmResult<Self> {
        let ptr_raw = Allocator::alloc(core, size_of::<RawJavaClassInstance>() as _)?;
        let ptr_fields = Allocator::alloc(core, (field_size + 4) as _)?;

        let zero = iter::repeat(0).take((field_size + 4) as _).collect::<Vec<_>>();
        core.write_bytes(ptr_fields, &zero)?;

        let vtable_index = JavaContextData::get_vtable_index(core, class)?;

        write_generic(
            core,
            ptr_raw,
            RawJavaClassInstance {
                ptr_fields,
                ptr_class: class.ptr_raw,
            },
        )?;
        write_generic(core, ptr_fields, (vtable_index * 4) << 5)?;

        tracing::trace!("Instantiate {}, vtable_index {:#x}", class.name()?, vtable_index);

        Ok(Self::from_raw(ptr_raw, core))
    }

    fn read_raw(&self) -> JvmResult<RawJavaClassInstance> {
        let instance: RawJavaClassInstance = read_generic(&self.core, self.ptr_raw as _)?;

        Ok(instance)
    }
}

impl ClassInstance for JavaClassInstance {
    fn destroy(self: Box<Self>) {
        (*self).destroy().unwrap()
    }

    fn class_definition(&self) -> Box<dyn ClassDefinition> {
        Box::new(self.class().unwrap())
    }

    fn equals(&self, other: &dyn ClassInstance) -> JvmResult<bool> {
        let other_instance = other.as_any().downcast_ref::<JavaClassInstance>().unwrap();

        Ok(self.ptr_raw == other_instance.ptr_raw)
    }

    fn get_field(&self, field: &dyn Field) -> JvmResult<JavaValue> {
        let field = field.as_any().downcast_ref::<JavaField>().unwrap();

        let result = self.read_field(field)?;

        let r#type = JavaType::parse(&field.descriptor());
        Ok(JavaValue::from_raw(result, &r#type, &self.core))
    }

    fn put_field(&mut self, field: &dyn Field, value: JavaValue) -> JvmResult<()> {
        let field = field.as_any().downcast_ref::<JavaField>().unwrap();

        self.write_field(field, value.as_raw())
    }
}

impl Debug for JavaClassInstance {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:#x}", self.ptr_raw)
    }
}
