use rspack_collections::Identifier;
use rspack_core::{impl_runtime_module, Compilation, RuntimeModule};

#[impl_runtime_module]
#[derive(Debug)]
pub struct ESMModuleDecoratorRuntimeModule {
  id: Identifier,
}

impl Default for ESMModuleDecoratorRuntimeModule {
  fn default() -> Self {
    Self::with_default(Identifier::from("webpack/runtime/esm_module_decorator"))
  }
}

impl RuntimeModule for ESMModuleDecoratorRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, _compilation: &Compilation) -> rspack_error::Result<String> {
    Ok(include_str!("runtime/esm_module_decorator.js").to_string())
  }
}
