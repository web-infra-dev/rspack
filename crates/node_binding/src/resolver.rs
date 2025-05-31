use std::{path::Path, sync::Arc};

use napi::{
  bindgen_prelude::{block_on, FnArgs, Function},
  Either, Env, JsString,
};
use napi_derive::napi;
use rspack_core::{ResolveOptionsWithDependencyType, Resolver, ResolverFactory};

use crate::{
  callbackify,
  raw_resolve::{
    normalize_raw_resolve_options_with_dependency_type, RawResolveOptionsWithDependencyType,
  },
  ErrorCode,
};

#[napi]
pub struct ResolveRequest {
  i: rspack_core::Resource,
}

#[napi]
impl ResolveRequest {
  #[napi(getter)]
  pub fn path(&self) -> &str {
    self.i.path.as_str()
  }

  #[napi(getter)]
  pub fn query(&self) -> &str {
    &self.i.query
  }

  #[napi(getter)]
  pub fn fragment(&self) -> &str {
    &self.i.fragment
  }

  #[napi(getter, ts_return_type = "any")]
  pub fn description_file_data(&self) -> Option<&serde_json::Value> {
    self.i.description_data.as_ref().map(|desc| desc.json())
  }

  #[napi(getter, ts_return_type = "string")]
  pub fn description_file_path<'a>(&self, env: &'a Env) -> napi::Result<Option<JsString<'a>>> {
    self
      .i
      .description_data
      .as_ref()
      .map(|desc| {
        let path = desc.path().to_string_lossy();
        env.create_string(path.as_ref())
      })
      .transpose()
  }
}

impl From<rspack_core::Resource> for ResolveRequest {
  fn from(value: rspack_core::Resource) -> Self {
    Self { i: value }
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
  #[napi(ts_return_type = "string | false")]
  pub fn resolve_sync(&self, path: String, request: String) -> napi::Result<Either<String, bool>> {
    block_on(async move {
      match self.resolver.resolve(Path::new(&path), &request).await {
        Ok(rspack_core::ResolveResult::Resource(resource)) => {
          Ok(Either::A(resource.path.into_string()))
        }
        Ok(rspack_core::ResolveResult::Ignored) => Ok(Either::B(false)),
        Err(err) => Err(napi::Error::from_reason(format!("{:?}", err))),
      }
    })
  }

  #[napi(
    ts_args_type = "path: string, request: string, callback: (err: null | Error, res?: string | false, req?: ResolveRequest) => void"
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
          Ok(rspack_core::ResolveResult::Resource(resource)) => Ok(FnArgs::from((
            Either::<String, bool>::A(resource.path.to_string()),
            Either::<ResolveRequest, ()>::A(ResolveRequest::from(resource)),
          ))),
          Ok(rspack_core::ResolveResult::Ignored) => {
            Ok(FnArgs::from((Either::B(false), Either::B(()))))
          }
          Err(err) => Err(napi::Error::from_reason(format!("{:?}", err))),
        }
      },
      || {},
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
