use std::{path::Path, sync::Arc};

use napi::{
  Either,
  bindgen_prelude::{Function, block_on},
};
use napi_derive::napi;
use rspack_core::{ResolveOptionsWithDependencyType, Resolver, ResolverFactory};
use serde::Serialize;

use crate::{
  ErrorCode, callbackify,
  raw_resolve::{
    RawResolveOptionsWithDependencyType, normalize_raw_resolve_options_with_dependency_type,
  },
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveRequest {
  pub path: String,
  pub query: String,
  pub fragment: String,
  pub description_file_data: Option<serde_json::Value>,
  pub description_file_path: Option<String>,
}

impl From<rspack_core::Resource> for ResolveRequest {
  fn from(value: rspack_core::Resource) -> Self {
    let (description_file_path, description_file_data) =
      value.description_data.map(|data| data.into_parts()).unzip();
    Self {
      path: value.path.to_string(),
      query: value.query,
      fragment: value.fragment,
      description_file_data: description_file_data.map(std::sync::Arc::unwrap_or_clone),
      description_file_path: description_file_path.map(|path| path.to_string_lossy().into_owned()),
    }
  }
}

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
  #[napi]
  pub fn resolve_sync(&self, path: String, request: String) -> napi::Result<Either<String, ()>> {
    block_on(async {
      match self.resolver.resolve(Path::new(&path), &request).await {
        Ok(rspack_core::ResolveResult::Resource(resource)) => Ok(Either::A(resource.full_path())),
        Ok(rspack_core::ResolveResult::Ignored) => Ok(Either::B(())),
        Err(err) => Err(napi::Error::from_reason(format!("{err:?}"))),
      }
    })
  }

  #[napi(
    ts_args_type = "path: string, request: string, callback: (err: null | Error, req?: string) => void"
  )]
  pub fn resolve(
    &self,
    path: String,
    request: String,
    f: Function<'static>,
  ) -> napi::Result<(), ErrorCode> {
    let resolver = self.resolver.clone();
    callbackify(
      f,
      async move {
        match resolver.resolve(Path::new(&path), &request).await {
          Ok(rspack_core::ResolveResult::Resource(resource)) => {
            let resolve_request = ResolveRequest::from(resource);
            Ok(match serde_json::to_string(&resolve_request) {
              Ok(json) => Either::<String, ()>::A(json),
              Err(_) => Either::B(()),
            })
          }
          Ok(rspack_core::ResolveResult::Ignored) => Ok(Either::B(())),
          Err(err) => Err(napi::Error::new(
            ErrorCode::Napi(napi::Status::GenericFailure),
            format!("{err:?}"),
          )),
        }
      },
      None::<fn()>,
    )
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
