use alloc::{boxed::Box, vec};

use crate::{
    base::{JavaClassProto, JavaContext, JavaFieldProto, JavaMethodProto, JavaResult},
    method::MethodBody,
    proxy::JavaObjectProxy,
    JavaError,
};

// class org.kwis.msp.lcdui.Display
pub struct Display {}

impl Display {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init),
                JavaMethodProto::new("getDisplay", "(Ljava/lang/String;)Lorg/kwis/msp/lcdui/Display;", Self::get_display),
                JavaMethodProto::new("getDefaultDisplay", "()Lorg/kwis/msp/lcdui/Display;", Self::get_default_display),
                JavaMethodProto::new("getDockedCard", "()Lorg/kwis/msp/lcdui/Card;", Self::get_docked_card),
                JavaMethodProto::new("pushCard", "(Lorg/kwis/msp/lcdui/Card;)V", Self::push_card),
                JavaMethodProto::new(
                    "addJletEventListener",
                    "(Lorg/kwis/msp/lcdui/JletEventListener;)V",
                    Self::add_jlet_event_listener,
                ),
            ],
            fields: vec![JavaFieldProto::new("card", "Lorg/kwis/msp/lcdui/Card;")],
        }
    }

    async fn init(_: &mut dyn JavaContext, instance: JavaObjectProxy) -> JavaResult<()> {
        log::warn!("stub Display::<init>({:#x})", instance.ptr_instance);

        Ok(())
    }

    async fn get_display(context: &mut dyn JavaContext, a0: JavaObjectProxy) -> JavaResult<JavaObjectProxy> {
        log::warn!("stub Display::getDisplay({:#x}", a0.ptr_instance);

        let instance = context.instantiate("Lorg/kwis/msp/lcdui/Display;")?;
        context.call_method(&instance, "<init>", "()V", &[]).await?;

        Ok(instance)
    }

    async fn get_default_display(context: &mut dyn JavaContext) -> JavaResult<JavaObjectProxy> {
        log::warn!("stub Display::getDefaultDisplay");

        let instance = context.instantiate("Lorg/kwis/msp/lcdui/Display;")?;
        context.call_method(&instance, "<init>", "()V", &[]).await?;

        Ok(instance)
    }

    async fn get_docked_card(_: &mut dyn JavaContext) -> JavaResult<JavaObjectProxy> {
        log::warn!("stub Display::getDockedCard");

        Ok(JavaObjectProxy::new(0))
    }

    async fn push_card(context: &mut dyn JavaContext, instance: JavaObjectProxy, a1: JavaObjectProxy) -> JavaResult<()> {
        log::warn!("stub Display::pushCard({:#x}, {:#x})", instance.ptr_instance, a1.ptr_instance);

        let card = context.get_field(&instance, "card")?;
        if card == 0 {
            context.put_field(&instance, "card", a1.ptr_instance)?;

            context.spawn(Box::new(DisplayLoop { instance: a1 }))?;
        }

        Ok(())
    }

    async fn add_jlet_event_listener(_: &mut dyn JavaContext, a0: JavaObjectProxy) -> JavaResult<()> {
        log::warn!("stub Display::addJletEventListener({:#x})", a0.ptr_instance);

        Ok(())
    }
}

struct DisplayLoop {
    instance: JavaObjectProxy,
}

#[async_trait::async_trait(?Send)]
impl MethodBody<JavaError> for DisplayLoop {
    async fn call(&self, context: &mut dyn JavaContext, _: &[u32]) -> Result<u32, JavaError> {
        let graphics = context.instantiate("Lorg/kwis/msp/lcdui/Graphics;")?;
        context.call_method(&graphics, "<init>", "()V", &[]).await?;

        loop {
            context.sleep(16).await;

            context
                .call_method(&self.instance, "paint", "(Lorg/kwis/msp/lcdui/Graphics;)V", &[graphics.ptr_instance])
                .await?;
        }

        #[allow(unreachable_code)]
        Ok(0)
    }
}
