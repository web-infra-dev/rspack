//! Port rspack_resolver napi
//!
//! This module is generally copied from https://github.com/web-infra-dev/rspack-resolver/blob/main/napi/src/lib.rs

use std::path::{Path, PathBuf};

use napi_derive::napi;
use rspack_resolver::{ResolveOptions, Resolver};

#[napi(object)]
pub struct ResolveResult {
  pub path: Option<String>,
  pub error: Option<String>,
  /// "type" field in the package.json file
  pub module_type: Option<String>,
}

async fn resolve(resolver: &Resolver, path: &Path, request: &str) -> ResolveResult {
  match resolver.resolve(path, request).await {
    Ok(resolution) => ResolveResult {
      path: Some(resolution.full_path().to_string_lossy().to_string()),
      error: None,
      module_type: resolution
        .package_json()
        .and_then(|p| p.r#type.as_ref())
        .and_then(|t| t.as_str())
        .map(|t| t.to_string()),
    },
    Err(err) => ResolveResult {
      path: None,
      module_type: None,
      error: Some(err.to_string()),
    },
  }
}

#[napi(js_name = "async")]
pub async fn async_(path: String, request: String) -> ResolveResult {
  let path = PathBuf::from(path);
  let resolver = Resolver::new(ResolveOptions::default());
  resolve(&resolver, &path, &request).await
}
