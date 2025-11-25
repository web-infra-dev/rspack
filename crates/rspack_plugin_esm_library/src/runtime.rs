use rspack_core::{
  Compilation, ModuleIdentifier, RuntimeGlobals, RuntimeModule, RuntimeTemplate,
  impl_runtime_module,
};

#[impl_runtime_module]
#[derive(Default, Debug)]
pub(crate) struct RegisterModuleRuntime {}

impl RegisterModuleRuntime {
  pub(crate) fn runtime_id(runtime_template: &RuntimeTemplate) -> String {
    format!(
      "{}.add",
      runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE)
    )
  }
}

#[async_trait::async_trait]
impl RuntimeModule for RegisterModuleRuntime {
  fn name(&self) -> ModuleIdentifier {
    "esm library register module runtime".into()
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    Ok(format!(
      "{} = function registerModules(modules) {{ Object.assign({}, modules) }}\n",
      Self::runtime_id(&compilation.runtime_template),
      compilation
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::MODULE_FACTORIES),
    ))
  }
}
