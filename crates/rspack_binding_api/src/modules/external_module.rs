use crate::{impl_module_methods, module::Module};

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
    Self::new_inherited(self, env, &["userRequest"])
  }

  fn as_ref(&mut self) -> napi::Result<(&rspack_core::Compilation, &rspack_core::ExternalModule)> {
    let (compilation, module) = self.module.as_ref()?;
    match module.as_external_module() {
      Some(external_module) => Ok((compilation, external_module)),
      None => Err(napi::Error::new(
        napi::Status::GenericFailure,
        "Module is not a ExternalModule",
      )),
    }
  }
}

#[napi]
impl ExternalModule {
  #[napi(skip_typescript, getter, js_name = "userRequest")]
  pub fn user_request(&mut self) -> napi::Result<String> {
    let (_, module) = self.as_ref()?;
    Ok(module.user_request().to_string())
  }
}

impl_module_methods!(ExternalModule);
