use std::{path::Path, sync::Arc};

use napi::{Either, bindgen_prelude::Function};
use napi_derive::napi;
use rspack_core::Resolver;
use serde::Serialize;

use crate::{error::ErrorCode, utils::callbackify};

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

pub(crate) fn javascript_module_type(
  path: &str,
  description: Option<&serde_json::Value>,
) -> Option<String> {
  if path.ends_with(".mjs") {
    Some("module".to_owned())
  } else if path.ends_with(".cjs") {
    Some("commonjs".to_owned())
  } else {
    description
      .and_then(|data| data.get("type"))
      .and_then(|value| value.as_str())
      .map(str::to_owned)
  }
}

#[napi(object)]
pub struct JsResolvedModule {
  pub path: String,
  pub r#type: Option<String>,
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
  /// Resolve a JavaScript module without loader query, pitch, raw or builtin semantics.
  #[napi]
  pub fn resolve_module_sync(
    &self,
    path: String,
    request: String,
  ) -> napi::Result<JsResolvedModule> {
    #[allow(clippy::disallowed_methods)]
    rspack_napi::runtime::block_on(async {
      match self.resolver.resolve(Path::new(&path), &request).await {
        Ok(rspack_core::ResolveResult::Resource(resource)) => {
          if !resource.query.is_empty() || !resource.fragment.is_empty() {
            return Err(napi::Error::from_reason(
              "workerFunction target must not contain a query or fragment",
            ));
          }
          Ok(JsResolvedModule {
            r#type: javascript_module_type(
              resource.path.as_str(),
              resource.description_data.as_ref().map(|data| data.json()),
            ),
            path: resource.path.to_string(),
          })
        }
        Ok(rspack_core::ResolveResult::Ignored) => Err(napi::Error::from_reason(
          "workerFunction target was ignored by resolveLoader",
        )),
        Err(error) => Err(napi::Error::from_reason(format!(
          "Cannot resolve workerFunction {request}: {error:?}"
        ))),
      }
    })
  }

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
        match resolve_result {
          Ok(rspack_core::ResolveResult::Resource(resource)) => {
            let mut resolve_request = ResolveRequest::from(resource);
            resolve_request.file_dependencies = resolve_dependencies
              .file_dependencies
              .drain()
              .map(|path| path.to_string_lossy().into_owned())
              .collect();
            resolve_request.missing_dependencies = resolve_dependencies
              .missing_dependencies
              .drain()
              .map(|path| path.to_string_lossy().into_owned())
              .collect();
            Ok(match simd_json::to_string(&resolve_request) {
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
}
