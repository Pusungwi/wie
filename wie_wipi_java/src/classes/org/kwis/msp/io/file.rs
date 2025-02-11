use alloc::vec;
use core::cmp::min;

use bytemuck::cast_slice;

use java_class_proto::{JavaFieldProto, JavaMethodProto, JavaResult};
use java_runtime::classes::java::lang::String;
use jvm::{runtime::JavaLangString, Array, ClassInstanceRef, Jvm};

use crate::context::{WIPIJavaClassProto, WIPIJavaContext};

// class org.kwis.msp.io.File
pub struct File {}

impl File {
    pub fn as_proto() -> WIPIJavaClassProto {
        WIPIJavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/lang/String;I)V", Self::init, Default::default()),
                JavaMethodProto::new("<init>", "(Ljava/lang/String;II)V", Self::init_with_flag, Default::default()),
                JavaMethodProto::new("write", "([BII)I", Self::write, Default::default()),
                JavaMethodProto::new("read", "([B)I", Self::read, Default::default()),
                JavaMethodProto::new("close", "()V", Self::close, Default::default()),
                JavaMethodProto::new("sizeOf", "()I", Self::size_of, Default::default()),
            ],
            fields: vec![
                JavaFieldProto::new("data", "[B", Default::default()),
                JavaFieldProto::new("pos", "I", Default::default()),
            ],
        }
    }

    async fn init(jvm: &Jvm, _: &mut WIPIJavaContext, this: ClassInstanceRef<Self>, filename: ClassInstanceRef<String>, mode: i32) -> JavaResult<()> {
        tracing::debug!("org.kwis.msp.io.File::<init>({:?}, {:?}, {:?})", &this, &filename, mode);

        jvm.invoke_special(&this, "org/kwis/msp/io/File", "<init>", "(Ljava/lang/String;II)V", (filename, mode, 0))
            .await?;

        Ok(())
    }

    async fn init_with_flag(
        jvm: &Jvm,
        context: &mut WIPIJavaContext,
        mut this: ClassInstanceRef<Self>,
        filename: ClassInstanceRef<String>,
        mode: i32,
        flag: i32,
    ) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.io.File::<init>({:?}, {:?}, {:?}, {:?})", &this, &filename, mode, flag);

        let filename = JavaLangString::to_rust_string(jvm, filename.into())?;
        tracing::debug!("Loading {}", filename);

        // TODO we don't have filesystem now, emulating file loading with resource for now..
        let data = {
            let resource = context.system().resource();
            let data = resource.data(resource.id(&filename).unwrap());

            cast_slice(data).to_vec()
        };

        let mut data_array = jvm.instantiate_array("B", data.len() as _).await?;
        jvm.store_byte_array(&mut data_array, 0, data)?;

        jvm.put_field(&mut this, "data", "[B", data_array)?;
        jvm.put_field(&mut this, "pos", "I", 0)?;

        Ok(())
    }

    async fn write(
        _jvm: &Jvm,
        _: &mut WIPIJavaContext,
        this: ClassInstanceRef<Self>,
        buf: ClassInstanceRef<ClassInstanceRef<Array<i8>>>,
        offset: i32,
        len: i32,
    ) -> JavaResult<i32> {
        tracing::warn!("stub org.kwis.msp.io.File::write({:?}, {:?}, {:?}, {:?})", &this, &buf, offset, len);

        Ok(0)
    }

    async fn read(jvm: &Jvm, _: &mut WIPIJavaContext, mut this: ClassInstanceRef<Self>, mut buf: ClassInstanceRef<Array<i8>>) -> JavaResult<i32> {
        tracing::debug!("org.kwis.msp.io.File::read({:?}, {:?})", &this, &buf);

        let data_array = jvm.get_field(&this, "data", "[B")?;
        let pos: i32 = jvm.get_field(&this, "pos", "I")?;

        let data_len = jvm.array_length(&data_array)?;
        let buf_len = jvm.array_length(&buf)?;

        let length_to_read = min(data_len - pos as usize, buf_len);

        let data = jvm.load_byte_array(&data_array, pos as _, length_to_read)?;
        jvm.store_byte_array(&mut buf, 0, data)?;

        jvm.put_field(&mut this, "pos", "I", pos + length_to_read as i32)?;

        Ok(length_to_read as _)
    }

    async fn close(_jvm: &Jvm, _: &mut WIPIJavaContext, this: ClassInstanceRef<Self>) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.io.File::close({:?})", &this);

        Ok(())
    }

    async fn size_of(jvm: &Jvm, _: &mut WIPIJavaContext, this: ClassInstanceRef<Self>) -> JavaResult<i32> {
        tracing::debug!("org.kwis.msp.io.File::sizeOf({:?})", &this);

        let data_array = jvm.get_field(&this, "data", "[B")?;
        let data_len = jvm.array_length(&data_array)?;

        Ok(data_len as _)
    }
}
