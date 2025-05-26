use crate::{impl_module_methods, Module};

#[napi]
#[repr(C)]
pub struct ContextModule {
  pub(crate) module: Module,
}

impl ContextModule {
  pub(crate) fn custom_into_instance(
    self,
    env: &napi::Env,
  ) -> napi::Result<napi::bindgen_prelude::ClassInstance<Self>> {
    Self::new_inherited(self, env, vec![])
  }
}

impl_module_methods!(ContextModule);
