use std::sync::Arc;

use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_collections::{Identifiable, Identifier};
use rspack_core::{
  BoxLoader, Context, Loader, ModuleRuleUseLoader, NormalModuleFactoryResolveLoader, ResolveResult,
  Resolver, Resource, RunnerContext,
};
use rspack_error::Result;
use rspack_hook::plugin_hook;
use rspack_paths::Utf8Path;

use super::{JsLoaderRspackPlugin, JsLoaderRspackPluginInner};

#[cacheable]
#[derive(Debug)]
pub struct JsLoader(pub Identifier);

#[cacheable_dyn]
impl Loader<RunnerContext> for JsLoader {}

impl Identifiable for JsLoader {
  fn identifier(&self) -> Identifier {
    self.0
  }
}

// TODO: should be compiled with a different cfg
pub fn get_builtin_test_loader(builtin: &str) -> Result<BoxLoader> {
  if builtin.starts_with(rspack_loader_testing::SIMPLE_ASYNC_LOADER_IDENTIFIER) {
    return Ok(Arc::new(rspack_loader_testing::SimpleAsyncLoader));
  }
  if builtin.starts_with(rspack_loader_testing::SIMPLE_LOADER_IDENTIFIER) {
    return Ok(Arc::new(rspack_loader_testing::SimpleLoader));
  }
  if builtin.starts_with(rspack_loader_testing::PITCHING_LOADER_IDENTIFIER) {
    return Ok(Arc::new(rspack_loader_testing::PitchingLoader));
  }
  if builtin.starts_with(rspack_loader_testing::PASS_THROUGH_LOADER_IDENTIFIER) {
    return Ok(Arc::new(rspack_loader_testing::PassthroughLoader));
  }
  if builtin.starts_with(rspack_loader_testing::NO_PASS_THROUGH_LOADER_IDENTIFIER) {
    return Ok(Arc::new(rspack_loader_testing::NoPassthroughLoader));
  }
  unreachable!("Unexpected builtin test loader: {builtin}")
}

#[plugin_hook(NormalModuleFactoryResolveLoader for JsLoaderRspackPlugin,tracing=false)]
pub(crate) async fn resolve_loader(
  &self,
  context: &Context,
  resolver: &Resolver,
  l: &ModuleRuleUseLoader,
) -> Result<Option<BoxLoader>> {
  let context = context.as_path();
  let loader_request = &l.loader;
  let mut rest = None;
  let prev = if let Some(index) = loader_request.find('?') {
    rest = Some(&loader_request[index..]);
    Utf8Path::new(&loader_request[0..index])
  } else {
    Utf8Path::new(loader_request)
  };

  if loader_request.starts_with("builtin:test") {
    return Ok(get_builtin_test_loader(loader_request).ok());
  }

  let Some(resolve_result) = resolver
    .resolve(context.as_std_path(), prev.as_str())
    .await
    .ok()
  else {
    return Ok(None);
  };

  match resolve_result {
    ResolveResult::Resource(resource) => {
      let Resource {
        path,
        query,
        description_data,
        ..
      } = resource;
      // Pitfall: `Path::ends_with` is different from `str::ends_with`
      // So we need to convert `PathBuf` to `&str`
      // Use `str::ends_with` instead of `Path::extension` to avoid unnecessary allocation
      let path = path.as_str();

      let r#type = if path.ends_with(".mjs") {
        Some("module")
      } else if path.ends_with(".cjs") {
        Some("commonjs")
      } else {
        description_data
          .as_ref()
          .and_then(|data| data.json().get("type").and_then(|t| t.as_str()))
      };
      // favor explicit loader query over aliased query, see webpack issue-3320
      let resource = if let Some(rest) = rest
        && !rest.is_empty()
      {
        format!("{path}{rest}")
      } else {
        format!("{path}{query}")
      };
      let ident = if let Some(ty) = r#type {
        format!("{ty}|{resource}")
      } else {
        resource
      };
      Ok(Some(Arc::new(JsLoader(ident.into()))))
    }
    ResolveResult::Ignored => Ok(None),
  }
}
