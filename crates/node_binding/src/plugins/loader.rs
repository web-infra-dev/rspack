use std::{fmt::Debug, path::Path, sync::Arc};

use rspack_binding_options::{JsLoaderAdapter, JsLoaderRunner};
use rspack_core::{
  resolve, BoxLoader, CompilerOptions, DependencyCategory, DependencyType, NormalModule, Plugin,
  ResolveArgs, ResolveResult, Resolver, ResolverFactory,
};
use rspack_error::{internal_error, Result};

pub struct InlineLoaderResolver {
  pub js_loader_runner: JsLoaderRunner,
}

impl Debug for InlineLoaderResolver {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("InlineLoaderResolver")
      .field("js_loader_runner", &"..")
      .finish()
  }
}

#[async_trait::async_trait]
impl Plugin for InlineLoaderResolver {
  async fn resolve_inline_loader(
    &self,
    compiler_options: &CompilerOptions,
    context: &Path,
    resolver: &Resolver,
    loader_request: &str,
  ) -> Result<Option<BoxLoader>> {
    if loader_request.starts_with("builtin:") {
      // builtin loaders are not supported.
      // TODO: Options have to be serializable.
      return Ok(None);
    }

    let resolve_result = resolver.resolve(context, loader_request).map_err(|err| {
      let context = context.display();
      internal_error!("Failed to resolve loader: {loader_request} in {context} {err:?}")
    })?;

    match resolve_result {
      ResolveResult::Resource(resource) => {
        let resource = resource.join().display().to_string();
        Ok(Some(Arc::new(JsLoaderAdapter {
          identifier: resource.into(),
          runner: self.js_loader_runner.clone(),
        })))
      }
      ResolveResult::Ignored => Err(internal_error!(
        "Failed to resolve loader: {loader_request}"
      )),
    }
  }
}
