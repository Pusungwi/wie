use alloc::{boxed::Box, rc::Rc, string::String, vec, vec::Vec};
use core::time::Duration;

use wie_backend::{AsyncCallable, SystemHandle};
use wie_core_arm::ArmCore;

use java_runtime::Runtime;
use jvm::{Jvm, JvmCallback};

#[derive(Clone)]
pub struct KtfRuntime {
    core: ArmCore,
    system: SystemHandle,
    jvm: Rc<Jvm>,
}

impl KtfRuntime {
    pub fn new(core: &mut ArmCore, system: &mut SystemHandle, jvm: Rc<Jvm>) -> Self {
        Self {
            core: core.clone(),
            system: system.clone(),
            jvm,
        }
    }
}

#[async_trait::async_trait(?Send)]
impl Runtime for KtfRuntime {
    async fn sleep(&self, duration: Duration) {
        let now = self.system.platform().now();
        let until = now + duration.as_millis() as u64;

        self.system.clone().sleep(until).await; // TODO remove clone
    }

    async fn r#yield(&self) {
        self.system.yield_now().await;
    }

    fn spawn(&self, callback: Box<dyn JvmCallback>) {
        struct SpawnProxy {
            jvm: Rc<Jvm>,
            callback: Box<dyn JvmCallback>,
        }

        #[async_trait::async_trait(?Send)]
        impl AsyncCallable<u32, anyhow::Error> for SpawnProxy {
            async fn call(mut self) -> Result<u32, anyhow::Error> {
                self.callback.call(&self.jvm, vec![].into_boxed_slice()).await?;

                Ok(0) // TODO
            }
        }

        self.core.clone().spawn(SpawnProxy {
            jvm: self.jvm.clone(),
            callback,
        });
    }

    fn now(&self) -> u64 {
        self.system.platform().now().raw()
    }

    fn encode_str(&self, s: &str) -> Vec<u8> {
        self.system.encode_str(s)
    }

    fn decode_str(&self, bytes: &[u8]) -> String {
        self.system.decode_str(bytes)
    }

    fn println(&mut self, s: &str) {
        tracing::info!("println {}", s);
    }
}
