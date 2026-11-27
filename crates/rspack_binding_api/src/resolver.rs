use std::{path::Path, sync::Arc};

use napi::{Either, bindgen_prelude::Function};
use napi_derive::napi;
use rspack_core::Resolver;
use serde::Serialize;

use crate::{error::ErrorCode, utils::callbackify};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ResolveResponse {
  #[serde(skip_serializing_if = "Option::is_none")]
  error: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  request: Option<ResolveRequest>,
  file_dependencies: Vec<String>,
  missing_dependencies: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveRequest {
  pub path: String,
  pub query: String,
  pub fragment: String,
  pub description_file_data: Option<serde_json::Value>,
  pub description_file_path: Option<String>,
  pub file_dependencies: Vec<String>,
  pub missing_dependencies: Vec<String>,
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
      file_dependencies: vec![],
      missing_dependencies: vec![],
    }
  }
}

#[napi]
#[derive(Debug)]
pub struct JsResolver {
  resolver: Arc<Resolver>,
}

impl JsResolver {
  pub fn new(resolver: Arc<Resolver>) -> Self {
    Self { resolver }
  }
}
#[napi]
impl JsResolver {
  #[napi]
  pub fn resolve_sync(&self, path: String, request: String) -> napi::Result<Either<String, ()>> {
    #[allow(clippy::disallowed_methods)]
    rspack_napi::runtime::block_on(async {
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
        let (resolve_result, mut resolve_dependencies) = resolver
          .resolve_with_context(Path::new(&path), &request)
          .await;
        let file_dependencies: Vec<String> = resolve_dependencies
          .file_dependencies
          .drain()
          .map(|path| path.to_string_lossy().into_owned())
          .collect();
        let missing_dependencies: Vec<String> = resolve_dependencies
          .missing_dependencies
          .drain()
          .map(|path| path.to_string_lossy().into_owned())
          .collect();
        let response = match resolve_result {
          Ok(rspack_core::ResolveResult::Resource(resource)) => ResolveResponse {
            error: None,
            request: Some(ResolveRequest::from(resource)),
            file_dependencies,
            missing_dependencies,
          },
          Ok(rspack_core::ResolveResult::Ignored) => ResolveResponse {
            error: None,
            request: None,
            file_dependencies,
            missing_dependencies,
          },
          Err(err) => ResolveResponse {
            error: Some(format!("{err:?}")),
            request: None,
            file_dependencies,
            missing_dependencies,
          },
        };
        simd_json::to_string(&response).map_err(|err| {
          napi::Error::new(
            ErrorCode::Napi(napi::Status::GenericFailure),
            err.to_string(),
          )
          .into()
        })
      },
      None::<fn(&napi::Env, &mut _)>,
    )
  }
}
