use rspack_error::Error;
use rspack_loader_runner::Content;

use crate::{
  ApplyContext, ExternalType, FactorizeAndBuildArgs, ModuleType, NormalModuleFactoryContext,
  ParseModuleArgs, Plugin, PluginContext, PluginFactorizeAndBuildHookOutput, TargetPlatform,
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

  // Todo The factorize_and_build hook is a temporary solution and will be replaced with the real factorize hook later
  // stage 1: we need move building function(parse,loader runner) out of normal module factory
  // stage 2: Create a new hook that is the same as factory in webpack and change factorize_and_build to that
  fn factorize_and_build(
    &self,
    _ctx: PluginContext,
    args: FactorizeAndBuildArgs,
    job_ctx: &mut NormalModuleFactoryContext,
  ) -> PluginFactorizeAndBuildHookOutput {
    let target = &job_ctx.options.target;
    let external_type = &job_ctx.options.external_type;
    for external_item in &job_ctx.options.external {
      match external_item {
        crate::External::Object(eh) => {
          let specifier = args.dependency.detail.specifier.as_str();
          if let Some(value) = eh.get(specifier) {
            job_ctx.module_type = Some(ModuleType::Js);
            let module = args.plugin_driver.parse(
              ParseModuleArgs {
                uri: specifier,
                meta: None,
                options: job_ctx.options.clone(),
                source: Content::Buffer(
                  (match external_type {
                    ExternalType::NodeCommonjs => {
                      // format!("module.exports = {}", value)
                      format!(r#"module.exports = require("{}")"#, value)
                    }
                    ExternalType::Window => {
                      format!("module.exports = window.{}", value)
                    }
                    ExternalType::Auto => match target.platform {
                      TargetPlatform::BrowsersList
                      | TargetPlatform::Web
                      | TargetPlatform::WebWorker
                      | TargetPlatform::None => format!("module.exports = {}", value),
                      TargetPlatform::Node(_) => {
                        format!(r#"module.exports = __rspack_require__.nr("{}")"#, value)
                        // format!(r#"module.exports = require("{}")"#, value)
                      }
                    },
                  })
                  .as_bytes()
                  .to_vec(),
                ),
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
