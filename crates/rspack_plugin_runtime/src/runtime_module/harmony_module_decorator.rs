use rspack_collections::Identifier;
use rspack_core::{impl_runtime_module, Compilation, RuntimeModule};

#[impl_runtime_module]
#[derive(Debug)]
pub struct HarmonyModuleDecoratorRuntimeModule {
  id: Identifier,
}

impl HarmonyModuleDecoratorRuntimeModule {
  fn generate(&self, _compilation: &Compilation) -> rspack_error::Result<String> {
    Ok(include_str!("runtime/harmony_module_decorator.js").to_string())
  }
}

impl Default for HarmonyModuleDecoratorRuntimeModule {
  fn default() -> Self {
    Self::with_default(Identifier::from("webpack/runtime/harmony_module_decorator"))
  }
}

impl RuntimeModule for HarmonyModuleDecoratorRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }
}
