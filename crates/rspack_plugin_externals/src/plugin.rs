use std::fmt::Debug;

use once_cell::sync::Lazy;
use regex::Regex;
use rspack_core::{
  ExternalItem, ExternalItemFnCtx, ExternalItemValue, ExternalModule, ExternalType, FactorizeArgs,
  ModuleDependency, ModuleExt, ModuleFactoryResult, NormalModuleFactoryContext, Plugin,
  PluginContext, PluginFactorizeHookOutput,
};

static UNSPECIFIED_EXTERNAL_TYPE_REGEXP: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^[a-z0-9-]+ ").expect("Invalid regex"));

pub struct ExternalsPlugin {
  externals: Vec<ExternalItem>,
  r#type: ExternalType,
}

impl Debug for ExternalsPlugin {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ExternalPlugin")
      .field("externals", &"Function")
      .field("r#type", &self.r#type)
      .finish()
  }
}

impl ExternalsPlugin {
  pub fn new(r#type: ExternalType, externals: Vec<ExternalItem>) -> Self {
    Self { externals, r#type }
  }

  fn handle_external(
    &self,
    config: &ExternalItemValue,
    r#type: Option<String>,
    dependency: &dyn ModuleDependency,
  ) -> Option<ExternalModule> {
    let mut external_module_config: Vec<String> = match config {
      ExternalItemValue::String(config) => vec![config.clone()],
      ExternalItemValue::Bool(config) => {
        if *config {
          vec![dependency.request().to_string()]
        } else {
          return None;
        }
      }
      ExternalItemValue::Array(config) => config.to_vec(),
    };

    let external_module_type = r#type.unwrap_or_else(|| {
      let head = external_module_config
        .get_mut(0)
        .expect("should have at least one element");
      if UNSPECIFIED_EXTERNAL_TYPE_REGEXP.is_match(head.as_str())
        && let Some((t, c)) = head.clone().as_str().split_once(' ') {
        *head = c.to_string();
        return t.to_owned();
      }
      self.r#type.clone()
    });
    Some(ExternalModule::new(
      external_module_config,
      external_module_type,
      dependency.request().to_owned(),
    ))
  }
}

#[async_trait::async_trait]
impl Plugin for ExternalsPlugin {
  fn name(&self) -> &'static str {
    "external"
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
        ExternalItem::Fn(f) => {
          let request = args.dependency.request();
          let context = args.context.to_string();
          let result = f(ExternalItemFnCtx {
            context,
            request: request.to_string(),
            dependency_type: args.dependency.category().to_string(),
          })
          .await?;
          if let Some(r) = result.result {
            let maybe_module = self.handle_external(&r, result.external_type, args.dependency);
            return Ok(maybe_module.map(|i| ModuleFactoryResult::new(i.boxed())));
          }
        }
      }
    }
    Ok(None)
  }
}
