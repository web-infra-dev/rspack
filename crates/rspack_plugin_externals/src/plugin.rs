use std::fmt::Debug;

use once_cell::sync::Lazy;
use regex::Regex;
use rspack_core::{
  ExternalItem, ExternalItemFnCtx, ExternalItemValue, ExternalModule, ExternalRequest,
  ExternalRequestValue, ExternalType, FactorizeArgs, ModuleDependency, ModuleExt,
  ModuleFactoryResult, Plugin, PluginContext, PluginFactorizeHookOutput,
};

static UNSPECIFIED_EXTERNAL_TYPE_REGEXP: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^[a-z0-9-]+ ").expect("Invalid regex"));

#[derive(Debug)]
pub struct ExternalsPlugin {
  externals: Vec<ExternalItem>,
  r#type: ExternalType,
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
    let (external_module_config, external_module_type) = match config {
      ExternalItemValue::String(config) => {
        let (external_type, config) =
          if let Some((external_type, new_config)) = parse_external_type_from_str(config) {
            (external_type, new_config)
          } else {
            (self.r#type.clone(), config.to_owned())
          };
        (
          ExternalRequest::Single(ExternalRequestValue::new(config, None)),
          external_type,
        )
      }
      ExternalItemValue::Array(arr) => {
        let mut iter = arr.iter().peekable();
        let primary = iter.next()?;
        let (external_type, primary) =
          if let Some((external_type, new_primary)) = parse_external_type_from_str(primary) {
            (external_type, new_primary)
          } else {
            (self.r#type.clone(), primary.to_owned())
          };
        let rest = iter.peek().is_some().then(|| iter.cloned().collect());
        (
          ExternalRequest::Single(ExternalRequestValue::new(primary, rest)),
          external_type,
        )
      }
      ExternalItemValue::Bool(config) => {
        if *config {
          (
            ExternalRequest::Single(ExternalRequestValue::new(
              dependency.request().to_string(),
              None,
            )),
            self.r#type.clone(),
          )
        } else {
          return None;
        }
      }
      ExternalItemValue::Object(map) => (
        ExternalRequest::Map(
          map
            .iter()
            .map(|(k, v)| {
              let mut iter = v.iter().peekable();
              let primary = iter.next().expect("should have at least one value");
              let rest = iter.peek().is_some().then(|| iter.cloned().collect());
              (
                k.clone(),
                ExternalRequestValue::new(primary.to_owned(), rest),
              )
            })
            .collect(),
        ),
        self.r#type.clone(),
      ),
    };

    fn parse_external_type_from_str(v: &str) -> Option<(ExternalType, String)> {
      if UNSPECIFIED_EXTERNAL_TYPE_REGEXP.is_match(v)
        && let Some((t, c)) = v.split_once(' ')
      {
        return Some((t.to_owned(), c.to_owned()));
      }
      None
    }

    Some(ExternalModule::new(
      external_module_config,
      r#type.unwrap_or(external_module_type),
      dependency.request().to_owned(),
    ))
  }
}

#[async_trait::async_trait]
impl Plugin for ExternalsPlugin {
  fn name(&self) -> &'static str {
    "rspack.ExternalsPlugin"
  }

  async fn factorize(
    &self,
    _ctx: PluginContext,
    args: FactorizeArgs<'_>,
  ) -> PluginFactorizeHookOutput {
    for external_item in &self.externals {
      match external_item {
        ExternalItem::Object(eh) => {
          let request = args.dependency.request();

          if let Some(value) = eh.get(request) {
            let maybe_module = self.handle_external(value, None, args.dependency);
            return Ok(maybe_module.map(|i| ModuleFactoryResult::new_with_module(i.boxed())));
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
            return Ok(maybe_module.map(|i| ModuleFactoryResult::new_with_module(i.boxed())));
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
            return Ok(maybe_module.map(|i| ModuleFactoryResult::new_with_module(i.boxed())));
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
            return Ok(maybe_module.map(|i| ModuleFactoryResult::new_with_module(i.boxed())));
          }
        }
      }
    }
    Ok(None)
  }
}
