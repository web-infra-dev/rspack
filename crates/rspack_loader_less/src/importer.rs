use std::{io, path::Path, sync::Arc};

use cow_utils::CowUtils;
use parking_lot::Mutex;
use rspack_core::{
  DependencyCategory, LoaderContext, Resolve, ResolveOptionsWithDependencyType, ResolveResult,
  Resolver, RunnerContext,
};
use rspack_loader_runner::LoaderDependencies;
use rspack_paths::Utf8PathBuf;

pub(crate) struct Importer {
  resolver: Arc<Resolver>,
  package_resolver: Arc<Resolver>,
  dependencies: Arc<Mutex<LoaderDependencies>>,
  context: Utf8PathBuf,
  runtime: tokio::runtime::Handle,
  only: bool,
}

impl Importer {
  pub fn new(
    context: &LoaderContext<RunnerContext>,
    dependencies: Arc<Mutex<LoaderDependencies>>,
    only: bool,
  ) -> Self {
    let get_resolver = |prefer_relative| {
      context
        .context
        .resolver_factory
        .get(ResolveOptionsWithDependencyType {
          resolve_options: Some(Box::new(Resolve {
            condition_names: Some(vec!["less".into(), "style".into(), "...".into()]),
            main_fields: Some(vec![
              "less".into(),
              "style".into(),
              "main".into(),
              "...".into(),
            ]),
            main_files: Some(vec!["index".into(), "...".into()]),
            extensions: Some(vec![".less".into(), ".css".into()]),
            prefer_relative: Some(prefer_relative),
            ..Default::default()
          })),
          resolve_to_context: false,
          dependency_category: DependencyCategory::Less,
        })
    };
    Self {
      resolver: get_resolver(true),
      package_resolver: get_resolver(false),
      dependencies,
      context: context.context.options.context.as_str().into(),
      runtime: tokio::runtime::Handle::current(),
      only,
    }
  }

  fn resolve_request(
    &self,
    resolver: &Resolver,
    directory: &Path,
    request: &str,
  ) -> io::Result<String> {
    // The Less compiler runs on Rayon, outside Tokio. Drive the asynchronous
    // resolver here without blocking a Tokio worker or holding dependency locks.
    let (result, dependencies) = self
      .runtime
      .block_on(resolver.resolve_with_context(directory, request));
    {
      let mut target = self.dependencies.lock();
      target.file.extend(dependencies.file_dependencies);
      target.missing.extend(dependencies.missing_dependencies);
    }
    match result {
      Ok(ResolveResult::Resource(resource)) => {
        Ok(resource.path.as_str().cow_replace('\\', "/").into_owned())
      }
      Ok(ResolveResult::Ignored) => Err(io::Error::other(format!(
        "Rspack resolver error: import '{request}' is ignored"
      ))),
      Err(error) => Err(io::Error::other(format!(
        "Rspack resolver error for '{request}' in '{}': {error}",
        directory.display()
      ))),
    }
  }
}

impl less_rs::ImportResolver for Importer {
  fn use_less_resolver(&self, filename: &str) -> bool {
    // less-loader skips native lookup for ~package so a local ~package.less
    // cannot shadow the package directory's entry point.
    !self.only && !(filename.starts_with('~') && filename.len() > 1 && !filename.contains('/'))
  }

  fn resolve(&self, filename: &str, current_directory: &str) -> io::Result<String> {
    let directory = self.context.join(current_directory);
    // Match less-loader's /^[^?]*~/ replacement, leaving query strings alone.
    let tilde = filename.split('?').next().and_then(|s| s.rfind('~'));
    let request = tilde.map_or(filename, |index| &filename[index + 1..]);
    let is_module = filename.strip_prefix('~').is_some_and(|name| {
      let name = name.strip_suffix('/').unwrap_or(name);
      !name.is_empty()
        && (!name.contains('/') || (name.starts_with('@') && name.matches('/').count() == 1))
    });
    let request = if is_module && !request.ends_with('/') {
      format!("{request}/")
    } else {
      request.to_string()
    };
    match self.resolve_request(&self.resolver, directory.as_std_path(), &request) {
      Ok(path) => Ok(path),
      Err(mut error) => {
        // Rspack's resolver treats a trailing slash as a directory request.
        // Retry the package root to permit an exports["."] entry as well,
        // without letting a same-named local Less file shadow the package.
        if is_module && let Some(root) = request.strip_suffix('/') {
          match self.resolve_request(&self.package_resolver, directory.as_std_path(), root) {
            Ok(path) => return Ok(path),
            Err(next) => error = next,
          }
        }
        if request != filename {
          return self
            .resolve_request(&self.resolver, directory.as_std_path(), filename)
            .map_err(|_| error);
        }
        Err(error)
      }
    }
  }
}
