use std::{any::Any, borrow::Cow, sync::Arc};

use concat_string::concat_string;
use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsOption, AsRefStr},
};
use rspack_collections::Identifier;
use rspack_core::{
  BoxLoader, Context, Loader, LoaderExecutionKind, ModuleFactoryCreateData, ModuleRuleUseLoader,
  NormalModuleFactoryResolveLoader, ResolveResult, Resolver, Resource, RunnerContext,
};
use rspack_error::Result;
use rspack_hook::plugin_hook;
use rspack_napi::ThreadsafeValueRef;
use rspack_paths::{InternedPath, Utf8Path};
use rspack_util::identifier::split_at_query_mark;

use super::{JsLoaderRspackPlugin, JsLoaderRspackPluginInner};

#[cacheable]
#[derive(Debug)]
pub struct JsLoader {
  pub identifier: Identifier,
  #[cacheable(with=AsOption<AsRefStr>)]
  pub r#type: Option<Cow<'static, str>>,
  pub build_dependencies: Vec<InternedPath>,
  #[cacheable(with=rspack_cacheable::with::Skip)]
  pub js_options: Option<ThreadsafeValueRef>,
}

#[cacheable_dyn]
impl Loader<RunnerContext> for JsLoader {
  fn identifier(&self) -> Identifier {
    self.identifier
  }

  fn r#type(&self) -> Option<&str> {
    self.r#type.as_deref()
  }

  fn build_dependencies(&self) -> &[InternedPath] {
    &self.build_dependencies
  }

  fn execution_kind(&self) -> LoaderExecutionKind {
    LoaderExecutionKind::JavaScript
  }

  fn js_options(&self) -> Option<&(dyn Any + Send + Sync)> {
    self
      .js_options
      .as_ref()
      .map(|v| v as &(dyn Any + Send + Sync))
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
  data: &mut ModuleFactoryCreateData,
) -> Result<Option<BoxLoader>> {
  let context = context.as_path();
  let loader_request = &l.loader;
  let (loader_path, rest) = split_at_query_mark(loader_request);
  let prev = Utf8Path::new(loader_path);
  #[cfg(feature = "test-loader")]
  if loader_request.starts_with("builtin:test") {
    return Ok(get_builtin_test_loader(loader_request));
  }

  let (result, dependencies) = resolver
    .resolve_with_context(context.as_std_path(), prev.as_str())
    .await;
  data
    .file_dependencies
    .extend(dependencies.file_dependencies);
  data
    .missing_dependencies
    .extend(dependencies.missing_dependencies);
  let Ok(resolve_result) = result else {
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

      let build_dependencies = if l.cache() {
        vec![InternedPath::from(Utf8Path::new(path).as_std_path())]
      } else {
        Vec::new()
      };

      let r#type = if path.ends_with(".mjs") {
        Some(Cow::Borrowed("module"))
      } else if path.ends_with(".cjs") {
        Some(Cow::Borrowed("commonjs"))
      } else {
        description_data.as_ref().and_then(|data| {
          data
            .get(["type"])
            .and_then(|t| t.as_str().map(|t| Cow::Owned(t.to_owned())))
        })
      };
      #[cfg(windows)]
      let stripped_path = {
        // Node's module loader may interpret a namespaced drive path as only
        // its drive component. Filesystem access and cache versioning above
        // still use the original namespaced path.
        rspack_paths::strip_dos_device_path_prefix(path)
      };
      #[cfg(windows)]
      let path = stripped_path.as_ref();

      // favor explicit loader query over aliased query, see webpack issue-3320
      let identifier = if let Some(rest) = rest
        && !rest.is_empty()
      {
        concat_string!(path, rest).into()
      } else if !query.is_empty() {
        concat_string!(path, query).into()
      } else if let Some(ident) = &l.ident {
        concat_string!(path, "??", ident.as_str()).into()
      } else {
        path.into()
      };
      let js_options = l
        .js_options
        .as_deref()
        .and_then(|v| v.downcast_ref::<ThreadsafeValueRef>())
        .cloned();
      Ok(Some(Arc::new(JsLoader {
        identifier,
        r#type,
        build_dependencies,
        js_options,
      })))
    }
    ResolveResult::Ignored => Ok(None),
  }
}
