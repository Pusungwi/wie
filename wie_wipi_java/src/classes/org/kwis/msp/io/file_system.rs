use alloc::{format, vec};

use java_class_proto::{JavaMethodProto, JavaResult};
use java_constants::MethodAccessFlags;
use java_runtime::classes::java::lang::String;
use jvm::{runtime::JavaLangString, ClassInstanceRef, Jvm};

use crate::context::{WIPIJavaClassProto, WIPIJavaContext};

// class org.kwis.msp.io.FileSystem
pub struct FileSystem {}

impl FileSystem {
    pub fn as_proto() -> WIPIJavaClassProto {
        WIPIJavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("isFile", "(Ljava/lang/String;)Z", Self::is_file, MethodAccessFlags::STATIC),
                JavaMethodProto::new("isDirectory", "(Ljava/lang/String;I)Z", Self::is_directory, MethodAccessFlags::STATIC),
                JavaMethodProto::new("exists", "(Ljava/lang/String;)Z", Self::exists, MethodAccessFlags::STATIC),
                JavaMethodProto::new("available", "()I", Self::available, MethodAccessFlags::STATIC),
            ],
            fields: vec![],
        }
    }

    async fn is_file(_jvm: &Jvm, _: &mut WIPIJavaContext, name: ClassInstanceRef<String>) -> JavaResult<bool> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::is_file({:?})", &name);

        Ok(false)
    }

    async fn is_directory(_jvm: &Jvm, _: &mut WIPIJavaContext, name: ClassInstanceRef<String>, flag: i32) -> JavaResult<bool> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::isDirectory({:?}, {:?})", &name, flag);

        Ok(true)
    }

    async fn exists(jvm: &Jvm, context: &mut WIPIJavaContext, name: ClassInstanceRef<String>) -> JavaResult<bool> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::exists({:?})", &name);

        let filename = JavaLangString::to_rust_string(jvm, name.into())?;

        // emulating filesystem by resource..
        let filename_on_resource = format!("P{}", filename);

        let id = context.system().resource().id(&filename_on_resource);

        Ok(id.is_some())
    }

    async fn available(_: &Jvm, _: &mut WIPIJavaContext) -> JavaResult<i32> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::available()");

        Ok(0x1000000) // TODO temp
    }
}
