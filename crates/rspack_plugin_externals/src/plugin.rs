use std::{fmt::Debug, sync::Arc};

use rspack_core::{
  BoxModule, ContextInfo, DependencyMeta, DependencyType, ExternalInterop, ExternalItem,
  ExternalItemFnCtx, ExternalItemValue, ExternalModule, ExternalRequest, ExternalRequestValue,
  ExternalType, ExternalTypeEnum, ModuleDependency, ModuleExt, ModuleFactoryCreateData,
  NormalModuleFactoryFactorize, Plugin, ResolveOptionsWithDependencyType, SourceType,
};
use rspack_error::{Result, error};
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_css::dependency::CssImportDependency;
use rspack_plugin_javascript::dependency::{ESMImportSideEffectDependency, ImportDependency};

#[plugin]
#[derive(Debug)]
pub struct ExternalsPlugin {
  externals: Vec<ExternalItem>,
  r#type: ExternalType,
  place_in_initial: bool,
  fallback_type: ExternalType,
}

impl ExternalsPlugin {
  pub fn new(r#type: ExternalType, externals: Vec<ExternalItem>, place_in_initial: bool) -> Self {
    Self::new_with_options(r#type, externals, place_in_initial, "commonjs".to_string())
  }

  pub fn new_with_options(
    r#type: ExternalType,
    externals: Vec<ExternalItem>,
    place_in_initial: bool,
    fallback_type: ExternalType,
  ) -> Self {
    Self::new_inner(externals, r#type, place_in_initial, fallback_type)
  }

  fn handle_external(
    &self,
    config: &ExternalItemValue,
    r#type: Option<String>,
    dependency: &dyn ModuleDependency,
  ) -> Result<Option<ExternalModule>> {
    let interop = if let ExternalItemValue::Object(map) = config
      && let Some(value) = map.get("interop")
    {
      match value.as_slice() {
        [value] if value == "default" => Some(ExternalInterop::Default),
        [value] if value == "esModule" => Some(ExternalInterop::EsModule),
        _ => {
          return Err(error!(
            "Invalid interop for external '{}': expected 'default' or 'esModule'",
            dependency.request()
          ));
        }
      }
    } else {
      None
    };
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
        let Some(primary) = iter.next() else {
          return Ok(None);
        };
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
          return Ok(None);
        }
      }
      ExternalItemValue::Object(map) => (
        ExternalRequest::Map(
          map
            .iter()
            .filter(|(key, _)| key.as_str() != "interop")
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
      let type_end = v
        .bytes()
        .take_while(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-')
        .count();
      if type_end > 0 && v.as_bytes().get(type_end) == Some(&b' ') {
        return Some((v[..type_end].to_owned(), v[type_end + 1..].to_owned()));
      }
      None
    }

    let dependency_type = *dependency.dependency_type();
    let is_import_dependency = dependency
      .as_any()
      .downcast_ref::<ImportDependency>()
      .is_some();
    let is_esm_import_side_effect_dependency = dependency
      .as_any()
      .downcast_ref::<ESMImportSideEffectDependency>()
      .is_some();

    let mut dependency_meta: DependencyMeta = DependencyMeta {
      attributes: dependency.get_attributes().cloned(),
      phase: dependency.get_phase(),
      external_type: {
        if is_import_dependency
          || matches!(
            dependency_type,
            DependencyType::DynamicImport
              | DependencyType::DynamicImportEager
              | DependencyType::LazyImport
          )
        {
          Some(ExternalTypeEnum::Import)
        } else if is_esm_import_side_effect_dependency {
          Some(ExternalTypeEnum::Module)
        } else {
          None
        }
      },
      source_type: None,
      css_import_conditions: dependency
        .as_any()
        .downcast_ref::<CssImportDependency>()
        .map(|dependency| dependency.render_condition().clone()),
    };

    let external_module_type = r#type.unwrap_or(external_module_type);
    if matches!(external_module_type.as_str(), "asset" | "asset-url")
      && matches!(dependency.dependency_type(), DependencyType::CssUrl)
    {
      dependency_meta.source_type = Some(SourceType::AssetUrl);
    }

    if external_module_type == "modern-module"
      && matches!(
        dependency_type,
        DependencyType::CjsRequire
          | DependencyType::CjsFullRequire
          | DependencyType::CjsExportRequire
          | DependencyType::CommonJSRequireContext
          | DependencyType::RequireContext
          | DependencyType::RequireResolve
          | DependencyType::RequireResolveContext
      )
    {
      dependency_meta.external_type = Some(ExternalTypeEnum::CommonJs(self.fallback_type.clone()));
    }

    Ok(Some(ExternalModule::new(
      external_module_config,
      external_module_type,
      dependency.request().to_owned(),
      dependency_meta,
      self.place_in_initial,
      interop,
    )))
  }
}

#[plugin_hook(NormalModuleFactoryFactorize for ExternalsPlugin, tracing=false)]
async fn factorize(&self, data: &mut ModuleFactoryCreateData) -> Result<Option<BoxModule>> {
  let dependency = data.dependencies[0]
    .as_module_dependency()
    .expect("should be module dependency");
  let context = &data.context;
  for external_item in &self.externals {
    match external_item {
      ExternalItem::Object(eh) => {
        let request = dependency.request();

        if let Some(value) = eh.get(request) {
          let maybe_module = self.handle_external(value, None, dependency)?;
          return Ok(maybe_module.map(|i| i.boxed()));
        }
      }
      ExternalItem::ByLayer(layers) => {
        let externals = layers
          .get(data.issuer_layer.as_deref().unwrap_or("null"))
          .or_else(|| layers.get("default"));
        if let Some(value) = externals.and_then(|externals| externals.get(dependency.request())) {
          return Ok(
            self
              .handle_external(value, None, dependency)?
              .map(|i| i.boxed()),
          );
        }
      }
      ExternalItem::RegExp(r) => {
        let request = dependency.request();
        if r.test(request) {
          let maybe_module = self.handle_external(
            &ExternalItemValue::String(request.to_string()),
            None,
            dependency,
          )?;
          return Ok(maybe_module.map(|i| i.boxed()));
        }
      }
      ExternalItem::String(s) => {
        let request = dependency.request();
        if s == request {
          let maybe_module = self.handle_external(
            &ExternalItemValue::String(request.to_string()),
            None,
            dependency,
          )?;
          return Ok(maybe_module.map(|i| i.boxed()));
        }
      }
      ExternalItem::RequestPredicate(predicate) => {
        if predicate(dependency.request()) {
          let maybe_module =
            self.handle_external(&ExternalItemValue::Bool(true), None, dependency)?;
          return Ok(maybe_module.map(|i| i.boxed()));
        }
      }
      ExternalItem::Fn(f) => {
        let request = dependency.request();
        let result = f(ExternalItemFnCtx {
          context: context.to_string(),
          request: request.to_string(),
          dependency_type: dependency.category().to_string(),
          context_info: ContextInfo {
            issuer: data
              .issuer
              .clone()
              .map(|i| i.to_string())
              .unwrap_or_default(),
            issuer_layer: data.issuer_layer.clone(),
          },
          resolve_options_with_dependency_type: ResolveOptionsWithDependencyType {
            resolve_options: data
              .resolve_options
              .clone()
              .map(|r| Box::new(Arc::unwrap_or_clone(r))),
            resolve_to_context: false,
            dependency_category: *data
              .dependencies
              .first()
              .expect("Expected at least one dependency")
              .category(),
          },
          resolver_factory: data.build_context.resolver_factory.clone(),
        })
        .await?;
        if let Some(r) = result.result {
          let maybe_module = self.handle_external(&r, result.external_type, dependency)?;
          return Ok(maybe_module.map(|i| i.boxed()));
        }
      }
    }
  }
  Ok(None)
}

impl Plugin for ExternalsPlugin {
  fn name(&self) -> &'static str {
    "rspack.ExternalsPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .normal_module_factory_hooks
      .factorize
      .tap(factorize::new(self));
    Ok(())
  }
}
