use std::{path::Path, sync::Arc};

use napi::Either;
use napi_derive::napi;
use rspack_core::{ResolveOptionsWithDependencyType, Resolver, ResolverFactory, ResourceData};

use crate::{
  raw_resolve::{
    normalize_raw_resolve_options_with_dependency_type, RawResolveOptionsWithDependencyType,
  },
  JsResourceData,
};

#[napi]
#[derive(Debug)]
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
  #[napi(ts_return_type = "Promise<JsResourceData | false>")]
  pub async fn resolve(
    &self,
    path: String,
    request: String,
  ) -> napi::Result<Either<JsResourceData, bool>> {
    match self.resolver.resolve(Path::new(&path), &request).await {
      Ok(rspack_core::ResolveResult::Resource(resource)) => {
        Ok(Either::A(ResourceData::from(resource).into()))
      }
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
            Some(base_resolve_options) => Some(Box::new(
              base_resolve_options.clone().merge(*resolve_options),
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
