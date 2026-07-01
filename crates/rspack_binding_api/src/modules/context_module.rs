use crate::{impl_module_methods, module::Module};

#[napi]
#[repr(C)]
pub struct ContextModule {
  pub(crate) module: Module,
}

impl ContextModule {
  pub(crate) fn into_module_instance(
    self,
    env: &napi::Env,
  ) -> napi::Result<napi::bindgen_prelude::ClassInstance<'_, Self>> {
    Self::new_inherited(self, env, &[])
  }
}

impl_module_methods!(ContextModule);
