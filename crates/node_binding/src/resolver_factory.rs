use std::sync::Arc;

use napi_derive::napi;
use rspack_core::{Resolve, ResolverFactory};
use rspack_fs::{NativeFileSystem, ReadableFileSystem};

use crate::{
  raw_resolve::{
    normalize_raw_resolve_options_with_dependency_type, RawResolveOptionsWithDependencyType,
  },
  JsResolver,
};

#[napi]
pub struct JsResolverFactory {
  pub(crate) resolver_factory: Option<Arc<ResolverFactory>>,
  pub(crate) loader_resolver_factory: Option<Arc<ResolverFactory>>,
  pub(crate) input_filesystem: Arc<dyn ReadableFileSystem>,
}

#[napi]
impl JsResolverFactory {
  #[napi(constructor)]
  pub fn new() -> napi::Result<Self> {
    let input_filesystem = Arc::new(NativeFileSystem {});
    Ok(Self {
      resolver_factory: None,
      loader_resolver_factory: None,
      input_filesystem,
    })
  }

  pub fn get_resolver_factory(&mut self, resolve_options: Resolve) -> Arc<ResolverFactory> {
    match &self.resolver_factory {
      Some(resolver_factory) => resolver_factory.clone(),

      None => {
        let resolver_factory = Arc::new(ResolverFactory::new(
          resolve_options,
          self.input_filesystem.clone(),
        ));
        self.resolver_factory = Some(resolver_factory.clone());
        resolver_factory
      }
    }
  }

  pub fn get_loader_resolver_factory(&mut self, resolve_options: Resolve) -> Arc<ResolverFactory> {
    match &self.loader_resolver_factory {
      Some(resolver_factory) => resolver_factory.clone(),
      None => {
        let resolver_factory = Arc::new(ResolverFactory::new(
          resolve_options,
          self.input_filesystem.clone(),
        ));
        self.loader_resolver_factory = Some(resolver_factory.clone());
        resolver_factory
      }
    }
  }

  #[napi(ts_args_type = "type: string, options?: RawResolveOptionsWithDependencyType")]
  pub fn get(
    &mut self,
    r#type: String,
    raw: Option<RawResolveOptionsWithDependencyType>,
  ) -> napi::Result<JsResolver> {
    match r#type.as_str() {
      "normal" => {
        let options = normalize_raw_resolve_options_with_dependency_type(raw, false).map_err(|e| napi::Error::from_reason(format!("{e}")))?;
        let resolver_factory = self.get_resolver_factory(*options.resolve_options.clone().unwrap_or_default());
        Ok(JsResolver::new(resolver_factory, options))
      }
      "loader" => {
        let options = normalize_raw_resolve_options_with_dependency_type(raw, false).map_err(|e| napi::Error::from_reason(format!("{e}")))?;
        let resolver_factory = self.get_loader_resolver_factory(*options.resolve_options.clone().unwrap_or_default());
        Ok(JsResolver::new(resolver_factory, options))
      }
      "context" => {
        let options = normalize_raw_resolve_options_with_dependency_type(raw, true).map_err(|e| napi::Error::from_reason(format!("{e}")))?;
        let resolver_factory = self.get_resolver_factory(*options.resolve_options.clone().unwrap_or_default());
        Ok(JsResolver::new(resolver_factory, options))
      }
      _ => {
        Err(napi::Error::from_reason(format!("Invalid resolver type '{type}' specified. Rspack only supports 'normal', 'context', and 'loader' types.")))
      }
    }
  }
}
