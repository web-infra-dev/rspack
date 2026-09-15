use crate::{
  impl_module_methods,
  module::{MODULE_PROPERTIES_BUFFER, Module},
};

#[napi]
#[repr(C)]
pub struct ExternalModule {
  pub(crate) module: Module,
}

impl ExternalModule {
  pub(crate) fn into_module_instance(
    mut self,
    env: &napi::Env,
  ) -> napi::Result<napi::bindgen_prelude::ClassInstance<'_, Self>> {
    let user_request = self.with_ref(|_, module| env.create_string(module.user_request()))?;

    MODULE_PROPERTIES_BUFFER.with(|ref_cell| {
      let mut properties = ref_cell.borrow_mut();
      properties.clear();

      properties.push(
        napi::Property::new()
          .with_utf8_name("userRequest")?
          .with_value(&user_request),
      );
      Self::new_inherited(self, env, &mut properties)
    })
  }

  fn with_ref<R>(
    &mut self,
    f: impl FnOnce(&rspack_core::Compilation, &rspack_core::ExternalModule) -> napi::Result<R>,
  ) -> napi::Result<R> {
    self
      .module
      .with_ref(|compilation, module| match module.as_external_module() {
        Some(external_module) => f(compilation, external_module),
        None => Err(napi::Error::new(
          napi::Status::GenericFailure,
          "Module is not a ExternalModule",
        )),
      })
  }
}

impl_module_methods!(ExternalModule);
