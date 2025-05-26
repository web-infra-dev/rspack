use crate::{impl_module_methods, Module, ModuleObject};

#[napi]
#[repr(C)]
pub struct ConcatenatedModule {
  pub(crate) module: Module,
}

impl ConcatenatedModule {
  pub(crate) fn custom_into_instance(
    self,
    env: &napi::Env,
  ) -> napi::Result<napi::bindgen_prelude::ClassInstance<Self>> {
    Self::new_inherited(self, env, vec![])
  }

  fn as_ref(
    &mut self,
  ) -> napi::Result<(&rspack_core::Compilation, &rspack_core::ConcatenatedModule)> {
    let (compilation, module) = self.module.as_ref()?;
    match module.as_concatenated_module() {
      Some(concatenated_module) => Ok((compilation, concatenated_module)),
      None => Err(napi::Error::new(
        napi::Status::GenericFailure,
        "Module is not a ConcatenatedModule",
      )),
    }
  }
}

#[napi]
impl ConcatenatedModule {
  #[napi(getter, ts_return_type = "Module")]
  pub fn root_module(&mut self) -> napi::Result<ModuleObject> {
    let (compilation, module) = self.as_ref()?;
    let root_module = compilation
      .module_by_identifier(&module.get_root())
      .expect("Root module should exist");
    Ok(ModuleObject::with_ref(
      root_module.as_ref(),
      compilation.compiler_id(),
    ))
  }

  #[napi(getter, ts_return_type = "Module[]")]
  pub fn modules(&mut self) -> napi::Result<Vec<ModuleObject>> {
    let (compilation, module) = self.as_ref()?;

    let inner_modules = module
      .get_modules()
      .iter()
      .filter_map(|inner_module_info| {
        compilation
          .module_by_identifier(&inner_module_info.id)
          .map(|module| ModuleObject::with_ref(module.as_ref(), compilation.compiler_id()))
      })
      .collect::<Vec<_>>();
    Ok(inner_modules)
  }
}

impl_module_methods!(ConcatenatedModule);
