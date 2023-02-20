use rspack_core::{
  ApplyContext, External, FactorizeArgs, ModuleExt, ModuleFactoryResult,
  NormalModuleFactoryContext, Plugin, PluginContext, PluginFactorizeHookOutput,
};
use rspack_error::Result;

use crate::ExternalModule;

#[derive(Debug, Default)]
pub struct ExternalPlugin {}

#[async_trait::async_trait]
impl Plugin for ExternalPlugin {
  fn name(&self) -> &'static str {
    "external"
  }

  fn apply(&mut self, _ctx: PluginContext<&mut ApplyContext>) -> Result<()> {
    Ok(())
  }

  async fn factorize(
    &self,
    _ctx: PluginContext,
    args: FactorizeArgs<'_>,
    job_ctx: &mut NormalModuleFactoryContext,
  ) -> PluginFactorizeHookOutput {
    let target = &job_ctx.options.target;
    let external_type = &job_ctx.options.externals_type;
    for external_item in &job_ctx.options.externals {
      match external_item {
        External::Object(eh) => {
          let specifier = args.dependency.request();

          if let Some(value) = eh.get(specifier) {
            let external_module = ExternalModule::new(
              value.to_owned(),
              external_type.to_owned(),
              target.to_owned(),
              specifier.to_string(),
            );
            return Ok(Some(ModuleFactoryResult::new(external_module.boxed())));
          }
        }
        _ => {
          return Ok(None);
        }
      }
    }
    Ok(None)
  }
}
