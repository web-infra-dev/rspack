use std::path::Path;
use std::sync::Arc;

use napi::Either;
use napi_derive::napi;
use rspack_core::{ResolveOptionsWithDependencyType, Resolver, ResolverFactory};

use crate::raw_resolve::{
  normalize_raw_resolve_options_with_dependency_type, RawResolveOptionsWithDependencyType,
};

#[napi]
pub struct JsResolver {
  resolver_factory: Arc<ResolverFactory>,
  resolver: Arc<Resolver>,
  options: ResolveOptionsWithDependencyType,
}

impl JsResolver {
  pub fn new(
    resolver_factory: Arc<ResolverFactory>,
    options: ResolveOptionsWithDependencyType,
  ) -> Self {
    let resolver = resolver_factory.get(options.clone());
    Self {
      resolver_factory,
      resolver,
      options,
    }
  }
}

#[napi]
impl JsResolver {
  #[napi(ts_return_type = "string | false")]
  pub fn resolve_sync(&self, path: String, request: String) -> napi::Result<Either<String, bool>> {
    match self.resolver.resolve(Path::new(&path), &request) {
      Ok(rspack_core::ResolveResult::Resource(resource)) => Ok(Either::A(
        resource.full_path().to_string_lossy().to_string(),
      )),
      Ok(rspack_core::ResolveResult::Ignored) => Ok(Either::B(false)),
      Err(err) => Err(napi::Error::from_reason(format!("{:?}", err))),
    }
  }

  #[napi]
  pub fn with_options(
    &self,
    raw: Option<RawResolveOptionsWithDependencyType>,
  ) -> napi::Result<Self> {
    let options =
      normalize_raw_resolve_options_with_dependency_type(raw, self.options.resolve_to_context);
    match options {
      Ok(mut options) => {
        options.resolve_options = match options.resolve_options.take() {
          Some(resolve_options) => match &self.options.resolve_options {
            Some(origin_resolve_options) => Some(Box::new(
              resolve_options.merge(*origin_resolve_options.clone()),
            )),
            None => Some(resolve_options),
          },
          None => self.options.resolve_options.clone(),
        };

        Ok(Self::new(self.resolver_factory.clone(), options))
      }
      Err(e) => Err(napi::Error::from_reason(format!("{e}"))),
    }
  }
}
