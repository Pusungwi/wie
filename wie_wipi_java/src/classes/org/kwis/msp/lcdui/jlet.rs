use alloc::{boxed::Box, vec};

use java_class_proto::{JavaError, JavaFieldProto, JavaMethodProto, JavaResult, MethodBody};
use java_constants::{FieldAccessFlags, MethodAccessFlags};
use java_runtime::classes::java::lang::String;
use jvm::{runtime::JavaLangString, ClassInstanceRef, JavaValue, Jvm};

use crate::{
    classes::org::kwis::msp::lcdui::EventQueue,
    context::{WIPIJavaClassProto, WIPIJavaContext},
};

// class org.kwis.msp.lcdui.Jlet
pub struct Jlet {}

impl Jlet {
    pub fn as_proto() -> WIPIJavaClassProto {
        WIPIJavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, Default::default()),
                JavaMethodProto::new(
                    "getActiveJlet",
                    "()Lorg/kwis/msp/lcdui/Jlet;",
                    Self::get_active_jlet,
                    MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "getEventQueue",
                    "()Lorg/kwis/msp/lcdui/EventQueue;",
                    Self::get_event_queue,
                    Default::default(),
                ),
                JavaMethodProto::new(
                    "getAppProperty",
                    "(Ljava/lang/String;)Ljava/lang/String;",
                    Self::get_app_property,
                    Default::default(),
                ),
            ],
            fields: vec![
                JavaFieldProto::new("dis", "Lorg/kwis/msp/lcdui/Display;", Default::default()),
                JavaFieldProto::new("eq", "Lorg/kwis/msp/lcdui/EventQueue;", Default::default()),
                JavaFieldProto::new("qtletActive", "Lorg/kwis/msp/lcdui/Jlet;", FieldAccessFlags::STATIC),
            ],
        }
    }

    async fn init(jvm: &Jvm, _: &mut WIPIJavaContext, mut this: ClassInstanceRef<Self>) -> JavaResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Jlet::<init>({:?})", &this);

        let display = jvm
            .new_class(
                "org/kwis/msp/lcdui/Display",
                "(Lorg/kwis/msp/lcdui/Jlet;Lorg/kwis/msp/lcdui/DisplayProxy;)V",
                (this.clone(), None),
            )
            .await?;

        jvm.put_field(&mut this, "dis", "Lorg/kwis/msp/lcdui/Display;", display)?;

        let event_queue = jvm
            .new_class("org/kwis/msp/lcdui/EventQueue", "(Lorg/kwis/msp/lcdui/Jlet;)V", (this.clone(),))
            .await?;

        jvm.put_field(&mut this, "eq", "Lorg/kwis/msp/lcdui/EventQueue;", event_queue)?;

        jvm.put_static_field("org/kwis/msp/lcdui/Jlet", "qtletActive", "Lorg/kwis/msp/lcdui/Jlet;", this.clone())
            .await?;

        Ok(())
    }

    async fn get_active_jlet(jvm: &Jvm, _: &mut WIPIJavaContext) -> JavaResult<ClassInstanceRef<Jlet>> {
        tracing::debug!("org.kwis.msp.lcdui.Jlet::getActiveJlet");

        let jlet = jvm
            .get_static_field("org/kwis/msp/lcdui/Jlet", "qtletActive", "Lorg/kwis/msp/lcdui/Jlet;")
            .await?;

        Ok(jlet)
    }

    async fn get_event_queue(jvm: &Jvm, _: &mut WIPIJavaContext, this: ClassInstanceRef<Self>) -> JavaResult<ClassInstanceRef<EventQueue>> {
        tracing::debug!("org.kwis.msp.lcdui.Jlet::getEventQueue({:?})", &this);

        let eq = jvm.get_field(&this, "eq", "Lorg/kwis/msp/lcdui/EventQueue;")?;

        Ok(eq)
    }

    async fn get_app_property(
        jvm: &Jvm,
        _: &mut WIPIJavaContext,
        this: ClassInstanceRef<Self>,
        key: ClassInstanceRef<String>,
    ) -> JavaResult<ClassInstanceRef<String>> {
        tracing::warn!("stub org.kwis.msp.lcdui.Jlet::getAppProperty({:?}, {:?})", &this, &key);

        Ok(JavaLangString::from_rust_string(jvm, "").await?.into())
    }

    pub async fn start(jvm: &Jvm, context: &mut WIPIJavaContext, main_class_name: &str) -> JavaResult<()> {
        let main_class_name = main_class_name.replace('.', "/");

        let main_class = jvm.new_class(&main_class_name, "()V", []).await?;

        tracing::debug!("Main class instance: {:?}", &main_class);

        let arg = jvm.instantiate_array("Ljava/lang/String;", 0).await?;
        jvm.invoke_virtual(&main_class, "startApp", "([Ljava/lang/String;)V", [arg.into()])
            .await?;

        struct StartProxy {}

        #[async_trait::async_trait(?Send)]
        impl MethodBody<JavaError, WIPIJavaContext> for StartProxy {
            #[tracing::instrument(name = "main", skip_all)]
            async fn call(&self, jvm: &Jvm, _: &mut WIPIJavaContext, _: Box<[JavaValue]>) -> Result<JavaValue, JavaError> {
                jvm.invoke_static("org/kwis/msp/lcdui/Main", "main", "([Ljava/lang/String;)V", [None.into()])
                    .await?;

                Ok(JavaValue::Void)
            }
        }

        context.spawn(Box::new(StartProxy {}))?;

        Ok(())
    }
}
