use alloc::{boxed::Box, vec, vec::Vec};
use core::iter;

use java_class_proto::{JavaError, JavaFieldProto, JavaMethodProto, JavaResult, MethodBody};
use java_constants::MethodAccessFlags;
use java_runtime::classes::java::lang::{Object, Runnable, String};
use jvm::{ClassInstanceRef, JavaValue, Jvm};

use crate::{
    classes::org::kwis::msp::lcdui::{Card, Jlet, JletEventListener},
    context::{WIPIJavaClassProto, WIPIJavaContext},
};

// class org.kwis.msp.lcdui.Display
pub struct Display {}

impl Display {
    pub fn as_proto() -> WIPIJavaClassProto {
        WIPIJavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new(
                    "<init>",
                    "(Lorg/kwis/msp/lcdui/Jlet;Lorg/kwis/msp/lcdui/DisplayProxy;)V",
                    Self::init,
                    Default::default(),
                ),
                JavaMethodProto::new(
                    "getDisplay",
                    "(Ljava/lang/String;)Lorg/kwis/msp/lcdui/Display;",
                    Self::get_display,
                    MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "getDefaultDisplay",
                    "()Lorg/kwis/msp/lcdui/Display;",
                    Self::get_default_display,
                    MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new("isDoubleBuffered", "()Z", Self::is_double_buffered, Default::default()),
                JavaMethodProto::new("getDockedCard", "()Lorg/kwis/msp/lcdui/Card;", Self::get_docked_card, Default::default()),
                JavaMethodProto::new("pushCard", "(Lorg/kwis/msp/lcdui/Card;)V", Self::push_card, Default::default()),
                JavaMethodProto::new("removeAllCards", "()V", Self::remove_all_cards, Default::default()),
                JavaMethodProto::new(
                    "addJletEventListener",
                    "(Lorg/kwis/msp/lcdui/JletEventListener;)V",
                    Self::add_jlet_event_listener,
                    Default::default(),
                ),
                JavaMethodProto::new("getWidth", "()I", Self::get_width, Default::default()),
                JavaMethodProto::new("getHeight", "()I", Self::get_height, Default::default()),
                JavaMethodProto::new("callSerially", "(Ljava/lang/Runnable;)V", Self::call_serially, Default::default()),
                JavaMethodProto::new("getGameAction", "(I)I", Self::get_game_action, MethodAccessFlags::NATIVE),
            ],
            fields: vec![
                JavaFieldProto::new("cards", "[Lorg/kwis/msp/lcdui/Card;", Default::default()),
                JavaFieldProto::new("szCard", "I", Default::default()),
                JavaFieldProto::new("m_w", "I", Default::default()),
                JavaFieldProto::new("m_h", "I", Default::default()),
            ],
        }
    }

    async fn init(
        jvm: &Jvm,
        context: &mut WIPIJavaContext,
        mut this: ClassInstanceRef<Self>,
        jlet: ClassInstanceRef<Jlet>,
        display_proxy: ClassInstanceRef<Object>,
    ) -> JavaResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Display::<init>({:?}, {:?}, {:?})", &this, &jlet, &display_proxy);

        let cards = jvm.instantiate_array("Lorg/kwis/msp/lcdui/Card;", 10).await?;
        jvm.put_field(&mut this, "cards", "[Lorg/kwis/msp/lcdui/Card;", cards)?;
        jvm.put_field(&mut this, "szCard", "I", 0)?;

        let (width, height) = {
            let mut platform = context.system().platform();
            let screen = platform.screen();
            (screen.width(), screen.height())
        };

        jvm.put_field(&mut this, "m_w", "I", width as i32)?;
        jvm.put_field(&mut this, "m_h", "I", height as i32)?;

        Ok(())
    }

    async fn get_display(jvm: &Jvm, _: &mut WIPIJavaContext, str: ClassInstanceRef<String>) -> JavaResult<ClassInstanceRef<Self>> {
        tracing::warn!("stub org.kwis.msp.lcdui.Display::getDisplay({:?})", &str);

        let jlet = jvm
            .invoke_static("org/kwis/msp/lcdui/Jlet", "getActiveJlet", "()Lorg/kwis/msp/lcdui/Jlet;", [])
            .await?;

        let display = jvm.get_field(&jlet, "dis", "Lorg/kwis/msp/lcdui/Display;")?;

        Ok(display)
    }

    async fn get_default_display(jvm: &Jvm, _: &mut WIPIJavaContext) -> JavaResult<ClassInstanceRef<Display>> {
        tracing::debug!("org.kwis.msp.lcdui.Display::getDefaultDisplay");

        let result = jvm
            .invoke_static(
                "org/kwis/msp/lcdui/Display",
                "getDisplay",
                "(Ljava/lang/String;)Lorg/kwis/msp/lcdui/Display;",
                [None.into()],
            )
            .await?;

        Ok(result)
    }

    async fn get_docked_card(_: &Jvm, _: &mut WIPIJavaContext) -> JavaResult<ClassInstanceRef<Card>> {
        tracing::warn!("stub org.kwis.msp.lcdui.Display::getDockedCard");

        Ok(None.into())
    }

    async fn is_double_buffered(_: &Jvm, _: &mut WIPIJavaContext, this: ClassInstanceRef<Self>) -> JavaResult<bool> {
        tracing::warn!("stub org.kwis.msp.lcdui.Display::isDoubleBuffered({:?})", &this);

        Ok(true)
    }

    async fn push_card(jvm: &Jvm, _: &mut WIPIJavaContext, mut this: ClassInstanceRef<Self>, c: ClassInstanceRef<Card>) -> JavaResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Display::pushCard({:?}, {:?})", &this, &c);

        let mut cards = jvm.get_field(&this, "cards", "[Lorg/kwis/msp/lcdui/Card;")?;
        let card_size: i32 = jvm.get_field(&this, "szCard", "I")?;

        let cards_data = jvm.load_array(&cards, 0, card_size as usize)?;
        let cards_data = cards_data.into_iter().chain(iter::once(c)).collect::<Vec<_>>();

        jvm.store_array(&mut cards, 0, cards_data)?;
        jvm.put_field(&mut this, "szCard", "I", card_size + 1)?;

        Ok(())
    }

    async fn remove_all_cards(jvm: &Jvm, _: &mut WIPIJavaContext, mut this: ClassInstanceRef<Self>) -> JavaResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Display::removeAllCards");

        jvm.put_field(&mut this, "szCard", "I", 0)?;

        Ok(())
    }

    async fn add_jlet_event_listener(
        _: &Jvm,
        _: &mut WIPIJavaContext,
        this: ClassInstanceRef<Display>,
        qel: ClassInstanceRef<JletEventListener>,
    ) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.Display::addJletEventListener({:?}, {:?})", &this, &qel);

        Ok(())
    }

    async fn get_width(jvm: &Jvm, _: &mut WIPIJavaContext, this: ClassInstanceRef<Self>) -> JavaResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Display::getWidth({:?})", &this);

        let width: i32 = jvm.get_field(&this, "m_w", "I")?;

        Ok(width)
    }

    async fn get_height(jvm: &Jvm, _: &mut WIPIJavaContext, this: ClassInstanceRef<Self>) -> JavaResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Display::getHeight({:?})", &this);

        let height: i32 = jvm.get_field(&this, "m_h", "I")?;

        Ok(height)
    }

    async fn call_serially(_: &Jvm, context: &mut WIPIJavaContext, this: ClassInstanceRef<Self>, r: ClassInstanceRef<Runnable>) -> JavaResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Display::callSerially({:?}, {:?})", &this, &r);

        // TODO this method have to queue runnable in event queue, but for now we'll spawn new task

        struct SpawnProxy {
            runnable: ClassInstanceRef<Runnable>,
        }

        #[async_trait::async_trait(?Send)]
        impl MethodBody<JavaError, WIPIJavaContext> for SpawnProxy {
            async fn call(&self, jvm: &Jvm, context: &mut WIPIJavaContext, _: Box<[JavaValue]>) -> Result<JavaValue, JavaError> {
                let until = context.system().platform().now() + 16; // TODO
                context.system().sleep(until).await;

                jvm.invoke_virtual(&self.runnable, "run", "()V", ()).await?;

                Ok(JavaValue::Void)
            }
        }

        context.spawn(Box::new(SpawnProxy { runnable: r }))?;

        Ok(())
    }

    async fn get_game_action(_: &Jvm, _: &mut WIPIJavaContext, key: i32) -> JavaResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Display::getGameAction({})", key);

        let action = match key {
            -1 => 1, // UP
            -2 => 6, // DOWN
            -3 => 2, // LEFT
            -4 => 5, // RIGHT
            -5 => 8, // FIRE,
            _ => 0,
        };

        Ok(action)
    }
}
