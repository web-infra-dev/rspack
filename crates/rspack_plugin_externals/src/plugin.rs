use once_cell::sync::Lazy;
use regex::Regex;
use rspack_core::{
  ApplyContext, ExternalItem, ExternalItemValue, ExternalModule, ExternalType, FactorizeArgs,
  ModuleDependency, ModuleExt, ModuleFactoryResult, NormalModuleFactoryContext, Plugin,
  PluginContext, PluginFactorizeHookOutput,
};
use rspack_error::Result;

static UNSPECIFIED_EXTERNAL_TYPE_REGEXP: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^[a-z0-9-]+ ").expect("Invalid regex"));

#[derive(Debug)]
pub struct ExternalPlugin {
  externals: Vec<ExternalItem>,
  r#type: ExternalType,
}

impl ExternalPlugin {
  pub fn new(r#type: ExternalType, externals: Vec<ExternalItem>) -> Self {
    Self { externals, r#type }
  }

  fn handle_external(
    &self,
    config: &ExternalItemValue,
    r#type: Option<String>,
    dependency: &dyn ModuleDependency,
  ) -> Option<ExternalModule> {
    let mut external_module_config = match config {
      ExternalItemValue::String(config) => config,
      ExternalItemValue::Bool(config) => {
        if *config {
          dependency.request()
        } else {
          return None;
        }
      }
    };
    let external_module_type = r#type.unwrap_or_else(|| {
      if UNSPECIFIED_EXTERNAL_TYPE_REGEXP.is_match(external_module_config)
        && let Some((t, c)) = external_module_config.split_once(' ') {
        external_module_config = c;
        return t.to_owned();
      }
      self.r#type.clone()
    });
    Some(ExternalModule::new(
      external_module_config.to_owned(),
      external_module_type,
      dependency.request().to_owned(),
    ))
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
    for external_item in &self.externals {
      match external_item {
        ExternalItem::Object(eh) => {
          let request = args.dependency.request();

          if let Some(value) = eh.get(request) {
            let maybe_module = self.handle_external(value, None, args.dependency);
            return Ok(maybe_module.map(|i| ModuleFactoryResult::new(i.boxed())));
          }
        }
        ExternalItem::RegExp(r) => {
          let request = args.dependency.request();
          if r.test(request) {
            let maybe_module = self.handle_external(
              &ExternalItemValue::String(request.to_string()),
              None,
              args.dependency,
            );
            return Ok(maybe_module.map(|i| ModuleFactoryResult::new(i.boxed())));
          }
        }
        ExternalItem::String(s) => {
          let request = args.dependency.request();
          if s == request {
            let maybe_module = self.handle_external(
              &ExternalItemValue::String(request.to_string()),
              None,
              args.dependency,
            );
            return Ok(maybe_module.map(|i| ModuleFactoryResult::new(i.boxed())));
          }
        }
      }
    }
    Ok(None)
  }
}
