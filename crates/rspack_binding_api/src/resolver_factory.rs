use std::sync::Arc;

use napi_derive::napi;
use rspack_core::{Resolve, ResolverFactory};
use rspack_fs::{NativeFileSystem, ReadableFileSystem};

use crate::{
  error::RspackResultToNapiResultExt,
  options::raw_resolve::{
    RawResolveOptions, RawResolveOptionsWithDependencyType,
    normalize_raw_resolve_options_with_dependency_type,
  },
  resolver::JsResolver,
};

#[napi]
pub struct JsResolverFactory {
  input_filesystem: Arc<dyn ReadableFileSystem>,
  resolve_options: Resolve,
  loader_resolve_options: Resolve,

  resolver_factory: Option<Arc<ResolverFactory>>,
  loader_resolver_factory: Option<Arc<ResolverFactory>>,
}

#[napi]
impl JsResolverFactory {
  #[napi(constructor)]
  pub fn new(
    pnp: bool,
    js_resolve_options: RawResolveOptions,
    js_loader_resolve_options: RawResolveOptions,
  ) -> napi::Result<Self> {
    // Webpack handles resolver creation in lib/WebpackOptionsApply.js via resolveOptions hook:
    // 1. For "normal" type: merges compiler.options.resolve and sets inputFileSystem
    // 2. For "loader" type: merges compiler.options.resolveLoader and sets inputFileSystem
    // 3. For "context" type: merges compiler.options.resolve and sets inputFileSystem
    //
    // Since Rspack doesn't support resolveOptions hook, we pre-configure resolve/resolveLoader
    // options and inputFileSystem during ResolverFactory creation for later resolver instantiation.
    let input_filesystem = Arc::new(NativeFileSystem::new(pnp));

    let resolve_options: Resolve = js_resolve_options
      .try_into()
      .map_err(|e: rspack_error::Error| napi::Error::from_reason(e.to_string()))?;

    let loader_resolve_options: Resolve = js_loader_resolve_options
      .try_into()
      .map_err(|e: rspack_error::Error| napi::Error::from_reason(e.to_string()))?;

    Ok(Self {
      resolve_options,
      loader_resolve_options,
      resolver_factory: None,
      loader_resolver_factory: None,
      input_filesystem,
    })
  }

  pub fn update_options(
    &mut self,
    input_filesystem: Option<Arc<dyn ReadableFileSystem>>,
    resolve_options: Resolve,
    loader_resolve_options: Resolve,
  ) {
    if input_filesystem.is_some()
      || self.resolve_options != resolve_options
      || self.loader_resolve_options != loader_resolve_options
    {
      self.resolver_factory = None;
      self.loader_resolver_factory = None;
    }
    if let Some(input_filesystem) = input_filesystem {
      self.input_filesystem = input_filesystem;
    }
    self.resolve_options = resolve_options;
    self.loader_resolve_options = loader_resolve_options;
  }

  pub fn get_resolver_factory(&mut self) -> Arc<ResolverFactory> {
    match &self.resolver_factory {
      Some(resolver_factory) => resolver_factory.clone(),

      None => {
        let resolver_factory = Arc::new(ResolverFactory::new(
          self.resolve_options.clone(),
          self.input_filesystem.clone(),
        ));
        self.resolver_factory = Some(resolver_factory.clone());
        resolver_factory
      }
    }
  }

  pub fn get_loader_resolver_factory(&mut self) -> Arc<ResolverFactory> {
    match &self.loader_resolver_factory {
      Some(resolver_factory) => resolver_factory.clone(),
      None => {
        let resolver_factory = Arc::new(ResolverFactory::new(
          self.loader_resolve_options.clone(),
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
        let resolver_factory = self.get_resolver_factory();
        let options =
          normalize_raw_resolve_options_with_dependency_type(raw, false).to_napi_result()?;
        let resolver = resolver_factory.get(options);
        Ok(JsResolver::new(resolver))
      }
      "loader" => {
        let resolver_factory = self.get_loader_resolver_factory();
        let options =
          normalize_raw_resolve_options_with_dependency_type(raw, false).to_napi_result()?;
        let resolver = resolver_factory.get(options);
        Ok(JsResolver::new(resolver))
      }
      "context" => {
        let resolver_factory = self.get_resolver_factory();
        let options =
          normalize_raw_resolve_options_with_dependency_type(raw, true).to_napi_result()?;
        let resolver = resolver_factory.get(options);
        Ok(JsResolver::new(resolver))
      }
      _ => Err(napi::Error::from_reason(format!(
        "Invalid resolver type '{type}' specified. Rspack only supports 'normal', 'context', and 'loader' types."
      ))),
    }
  }
}
