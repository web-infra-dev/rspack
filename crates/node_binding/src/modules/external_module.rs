use napi::Either;

use crate::{impl_module_methods, Module};

#[napi]
pub struct ExternalModule {
  pub(crate) module: Module,
}

#[napi]
impl ExternalModule {
  #[napi(getter)]
  pub fn user_request(&mut self) -> napi::Result<Either<&str, ()>> {
    let (_, module) = self.module.as_ref()?;

    Ok(match module.as_external_module() {
      Some(external_module) => Either::A(external_module.user_request()),
      None => Either::B(()),
    })
  }

  #[napi(setter)]
  pub fn set_user_request(&mut self, val: Either<String, ()>) -> napi::Result<()> {
    match val {
      Either::A(val) => {
        let module: &mut dyn rspack_core::Module = self.module.as_mut()?;
        if let Some(external_module) = module.as_external_module_mut() {
          *external_module.user_request_mut() = val;
        }
      }
      Either::B(_) => {}
    }
    Ok(())
  }
}

impl_module_methods!(ExternalModule);
