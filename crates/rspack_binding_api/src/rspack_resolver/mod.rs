//! Port rspack_resolver napi
//!
//! This module is generally copied from https://github.com/web-infra-dev/rspack-resolver/blob/main/napi/src/lib.rs

use std::path::{Path, PathBuf};

use napi_derive::napi;
use rspack_resolver::{ResolveOptions, Resolver};

async fn resolve(resolver: &Resolver, path: &Path, request: &str) -> Option<String> {
  resolver
    .resolve(path, request)
    .await
    .map(|resolution| resolution.full_path().to_string_lossy().to_string())
    .ok()
}

#[napi(js_name = "async")]
pub async fn async_(path: String, request: String) -> Option<String> {
  let path = PathBuf::from(path);
  let resolver = Resolver::new(ResolveOptions::default());
  resolve(&resolver, &path, &request).await
}
