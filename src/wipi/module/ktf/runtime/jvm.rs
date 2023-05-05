use std::{fmt::Display, mem::size_of};

use crate::{
    core::arm::ArmCore,
    wipi::java_impl::{get_java_impl, JavaMethodBody, Jvm},
};

use super::Context;

#[repr(C)]
#[derive(Clone, Copy)]
struct JavaClass {
    ptr_next: u32,
    unk1: u32,
    ptr_descriptor: u32,
    unk2: u32,
    unk3: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct JavaClassDescriptor {
    ptr_name: u32,
    unk1: u32,
    parent_class: u32,
    ptr_methods: u32,
    ptr_interfaces: u32,
    ptr_properties: u32,
    method_count: u16,
    fields_size: u16,
    access_flag: u16,
    unk6: u16,
    unk7: u16,
    unk8: u16,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct JavaMethod {
    fn_body: u32,
    ptr_class: u32,
    unk1: u32,
    ptr_name: u32,
    unk2: u32,
    unk3: u32,
    unk4: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct JavaClassInstance {
    ptr_fields: u32,
    ptr_class: u32,
}

#[derive(Clone)]
pub struct JavaMethodFullname {
    pub tag: u8,
    pub name: String,
    pub signature: String,
}

impl JavaMethodFullname {
    pub fn from_ptr(core: &ArmCore, ptr: u32) -> anyhow::Result<Self> {
        let tag = core.read(ptr)?;

        let value = core.read_null_terminated_string(ptr + 1)?;
        let value = value.split('+').collect::<Vec<_>>();

        Ok(JavaMethodFullname {
            tag,
            name: value[1].into(),
            signature: value[0].into(),
        })
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.push(self.tag);
        bytes.extend_from_slice(self.signature.as_bytes());
        bytes.push(b'+');
        bytes.extend_from_slice(self.name.as_bytes());
        bytes.push(0);

        bytes
    }
}

impl Display for JavaMethodFullname {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)?;
        self.signature.fmt(f)?;
        write!(f, "@{}", self.tag)?;

        Ok(())
    }
}

impl PartialEq for JavaMethodFullname {
    fn eq(&self, other: &Self) -> bool {
        self.signature == other.signature && self.name == other.name
    }
}

pub struct KtfJvm<'a> {
    core: &'a mut ArmCore,
    context: &'a Context,
}

impl<'a> KtfJvm<'a> {
    pub fn new(core: &'a mut ArmCore, context: &'a Context) -> Self {
        Self { core, context }
    }

    pub fn get_method(&mut self, ptr_class: u32, fullname: JavaMethodFullname) -> anyhow::Result<u32> {
        let class = self.core.read::<JavaClass>(ptr_class)?;
        let class_descriptor = self.core.read::<JavaClassDescriptor>(class.ptr_descriptor)?;
        let class_name = self.core.read_null_terminated_string(class_descriptor.ptr_name)?;

        let mut cursor = class_descriptor.ptr_methods;
        loop {
            let ptr = self.core.read::<u32>(cursor)?;
            if ptr == 0 {
                log::error!("Can't find function {} from {}", fullname, class_name);

                return Ok(0);
            }

            let current_method = self.core.read::<JavaMethod>(ptr)?;
            let current_fullname = JavaMethodFullname::from_ptr(self.core, current_method.ptr_name)?;

            if current_fullname == fullname {
                return Ok(ptr);
            }

            cursor += 4;
        }
    }

    pub fn load_class(&mut self, ptr_target: u32, name: String) -> anyhow::Result<()> {
        let r#impl = get_java_impl(&name);
        if r#impl.is_none() {
            return Err(anyhow::anyhow!("No such class"));
        }
        let r#impl = r#impl.unwrap();

        let ptr_class = self.context.alloc(size_of::<JavaClass>() as u32)?;
        self.core.write(
            ptr_class,
            JavaClass {
                ptr_next: ptr_class + 4,
                unk1: 0,
                ptr_descriptor: 0,
                unk2: 0,
                unk3: 0,
            },
        )?;

        let method_count = r#impl.methods.len();
        let ptr_methods = self.context.alloc(((method_count + 1) * size_of::<u32>()) as u32)?;

