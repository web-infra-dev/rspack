use rspack_core::{
  ApplyContext, ExternalItem, ExternalModule, ExternalType, FactorizeArgs, ModuleExt,
  ModuleFactoryResult, NormalModuleFactoryContext, Plugin, PluginContext,
  PluginFactorizeHookOutput,
};
use rspack_error::Result;

#[derive(Debug)]
pub struct ExternalPlugin {
  externals: Vec<ExternalItem>,
  r#type: ExternalType,
}

impl ExternalPlugin {
  pub fn new(r#type: ExternalType, externals: Vec<ExternalItem>) -> Self {
    Self { externals, r#type }
  }
}

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
    _job_ctx: &mut NormalModuleFactoryContext,
  ) -> PluginFactorizeHookOutput {
    let external_type = &self.r#type;
    for external_item in &self.externals {
      match external_item {
        ExternalItem::Object(eh) => {
          let request = args.dependency.request();

          if let Some(value) = eh.get(request) {
            let external_module = ExternalModule::new(
              value.to_owned(),
              external_type.to_owned(),
              request.to_string(),
            );
            return Ok(Some(ModuleFactoryResult::new(external_module.boxed())));
          }
        }
        ExternalItem::RegExp(r) => {
          let request = args.dependency.request();
          if r.test(request) {
            let external_module = ExternalModule::new(
              request.to_owned(),
              external_type.to_owned(),
              request.to_string(),
            );
            return Ok(Some(ModuleFactoryResult::new(external_module.boxed())));
          }
        }
        ExternalItem::String(s) => {
          let request = args.dependency.request();
          if s == request {
            let external_module = ExternalModule::new(
              request.to_owned(),
              external_type.to_owned(),
              request.to_string(),
            );
            return Ok(Some(ModuleFactoryResult::new(external_module.boxed())));
          }
        }
      }
    }
    Ok(None)
  }
}
