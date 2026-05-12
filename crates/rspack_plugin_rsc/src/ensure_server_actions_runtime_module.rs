use indoc::formatdoc;
use rspack_core::{
  RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext, RuntimeModuleStage, RuntimeTemplate,
  impl_runtime_module,
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct RscEnsureServerActionsRuntimeModule {}

impl RscEnsureServerActionsRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(runtime_template)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for RscEnsureServerActionsRuntimeModule {
  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    Ok(formatdoc! {
      r#"
        {ensure_server_actions} = function(actions) {{
          for (var i = 0; i < actions.length; i++) {{
            var action = actions[i];
            if (typeof action !== "function") {{
              throw new Error('A "use server" file can only export async functions, found ' + typeof action + ".");
            }}
          }}
        }};
      "#,
      ensure_server_actions = context
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::RSC_ENSURE_SERVER_ACTIONS),
    })
  }
}
