use crate::{
  impl_module_methods,
  module::{MODULE_PROPERTIES_BUFFER, Module},
};

#[napi]
#[repr(C)]
pub struct ContextModule {
  pub(crate) module: Module,
}

impl ContextModule {
  pub(crate) fn custom_into_instance(
    self,
    env: &napi::Env,
  ) -> napi::Result<napi::bindgen_prelude::ClassInstance<'_, Self>> {
    MODULE_PROPERTIES_BUFFER.with(|cell| {
      let mut properties = cell.take();
      properties.clear();

      let result = Self::new_inherited(self, env, &mut properties);
      cell.set(properties);
      result
    })
  }
}

impl_module_methods!(ContextModule);
