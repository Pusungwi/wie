use alloc::vec;

use java_class_proto::{JavaMethodProto, JavaResult};
use java_constants::MethodAccessFlags;
use java_runtime::classes::java::lang::String;
use jvm::{runtime::JavaLangString, ClassInstanceRef, Jvm};

use crate::context::{WIPIJavaClassProto, WIPIJavaContext};

// class org.kwis.msp.handset.HandsetProperty
pub struct HandsetProperty {}

impl HandsetProperty {
    pub fn as_proto() -> WIPIJavaClassProto {
        WIPIJavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new(
                "getSystemProperty",
                "(Ljava/lang/String;)Ljava/lang/String;",
                Self::get_system_property,
                MethodAccessFlags::STATIC,
            )],
            fields: vec![],
        }
    }

    async fn get_system_property(jvm: &Jvm, _: &mut WIPIJavaContext, name: ClassInstanceRef<String>) -> JavaResult<ClassInstanceRef<String>> {
        let name = JavaLangString::to_rust_string(jvm, name.into())?;
        tracing::warn!("stub org.kwis.msp.handset.HandsetProperty::getSystemProperty({})", name);

        let result = JavaLangString::from_rust_string(jvm, "").await?;
        Ok(result.into())
    }
}
