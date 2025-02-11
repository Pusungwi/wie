use alloc::{boxed::Box, rc::Rc};

use java_class_proto::MethodBody;
use jvm::{Jvm, JvmResult};

use wie_backend::{AsyncCallable, SystemHandle};
use wie_core_arm::ArmCore;
use wie_wipi_java::WIPIJavaContextBase;

#[derive(Clone)]
pub struct KtfWIPIJavaContext {
    core: ArmCore,
    system: SystemHandle,
    jvm: Rc<Jvm>,
}

impl KtfWIPIJavaContext {
    pub fn new(core: &ArmCore, system: &SystemHandle, jvm: Rc<Jvm>) -> Self {
        Self {
            core: core.clone(),
            system: system.clone(),
            jvm,
        }
    }
}

#[async_trait::async_trait(?Send)]
impl WIPIJavaContextBase for KtfWIPIJavaContext {
    fn system(&mut self) -> &mut SystemHandle {
        &mut self.system
    }

    fn spawn(&mut self, callback: Box<dyn MethodBody<anyhow::Error, dyn WIPIJavaContextBase>>) -> JvmResult<()> {
        struct SpawnProxy {
            core: ArmCore,
            system: SystemHandle,
            jvm: Rc<Jvm>,
            callback: Box<dyn MethodBody<anyhow::Error, dyn WIPIJavaContextBase>>,
        }

        #[async_trait::async_trait(?Send)]
        impl AsyncCallable<u32, anyhow::Error> for SpawnProxy {
            async fn call(mut self) -> Result<u32, anyhow::Error> {
                let mut context = KtfWIPIJavaContext::new(&self.core, &self.system, self.jvm.clone());

                let _ = self.callback.call(&self.jvm, &mut context, Box::new([])).await?;

                Ok(0) // TODO resturn value
            }
        }

        self.core.spawn(SpawnProxy {
            core: self.core.clone(),
            system: self.system.clone(),
            jvm: self.jvm.clone(),
            callback,
        });

        Ok(())
    }
}
