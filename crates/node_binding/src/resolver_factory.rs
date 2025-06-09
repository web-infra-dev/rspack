use core::panic;
use std::{
  hash::{Hash, Hasher},
  sync::Arc,
};

use napi_derive::napi;
use rspack_core::{Resolve, ResolverFactory};
use rspack_fs::{NativeFileSystem, ReadableFileSystem};
use rustc_hash::FxHashMap;
use tracing::level_filters::LevelFilter;

use crate::{
  raw_resolve::{
    normalize_raw_resolve_options_with_dependency_type, RawResolveOptionsWithDependencyType,
  },
  JsResolver, RspackResultToNapiResultExt,
};

#[napi]
pub struct JsResolverFactory {
  pub(crate) cached_resolver_factories: FxHashMap<u64, Arc<ResolverFactory>>,
  pub(crate) input_filesystem: Arc<dyn ReadableFileSystem>,
}

#[napi]
impl JsResolverFactory {
  #[napi(constructor)]
  pub fn new(pnp: bool) -> napi::Result<Self> {
    let input_filesystem = Arc::new(NativeFileSystem::new(pnp));

    Ok(Self {
      cached_resolver_factories: FxHashMap::default(),
      input_filesystem,
    })
  }

  pub fn get_resolver_factory(&mut self, resolve_options: Resolve) -> Arc<ResolverFactory> {
    let mut hasher = rustc_hash::FxHasher::default();
    resolve_options.hash(&mut hasher);
    let hash = hasher.finish();

    match self.cached_resolver_factories.get(&hash) {
      Some(resolver_factory) => resolver_factory.clone(),

      None => {
        let resolver_factory = Arc::new(ResolverFactory::new(
          resolve_options,
          self.input_filesystem.clone(),
        ));
        self
          .cached_resolver_factories
          .insert(hash, resolver_factory.clone());
        resolver_factory
      }
    }
  }

  #[deprecated(note = "Use get_resolver_factory instead")]
  pub fn get_loader_resolver_factory(&mut self, resolve_options: Resolve) -> Arc<ResolverFactory> {
    self.get_resolver_factory(resolve_options)
  }

  #[napi(ts_args_type = "type: string, options?: RawResolveOptionsWithDependencyType")]
  pub fn get(
    &mut self,
    r#type: String,
    raw: Option<RawResolveOptionsWithDependencyType>,
  ) -> napi::Result<JsResolver> {
    match r#type.as_str() {
      "normal" => {
        let options = normalize_raw_resolve_options_with_dependency_type(raw, false).to_napi_result()?;
        let resolver_factory = self.get_resolver_factory(*options.resolve_options.clone().unwrap_or_default());
        Ok(JsResolver::new(resolver_factory, options))
      }
      "loader" => {
        let options = normalize_raw_resolve_options_with_dependency_type(raw, false).to_napi_result()?;
        let resolver_factory = self.get_resolver_factory(*options.resolve_options.clone().unwrap_or_default());
        Ok(JsResolver::new(resolver_factory, options))
      }
      "context" => {
        let options = normalize_raw_resolve_options_with_dependency_type(raw, true).to_napi_result()?;
        let resolver_factory = self.get_resolver_factory(*options.resolve_options.clone().unwrap_or_default());
        Ok(JsResolver::new(resolver_factory, options))
      }
      _ => {
        Err(napi::Error::from_reason(format!("Invalid resolver type '{type}' specified. Rspack only supports 'normal', 'context', and 'loader' types.")))
      }
    }
  }
}
