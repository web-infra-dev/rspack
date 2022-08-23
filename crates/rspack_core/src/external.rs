use rspack_error::Error;
use rspack_loader_runner::Content;

use crate::{
  ApplyContext, FactorizeArgs, ModuleType, NormalModuleFactoryContext, ParseModuleArgs, Plugin,
  PluginContext, PluginFactorizeHookOutput,
};

#[derive(Debug)]
pub struct ExternalPlugin {}

impl Plugin for ExternalPlugin {
  fn name(&self) -> &'static str {
    "external"
  }

  fn apply(&mut self, _ctx: PluginContext<&mut ApplyContext>) -> Result<(), Error> {
    Ok(())
  }

  fn factorize(
    &self,
    _ctx: PluginContext,
    args: FactorizeArgs,
    job_ctx: &mut NormalModuleFactoryContext,
  ) -> PluginFactorizeHookOutput {
    for external_item in &job_ctx.options.external {
      match external_item {
        crate::External::Object(eh) => {
          let specifier = args.dependency.detail.specifier.as_str();
          if let Some(value) = eh.get(specifier) {
            job_ctx.module_type = Some(ModuleType::Js);
            let module = args.plugin_driver.parse(
              ParseModuleArgs {
                uri: specifier,
                options: job_ctx.options.clone(),
                source: Content::Buffer(format!("module.exports = {}", value).as_bytes().to_vec()),
              },
              job_ctx,
            )?;
            tracing::trace!("parsed module {:?}", module);
            return Ok(Some((specifier.to_string(), module)));
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
