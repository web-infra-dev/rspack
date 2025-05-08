use rspack_core::{
  Compilation, ModuleIdentifier, RuntimeGlobals, RuntimeModule, impl_runtime_module,
};

#[impl_runtime_module]
#[derive(Default, Debug)]
pub(crate) struct RegisterModuleRuntime {}

impl RegisterModuleRuntime {
  pub(crate) fn runtime_id() -> &'static str {
    "__webpack_require__.add"
  }
}

#[async_trait::async_trait]
impl RuntimeModule for RegisterModuleRuntime {
  fn name(&self) -> ModuleIdentifier {
    "esm library register module runtime".into()
  }

  async fn generate(&self, _compilation: &Compilation) -> rspack_error::Result<String> {
    Ok(format!(
      "{} = function registerModules(modules) {{ Object.assign({}, modules) }}\n",
      Self::runtime_id(),
      RuntimeGlobals::MODULE_FACTORIES,
    ))
  }
}
