use napi::{Result, bindgen_prelude::*};

use crate::{
  module::{ModuleHandle, ModuleObject},
  module_bridge::ModuleRef,
};

impl ModuleHandle {
  fn with_concatenated<R>(
    &self,
    f: impl FnOnce(&rspack_core::Compilation, &rspack_core::ConcatenatedModule) -> Result<R>,
  ) -> Result<R> {
    self.with_ref(
      |compilation, module| match module.as_concatenated_module() {
        Some(module) => f(compilation, module),
        None => Err(Error::from_reason("Module is not a ConcatenatedModule")),
      },
    )
  }
}

#[napi(ts_args_type = "module: Module", ts_return_type = "Module")]
pub fn concatenated_module_get_root_module(module: ModuleRef<'_>) -> Result<ModuleObject> {
  module.with_concatenated(|compilation, module| {
    let root = compilation
      .module_by_identifier(&module.get_root())
      .expect("Root module should exist");
    Ok(ModuleObject::with_ref(
      root.as_ref(),
      compilation.compiler_id(),
    ))
  })
}
#[napi(ts_args_type = "module: Module", ts_return_type = "Module[]")]
pub fn concatenated_module_get_modules(module: ModuleRef<'_>) -> Result<Vec<ModuleObject>> {
  module.with_concatenated(|compilation, module| {
    Ok(
      module
        .get_modules()
        .iter()
        .filter_map(|inner| {
          compilation
            .module_by_identifier(&inner.id)
            .map(|module| ModuleObject::with_ref(module.as_ref(), compilation.compiler_id()))
        })
        .collect(),
    )
  })
}
