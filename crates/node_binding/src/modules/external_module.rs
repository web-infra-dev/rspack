use crate::{impl_module_methods, Module};

#[napi]
pub struct ExternalModule {
  pub(crate) module: Module,
}

impl ExternalModule {
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

  fn as_mut(&mut self) -> napi::Result<&mut rspack_core::ExternalModule> {
    let module = self.module.as_mut()?;
    match module.as_external_module_mut() {
      Some(external_module) => Ok(external_module),
      None => Err(napi::Error::new(
        napi::Status::GenericFailure,
        "Module is not a ExternalModule",
      )),
    }
  }
}

#[napi]
impl ExternalModule {
  #[napi(getter)]
  pub fn user_request(&mut self) -> napi::Result<&str> {
    let (_, module) = self.as_ref()?;

    Ok(module.user_request())
  }

  #[napi(setter)]
  pub fn set_user_request(&mut self, val: String) -> napi::Result<()> {
    let module = self.as_mut()?;

    *module.user_request_mut() = val;
    Ok(())
  }
}

impl_module_methods!(ExternalModule);
