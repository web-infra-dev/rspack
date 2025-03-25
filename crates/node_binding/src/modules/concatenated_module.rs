use napi::{bindgen_prelude::This, Env};

use crate::{impl_module_methods, Module, ModuleObject};

#[napi]
pub struct ConcatenatedModule {
  pub(crate) module: Module,
}

impl ConcatenatedModule {
  fn as_ref(&self) -> napi::Result<&rspack_core::ConcatenatedModule> {
    match self.module.0.as_ref().as_concatenated_module() {
      Some(concatenated_module) => Ok(concatenated_module),
      None => Err(napi::Error::new(
        napi::Status::GenericFailure,
        "Module is not a ConcatenatedModule",
      )),
    }
  }
}

#[napi]
impl ConcatenatedModule {
  #[napi(getter, ts_return_type = "Module[]")]
  pub fn modules(&mut self, env: &Env, this: This) -> napi::Result<Vec<ModuleObject>> {
    let module = self.as_ref()?;
    let Some(compilation) = self.get_compilation_ref(env, this)? else {
      return Ok(vec![]);
    };

    let inner_modules = module
      .get_modules()
      .iter()
      .filter_map(|inner_module_info| {
        compilation
          .module_by_identifier(&inner_module_info.id)
          .map(|module| ModuleObject::new(module.as_ref(), compilation.id()))
      })
      .collect::<Vec<_>>();
    Ok(inner_modules)
  }
}

impl_module_methods!(ConcatenatedModule);
