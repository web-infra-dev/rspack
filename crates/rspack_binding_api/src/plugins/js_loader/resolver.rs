use std::{borrow::Cow, ffi::OsStr, sync::Arc};

use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsOption, AsRefStr},
};
use rspack_collections::Identifier;
use rspack_core::{
  BoxLoader, Context, Loader, LoaderExecutionKind, ModuleFactoryCreateData, ModuleRuleUseLoader,
  NormalModuleFactoryResolveLoader, ResolveResult, Resolver, Resource, RunnerContext,
  cache::is_node_package_path,
};
use rspack_error::Result;
use rspack_hook::plugin_hook;
use rspack_loader_runner::DescriptionData;
use rspack_paths::{InternedPath, Utf8Path};
use rspack_util::identifier::split_at_query_mark;

use super::{JsLoaderRspackPlugin, JsLoaderRspackPluginInner};

async fn loader_build_dependencies(
  resolver: &Resolver,
  path: &Utf8Path,
  description_data: Option<&DescriptionData>,
) -> Vec<InternedPath> {
  let mut build_dependencies = vec![InternedPath::from(path.as_std_path())];
  if let Some(data) = description_data {
    let package_dir = data.path();
    if is_node_package_path(package_dir) {
      build_dependencies.push(InternedPath::from(package_dir.join("package.json")));
      let has_package_name = data
        .get(["name"])
        .and_then(|value| value.as_str().map(|name| !name.is_empty()))
        .unwrap_or(false);
      if !has_package_name {
        let mut current = package_dir.parent();
        while let Some(dir) = current {
          if dir.file_name() == Some(OsStr::new("node_modules")) || !is_node_package_path(dir) {
            break;
          }
          let candidate = dir.join("package.json");
          if let Some(candidate_utf8) = Utf8Path::from_path(&candidate)
            && resolver
              .inner_fs()
              .metadata(candidate_utf8)
              .await
              .is_ok_and(|metadata| !metadata.is_directory)
          {
            build_dependencies.push(InternedPath::from(candidate));
            break;
          }
          current = dir.parent();
        }
      }
    }
  }
  build_dependencies.sort_unstable_by(|a, b| a.as_path().cmp(b.as_path()));
  build_dependencies.dedup();
  build_dependencies
}

#[cacheable]
#[derive(Debug)]
pub struct JsLoader(
  pub Identifier,
  /* LoaderType */ #[cacheable(with=AsOption<AsRefStr>)] pub Option<Cow<'static, str>>,
  pub Vec<InternedPath>,
);

#[cacheable_dyn]
impl Loader<RunnerContext> for JsLoader {
  fn identifier(&self) -> Identifier {
    self.0
  }

  fn r#type(&self) -> Option<&str> {
    self.1.as_deref()
  }

  fn build_dependencies(&self) -> &[InternedPath] {
    &self.2
  }

  fn execution_kind(&self) -> LoaderExecutionKind {
    LoaderExecutionKind::JavaScript
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

      let build_dependencies = if l.cache.is_some() {
        loader_build_dependencies(resolver, Utf8Path::new(path), description_data.as_ref()).await
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
        build_dependencies,
      ))))
    }
    ResolveResult::Ignored => Ok(None),
  }
}
