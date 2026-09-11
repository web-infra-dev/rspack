use std::{borrow::Cow, sync::Arc};

use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsOption, AsRefStr},
};
use rspack_collections::Identifier;
use rspack_core::{
  BoxLoader, Context, Loader, ModuleRuleUseLoader, NormalModuleFactoryResolveLoader, ResolveResult,
  Resolver, Resource, RunnerContext,
};
use rspack_error::Result;
use rspack_hook::plugin_hook;
use rspack_paths::Utf8Path;
use rspack_util::identifier::split_at_query_mark;

use super::{JsLoaderRspackPlugin, JsLoaderRspackPluginInner, cache::loader_cache_version};

#[cacheable]
#[derive(Debug)]
pub struct JsLoader(
  pub Identifier,
  /* LoaderType */ #[cacheable(with=AsOption<AsRefStr>)] pub Option<Cow<'static, str>>,
  pub Option<String>,
);

#[cacheable_dyn]
impl Loader<RunnerContext> for JsLoader {
  fn identifier(&self) -> Identifier {
    self.0
  }

  fn r#type(&self) -> Option<&str> {
    self.1.as_deref()
  }

  fn cache_version(&self) -> Option<&str> {
    self.2.as_deref()
  }
}

#[cfg(feature = "test-loader")]
pub fn get_builtin_test_loader(builtin: &str) -> Option<BoxLoader> {
  if builtin.starts_with(rspack_loader_testing::SIMPLE_ASYNC_LOADER_IDENTIFIER) {
    return Some(Arc::new(rspack_loader_testing::SimpleAsyncLoader));
  }
  if builtin.starts_with(rspack_loader_testing::SIMPLE_LOADER_IDENTIFIER) {
    return Some(Arc::new(rspack_loader_testing::SimpleLoader));
  }
  if builtin.starts_with(rspack_loader_testing::PITCHING_LOADER_IDENTIFIER) {
    return Some(Arc::new(rspack_loader_testing::PitchingLoader));
  }
  if builtin.starts_with(rspack_loader_testing::PASS_THROUGH_LOADER_IDENTIFIER) {
    return Some(Arc::new(rspack_loader_testing::PassthroughLoader));
  }
  if builtin.starts_with(rspack_loader_testing::NO_PASS_THROUGH_LOADER_IDENTIFIER) {
    return Some(Arc::new(rspack_loader_testing::NoPassthroughLoader));
  }
  if builtin.starts_with(rspack_loader_testing::DEPENDENCY_LOADER_IDENTIFIER) {
    return Some(Arc::new(rspack_loader_testing::DependencyLoader));
  }
  None
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
  let (loader_path, rest) = split_at_query_mark(loader_request);
  let prev = Utf8Path::new(loader_path);
  #[cfg(feature = "test-loader")]
  if loader_request.starts_with("builtin:test") {
    return Ok(get_builtin_test_loader(loader_request));
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

      let cache_version = if l.cache {
        loader_cache_version(resolver, Utf8Path::new(path)).await?
      } else {
        None
      };

      let r#type = if path.ends_with(".mjs") {
        Some(Cow::Borrowed("module"))
      } else if path.ends_with(".cjs") {
        Some(Cow::Borrowed("commonjs"))
      } else {
        description_data.as_ref().and_then(|data| {
          data
            .json()
            .get("type")
            .and_then(|t| t.as_str().map(|t| Cow::Owned(t.to_owned())))
        })
      };
      #[cfg(windows)]
      let path = {
        // Node's module loader may interpret a namespaced drive path as only
        // its drive component. Filesystem access and cache versioning above
        // still use the original namespaced path.
        rspack_paths::strip_dos_device_path_prefix(path)
      };

      // favor explicit loader query over aliased query, see webpack issue-3320
      let resource = if let Some(rest) = rest
        && !rest.is_empty()
      {
        format!("{path}{rest}")
      } else {
        format!("{path}{query}")
      };
      Ok(Some(Arc::new(JsLoader(
        resource.into(),
        r#type,
        cache_version,
      ))))
    }
    ResolveResult::Ignored => Ok(None),
  }
}
