use napi::Either;

use crate::{impl_module_methods, Module, ModuleObject};

#[napi]
pub struct ConcatenatedModule {
  pub(crate) module: Module,
}

#[napi]
impl ConcatenatedModule {
  #[napi(getter, ts_return_type = "Module[] | undefined")]
  pub fn modules(&mut self) -> napi::Result<Either<Vec<ModuleObject>, ()>> {
    let (compilation, module) = self.module.as_ref()?;

    Ok(match module.as_concatenated_module() {
      Some(concatenated_module) => {
        let inner_modules = concatenated_module
          .get_modules()
          .iter()
          .filter_map(|inner_module_info| {
            compilation
              .module_by_identifier(&inner_module_info.id)
              .map(|module| ModuleObject::with_ref(module.as_ref(), compilation.compiler_id()))
          })
          .collect::<Vec<_>>();
        Either::A(inner_modules)
      }
      None => Either::B(()),
    })
  }
}

impl_module_methods!(ConcatenatedModule);
