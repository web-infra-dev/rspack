use std::sync::Arc;

use napi_derive::napi;
use rspack_core::ResolverFactory;

use crate::{
  raw_resolve::{
    normalize_raw_resolve_options_with_dependency_type, RawResolveOptionsWithDependencyType,
  },
  JsResolver,
};

#[napi]
pub struct JsResolverFactory {
  resolver_factory: Arc<ResolverFactory>,
  loader_resolver_factory: Arc<ResolverFactory>,
}

#[napi]
impl JsResolverFactory {
  pub fn new(
    resolver_factory: Arc<ResolverFactory>,
    loader_resolver_factory: Arc<ResolverFactory>,
  ) -> Self {
    Self {
      resolver_factory,
      loader_resolver_factory,
    }
  }

  #[napi(ts_args_type = "type: string, options?: RawResolveOptionsWithDependencyType")]
  pub fn get(
    &self,
    r#type: String,
    raw: Option<RawResolveOptionsWithDependencyType>,
  ) -> JsResolver {
    match r#type.as_str() {
      "normal" => {
        let options = normalize_raw_resolve_options_with_dependency_type(raw, false).unwrap();
        JsResolver::new(self.resolver_factory.clone(), options)
      }
      "loader" => {
        let options = normalize_raw_resolve_options_with_dependency_type(raw, false).unwrap();
        JsResolver::new(self.loader_resolver_factory.clone(), options)
      }
      "context" => {
        let options = normalize_raw_resolve_options_with_dependency_type(raw, true).unwrap();
        JsResolver::new(self.resolver_factory.clone(), options)
      }
      _ => {
        panic!("Invalid resolver type '{}' specified. Rspack only supports 'normal', 'context', and 'loader' types.", r#type)
      }
    }
  }
}