        let mut cursor = ptr_methods;
        for method in r#impl.methods {
            let fullname = (JavaMethodFullname {
                tag: 0,
                name: method.name,
                signature: method.signature,
            })
            .as_bytes();

            let ptr_name = self.context.alloc(fullname.len() as u32)?;
            self.core.write_raw(ptr_name, &fullname)?;

            let ptr_method = self.context.alloc(size_of::<JavaMethod>() as u32)?;
            let fn_body = self.register_java_method(method.body)?;
            self.core.write(
                ptr_method,
                JavaMethod {
                    fn_body,
                    ptr_class,
                    unk1: 0,
                    ptr_name,
                    unk2: 0,
                    unk3: 0,
                    unk4: 0,
                },
            )?;

            self.core.write(cursor, ptr_method)?;
            cursor += 4;
        }

        let ptr_name = self.context.alloc((name.len() + 1) as u32)?;
        self.core.write_raw(ptr_name, name.as_bytes())?;

        let ptr_descriptor = self.context.alloc(size_of::<JavaClassDescriptor>() as u32)?;
        self.core.write(
            ptr_descriptor,
            JavaClassDescriptor {
                ptr_name,
                unk1: 0,
                parent_class: 0,
                ptr_methods,
                ptr_interfaces: 0,
                ptr_properties: 0,
                method_count: method_count as u16,
                fields_size: 0,
                access_flag: 0x21, // ACC_PUBLIC | ACC_SUPER
                unk6: 0,
                unk7: 0,
                unk8: 0,
            },
        )?;

        self.core.write(ptr_class + 8, ptr_descriptor)?;

        self.core.write(ptr_target, ptr_class)?; // we should cache ptr_class

        Ok(())
    }

    pub fn instantiate_from_ptr_class(&mut self, ptr_class: u32) -> anyhow::Result<u32> {
        let class = self.core.read::<JavaClass>(ptr_class)?;
        let class_descriptor = self.core.read::<JavaClassDescriptor>(class.ptr_descriptor)?;
        let class_name = self.core.read_null_terminated_string(class_descriptor.ptr_name)?;

        log::info!("Instantiate {}", class_name);

        let ptr_instance = self.context.alloc(size_of::<JavaClassInstance>() as u32)?;
        let ptr_fields = self.context.alloc(class_descriptor.fields_size as u32 + 4)?;

        self.core.write(ptr_instance, JavaClassInstance { ptr_fields, ptr_class })?;

        Ok(ptr_instance)
    }

    fn register_java_method(&mut self, body: JavaMethodBody) -> anyhow::Result<u32> {
        let closure = move |core: &mut ArmCore, context: &Context| {
            let mut jvm = KtfJvm::new(core, context);
            body(&mut jvm, vec![]);

            Ok::<u32, anyhow::Error>(0u32)
        };

        self.core.register_function(closure, self.context)
    }
}

impl Jvm for KtfJvm<'_> {
    fn instantiate(&mut self, _class_name: &str) -> anyhow::Result<u32> {
        todo!()
    }

    fn call_method(&mut self, ptr_instance: u32, name: &str, signature: &str, _args: &[u32]) -> anyhow::Result<u32> {
        let instance = self.core.read::<JavaClassInstance>(ptr_instance)?;
        let class = self.core.read::<JavaClass>(instance.ptr_class)?;
        let class_descriptor = self.core.read::<JavaClassDescriptor>(class.ptr_descriptor)?;
        let class_name = self.core.read_null_terminated_string(class_descriptor.ptr_name)?;

        log::info!("Call {}::{}({})", class_name, name, signature);

        let fullname = JavaMethodFullname {
            tag: 0,
            name: name.to_owned(),
            signature: signature.to_owned(),
        };

        let ptr_method = self.get_method(instance.ptr_class, fullname)?;

        let method = self.core.read::<JavaMethod>(ptr_method)?;

        self.core.run_function(method.fn_body, &[0, ptr_instance])
    }

    fn get_field(&mut self, _ptr_instance: u32, _field_offset: u32) -> anyhow::Result<u32> {
        todo!()
    }

    fn put_field(&mut self, _ptr_instance: u32, _field_offset: u32, _value: u32) {
        todo!()
    }
}
