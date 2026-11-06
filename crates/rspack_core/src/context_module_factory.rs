use std::{borrow::Cow, sync::Arc};

use cow_utils::CowUtils;
use derive_more::Debug;
use rspack_error::{Result, ToStringResultToRspackResultExt, error};
use rspack_fs::{ReadableFileSystem, WalkMatcher};
use rspack_hook::define_hook;
use rspack_loader_runner::parse_resource;
use rspack_paths::{Utf8Path, Utf8PathBuf, normalize_path_separators};
use smallvec::SmallVec;
use swc_core::common::util::take::Take;
use tracing::instrument;

pub use self::glob::{CompiledContextModuleGlobRequest, compile_context_module_glob_request};
use self::glob::{
  ContextModuleGlobMatcher, bare_context_module_glob_alias_request,
  resolve_context_module_glob_alias,
};
use crate::{
  CompilationId, ContextElementDependency, ContextMode, ContextModule, ContextModuleOptions,
  ContextModulePattern, DependencyCategory, DependencyId, DependencyRef, DependencyType, ModuleExt,
  ModuleFactory, ModuleFactoryCreateData, ModuleFactoryResult, OverrideStrict, ResolveArgs,
  ResolveContextModuleDependencies, ResolveInnerOptions, ResolveOptionsWithDependencyType,
  ResolveResult, Resolver, ResolverFactory, SharedPluginDriver, resolve,
};

mod glob;

#[derive(Debug)]
pub enum BeforeResolveResult {
  Ignored,
  Data(Box<BeforeResolveData>),
}

#[derive(Debug, Clone)]
pub struct BeforeResolveData {
  // context_info
  // resolve_options
  pub context: String,
  pub request: String,
  // assertions
  pub dependencies: Vec<DependencyRef>,
  // dependency_type
  // file_dependencies
  // missing_dependencies
  // context_dependencies
  // create_data
  // cacheable
  pub recursive: bool,
  pub pattern: ContextModulePattern,
}

pub enum AfterResolveResult {
  Ignored,
  Data(Box<AfterResolveData>),
}

#[derive(Debug, Clone)]
pub struct AfterResolveData {
  pub compilation_id: CompilationId,
  pub resource: Utf8PathBuf,
  pub context: String,
  pub dependencies: Vec<DependencyRef>,
  // layer
  // resolve_options
  // file_dependencies: HashSet<String>,
  // missing_dependencies: HashSet<String>,
  // context_dependencies: HashSet<String>,
  pub request: String,
  // mode
  pub recursive: bool,
  pub pattern: ContextModulePattern,
  // namespace_object
  // addon: String,
  // chunk_name: String,
  // include
  // exclude
  // group_options
  // type_prefix: String,
  // category: String,
  // referenced_exports
  #[debug(skip)]
  pub resolve_dependencies: ResolveContextModuleDependencies,
}

define_hook!(ContextModuleFactoryBeforeResolve: SeriesWaterfall(data: BeforeResolveResult) -> BeforeResolveResult);
define_hook!(ContextModuleFactoryAfterResolve: SeriesWaterfall(data: AfterResolveResult) -> AfterResolveResult);

#[derive(Debug, Default)]
pub struct ContextModuleFactoryHooks {
  pub before_resolve: ContextModuleFactoryBeforeResolveHook,
  pub after_resolve: ContextModuleFactoryAfterResolveHook,
}

#[derive(Debug)]
pub struct ContextModuleFactory {
  loader_resolver_factory: Arc<ResolverFactory>,
  plugin_driver: SharedPluginDriver,
  #[debug(skip)]
  resolve_dependencies: ResolveContextModuleDependencies,
}

struct ContextModuleAfterResolveInput {
  options: ContextModuleOptions,
  resolve_context: String,
}

#[async_trait::async_trait]
impl ModuleFactory for ContextModuleFactory {
  #[instrument("context_module_factory:create", skip_all)]
  async fn create(&self, data: &mut ModuleFactoryCreateData) -> Result<ModuleFactoryResult> {
    match self.before_resolve(data).await? {
      BeforeResolveResult::Ignored => return Ok(ModuleFactoryResult::default()),
      BeforeResolveResult::Data(before_resolve_result) => {
        let (factorize_result, after_resolve_input) =
          self.resolve(data, before_resolve_result).await?;
        if let Some(after_resolve_input) = after_resolve_input
          && let Some(factorize_result) = self.after_resolve(data, after_resolve_input).await?
        {
          return Ok(factorize_result);
        }

        Ok(factorize_result)
      }
    }
  }
}

impl ContextModuleFactory {
  pub fn new(
    resolver_factory: Arc<ResolverFactory>,
    loader_resolver_factory: Arc<ResolverFactory>,
    plugin_driver: SharedPluginDriver,
  ) -> Self {
    let resolve_dependencies: ResolveContextModuleDependencies = Arc::new(move |options| {
      let resolver_factory = resolver_factory.clone();
      Box::pin(async move {
        tracing::trace!("resolving context module path {}", options.resource);
        if options.resource.as_str().is_empty() {
          return Ok(vec![]);
        }

        let resolver = &resolver_factory.get(ResolveOptionsWithDependencyType {
          resolve_options: options
            .resolve_options
            .clone()
            .map(|r| Box::new(Arc::unwrap_or_clone(r))),
          resolve_to_context: false,
          dependency_category: options.context_options.category,
        });
        let mut context_element_dependencies = vec![];
        visit_dirs(
          &mut context_element_dependencies,
          &options,
          &resolver.options(),
          resolver.inner_fs(),
        )
        .await?;
        context_element_dependencies.sort_by_cached_key(|d| d.user_request.clone());

        tracing::trace!(
          "resolving dependencies for {:?}",
          context_element_dependencies
        );

        Ok(context_element_dependencies)
      })
    });

    Self {
      loader_resolver_factory,
      plugin_driver,
      resolve_dependencies,
    }
  }

  async fn before_resolve(
    &self,
    data: &mut ModuleFactoryCreateData,
  ) -> Result<BeforeResolveResult> {
    let dependency = data.dependencies[0]
      .as_context_dependency()
      .expect("should be context dependency");
    let dependency_options = dependency.options();

    let before_resolve_data = BeforeResolveData {
      context: data.context.to_string(),
      request: dependency.request().to_string(),
      recursive: dependency_options.recursive,
      pattern: dependency_options.pattern.clone(),
      dependencies: data.dependencies.clone(),
    };

    match self
      .plugin_driver
      .context_module_factory_hooks
      .before_resolve
      .call(BeforeResolveResult::Data(Box::new(before_resolve_data)))
      .await?
    {
      BeforeResolveResult::Ignored => Ok(BeforeResolveResult::Ignored),
      BeforeResolveResult::Data(mut result) => {
        // The dependencies can be modified  in the before resolve hook
        data.dependencies = result.dependencies.take();
        Ok(BeforeResolveResult::Data(result))
      }
    }
  }

  fn get_loader_resolver(&self) -> Arc<Resolver> {
    self
      .loader_resolver_factory
      .get(ResolveOptionsWithDependencyType {
        resolve_options: None,
        resolve_to_context: false,
        dependency_category: DependencyCategory::CommonJS,
      })
  }

  async fn resolve(
    &self,
    data: &mut ModuleFactoryCreateData,
    before_resolve_data: Box<BeforeResolveData>,
  ) -> Result<(ModuleFactoryResult, Option<ContextModuleAfterResolveInput>)> {
    let plugin_driver = &self.plugin_driver;
    let strict = self.global_override_strict();
    let dependency = data.dependencies[0]
      .as_context_dependency()
      .expect("should be context dependency");
    let hook_request = before_resolve_data.request.clone();
    let request = before_resolve_data.request;
    let (loader_request, specifier) = match request.rfind('!') {
      Some(idx) => {
        let mut loaders_prefix = String::new();
        let mut i = 0;

        let loaders_request = Cow::Borrowed(&request[..idx + 1]);
        while i < loaders_request.len() && loaders_request.chars().nth(i) == Some('!') {
          loaders_prefix.push('!');
          i += 1;
        }
        let loaders_request = loaders_request.as_ref()[i..]
          .trim_end_matches('!')
          .cow_replace("!!", "!");

        let loaders = if loaders_request.is_empty() {
          vec![]
        } else {
          loaders_request.split('!').collect()
        };
        let resource = request[idx + 1..].to_string();

        let mut loader_result = Vec::with_capacity(loaders.len());
        let loader_resolver = self.get_loader_resolver();
        for loader_request in loaders {
          let (result, dependencies) = loader_resolver
            .resolve_with_context(data.context.as_ref(), loader_request)
            .await;
          data
            .file_dependencies
            .extend(dependencies.file_dependencies);
          data
            .missing_dependencies
            .extend(dependencies.missing_dependencies);
          let resolve_result = result.to_rspack_result_with_message(|e| {
            format!(
              "Failed to resolve loader: {loader_request} in {} {e}",
              data.context
            )
          })?;
          match resolve_result {
            ResolveResult::Resource(resource) => {
              let resource = resource.full_path();
              loader_result.push(resource);
            }
            ResolveResult::Ignored => {
              let context = data.context.to_string();
              return Err(error!(
                "Failed to resolve loader: loader_request={loader_request}, context={context}"
              ));
            }
          }
        }
        let request = format!(
          "{}{}{}",
          loaders_prefix,
          loader_result.join("!"),
          if loader_result.is_empty() { "" } else { "!" }
        );
        (request, resource)
      }
      None => (String::new(), request),
    };

    let context = before_resolve_data.context;
    let recursive = before_resolve_data.recursive;
    let is_glob = matches!(&before_resolve_data.pattern, ContextModulePattern::Glob(_));
    let specifier = match &before_resolve_data.pattern {
      ContextModulePattern::Glob(patterns) => {
        let resolver = plugin_driver
          .resolver_factory
          .get(ResolveOptionsWithDependencyType {
            resolve_options: data
              .resolve_options
              .clone()
              .map(|r| Box::new(Arc::unwrap_or_clone(r))),
            resolve_to_context: true,
            dependency_category: *dependency.category(),
          });
        bare_context_module_glob_alias_request(patterns, &specifier, &resolver.options())
          .map(str::to_string)
          .unwrap_or(specifier)
      }
      _ => specifier,
    };
    let resolve_args = ResolveArgs {
      context: context.clone().into(),
      importer: data.issuer_identifier.as_ref(),
      issuer: data.issuer.as_deref(),
      specifier: specifier.as_str(),
      dependency_type: dependency.dependency_type(),
      dependency_category: dependency.category(),
      span: dependency.range(),
      resolve_options: data.resolve_options.clone(),
      resolve_to_context: true,
      optional: dependency.get_optional(),
    };

    let (resource_data, resolve_dependencies) = resolve(resolve_args, plugin_driver).await;
    let file_dependencies = resolve_dependencies.file_dependencies;
    let missing_dependencies = resolve_dependencies.missing_dependencies;

    let (module, context_module_options) = match resource_data {
      Ok(ResolveResult::Resource(resource)) => {
        let mut dependency_options = dependency.options().clone();
        dependency_options.request = hook_request.clone();
        dependency_options.recursive = recursive;
        dependency_options.pattern = before_resolve_data.pattern.clone();
        if !is_glob {
          dependency_options.context = context.clone();
        } else if !specifier.starts_with('.')
          && !specifier.starts_with('/')
          && let Some(request) = parse_resource(&specifier)
          && let ContextModulePattern::Glob(patterns) = &dependency_options.pattern
        {
          dependency_options.pattern =
            ContextModulePattern::Glob(resolve_context_module_glob_alias(
              patterns,
              request.path.as_str(),
              resource.path.as_str(),
              dependency_options.context.as_str(),
              dependency_options.compiler_context.as_str(),
            ));
        }

        let options = ContextModuleOptions {
          addon: loader_request.clone(),
          resource: resource.path,
          resource_query: resource.query,
          resource_fragment: resource.fragment,
          layer: data.issuer_layer.clone(),
          resolve_options: data.resolve_options.clone(),
          context_options: dependency_options,
          type_prefix: dependency.type_prefix(),
        };
        let module = ContextModule::new_with_strict(
          self.resolve_dependencies.clone(),
          options.clone(),
          strict,
        )
        .boxed();
        (module, Some(options))
      }
      Ok(ResolveResult::Ignored) => {
        // should create an empty context module when ignored
        let mut dependency_options = dependency.options().clone();
        dependency_options.request = hook_request;
        dependency_options.recursive = recursive;
        dependency_options.pattern = before_resolve_data.pattern.clone();
        if !is_glob {
          dependency_options.context = context.clone();
        }

        let options = ContextModuleOptions {
          addon: loader_request.clone(),
          resource: Default::default(),
          resource_query: Default::default(),
          resource_fragment: Default::default(),
          layer: data.issuer_layer.clone(),
          resolve_options: data.resolve_options.clone(),
          context_options: dependency_options,
          type_prefix: dependency.type_prefix(),
        };
        let module = ContextModule::new_with_strict(
          self.resolve_dependencies.clone(),
          options.clone(),
          strict,
        )
        .boxed();
        (module, Some(options))
      }
      Err(err) => {
        data.add_file_dependencies(file_dependencies);
        data.add_missing_dependencies(missing_dependencies);
        return Err(err);
      }
    };

    data.add_file_dependencies(file_dependencies);
    data.add_missing_dependencies(missing_dependencies);
    // data.add_context_dependencies(context_dependencies);

    let module_factory_result = ModuleFactoryResult {
      module: Some(module),
    };
    Ok((
      module_factory_result,
      context_module_options.map(|options| ContextModuleAfterResolveInput {
        options,
        resolve_context: context,
      }),
    ))
  }

  async fn after_resolve(
    &self,
    data: &mut ModuleFactoryCreateData,
    input: ContextModuleAfterResolveInput,
  ) -> Result<Option<ModuleFactoryResult>> {
    let ContextModuleAfterResolveInput {
      mut options,
      resolve_context,
    } = input;
    let context_module_options = &mut options;
    let context_options = &context_module_options.context_options;
    let after_resolve_data = AfterResolveData {
      compilation_id: data.build_context.compilation_id,
      resource: context_module_options.resource.clone(),
      context: resolve_context,
      dependencies: data.dependencies.clone(),
      request: context_options.request.clone(),
      pattern: context_options.pattern.clone(),
      recursive: context_options.recursive,
      resolve_dependencies: self.resolve_dependencies.clone(),
    };

    match self
      .plugin_driver
      .context_module_factory_hooks
      .after_resolve
      .call(AfterResolveResult::Data(Box::new(after_resolve_data)))
      .await?
    {
      AfterResolveResult::Ignored => Ok(Some(ModuleFactoryResult::default())),
      AfterResolveResult::Data(mut after_resolve_data) => {
        // The dependencies can be modified  in the after resolve hook
        data.dependencies = after_resolve_data.dependencies.take();

        let parsed_resource = parse_resource(after_resolve_data.resource.as_str());
        if let Some(parsed_resource) = parsed_resource {
          if let Some(query) = &parsed_resource.query {
            context_module_options.resource_query.clone_from(query);
          }
          if let Some(fragment) = &parsed_resource.fragment {
            context_module_options
              .resource_fragment
              .clone_from(fragment);
          }
        }

        context_module_options.resource = after_resolve_data.resource;
        if !matches!(&after_resolve_data.pattern, ContextModulePattern::Glob(_)) {
          context_module_options.context_options.context = after_resolve_data.context;
        }
        context_module_options.context_options.pattern = after_resolve_data.pattern.clone();
        context_module_options.context_options.recursive = after_resolve_data.recursive;

        let module = ContextModule::new_with_strict(
          after_resolve_data.resolve_dependencies,
          context_module_options.clone(),
          self.global_override_strict(),
        )
        .boxed();

        Ok(Some(ModuleFactoryResult::new_with_module(module)))
      }
    }
  }

  fn global_override_strict(&self) -> Option<bool> {
    self
      .plugin_driver
      .options
      .module
      .parser
      .as_ref()
      .and_then(|parser| parser.get("javascript"))
      .and_then(|parser| parser.get_javascript())
      .and_then(|options| options.override_strict)
      .map(|strict| matches!(strict, OverrideStrict::Strict))
  }
}

async fn visit_dirs(
  dependencies: &mut Vec<ContextElementDependency>,
  options: &ContextModuleOptions,
  resolve_options: &ResolveInnerOptions<'_>,
  fs: Arc<dyn ReadableFileSystem>,
) -> Result<()> {
  let glob = ContextModuleGlobMatcher::new(options);
  let matcher = ContextModuleMatcher::new(options, resolve_options, glob.as_ref());
  if matcher.is_empty() {
    return Ok(());
  }
  let is_glob = matcher.glob.is_some();

  matcher
    .walk(&options.resource, fs.as_ref(), !is_glob, &mut |_, child| {
      for matched in child.matches {
        if !is_glob
          || !dependencies
            .iter()
            .any(|d| d.user_request == matched.request)
        {
          push_context_element_dependency(dependencies, options, matched);
        }
      }
    })
    .await?;
  Ok(())
}

fn push_context_element_dependency(
  dependencies: &mut Vec<ContextElementDependency>,
  options: &ContextModuleOptions,
  matched: ContextModuleMatchedRequest,
) {
  let resource_request = matched
    .resource_request
    .as_deref()
    .unwrap_or(&matched.request);
  let request = format!(
    "{}{}{}{}",
    options.addon, resource_request, options.resource_query, options.resource_fragment,
  );
  let resource_identifier = ContextElementDependency::create_resource_identifier(
    options.resource.as_str(),
    &request,
    options.context_options.attributes.as_ref(),
  );

  dependencies.push(ContextElementDependency {
    id: DependencyId::new(),
    weak: matches!(
      options.context_options.mode,
      ContextMode::AsyncWeak | ContextMode::Weak
    ),
    request,
    user_request: matched.request,
    category: options.context_options.category,
    context: options.resource.clone().into(),
    layer: options.layer.clone(),
    resource_identifier,
    attributes: options.context_options.attributes.clone(),
    referenced_specifiers: options.context_options.referenced_specifiers.clone(),
    dependency_type: DependencyType::ContextElement(options.type_prefix),
  });
}

fn is_non_exhaustive_import_meta_glob_skipped_dir(dirname: &str) -> bool {
  dirname == "node_modules" || dirname.starts_with('.')
}

#[derive(Clone)]
struct ContextModuleMatchedRequest {
  request: String,
  resource_request: Option<String>,
}

/// The walker clones directory matchers to prepare entries. Only file matches
/// populate `matches`, so a directory clone just copies configuration references.
#[derive(Clone)]
struct ContextModuleMatcher<'a, 'resolver> {
  options: &'a ContextModuleOptions,
  resolve_options: &'a ResolveInnerOptions<'resolver>,
  glob: Option<&'a ContextModuleGlobMatcher<'a>>,
  matches: SmallVec<[ContextModuleMatchedRequest; 1]>,
}

impl<'a, 'resolver> ContextModuleMatcher<'a, 'resolver> {
  fn new(
    options: &'a ContextModuleOptions,
    resolve_options: &'a ResolveInnerOptions<'resolver>,
    glob: Option<&'a ContextModuleGlobMatcher<'a>>,
  ) -> Self {
    Self {
      options,
      resolve_options,
      glob,
      matches: SmallVec::new(),
    }
  }

  fn is_empty(&self) -> bool {
    match &self.options.context_options.pattern {
      ContextModulePattern::Glob(_) => self.glob.is_none_or(ContextModuleGlobMatcher::is_empty),
      pattern => pattern.is_empty(),
    }
  }
}

impl WalkMatcher for ContextModuleMatcher<'_, '_> {
  const VISIT_ON_METADATA_ERROR: bool = true;

  fn match_entry(mut self, dir: &Utf8Path, name: &str, is_directory: bool) -> Option<Self> {
    let context_options = &self.options.context_options;
    if is_directory && !context_options.recursive {
      return None;
    }
    let path = dir.join(name);
    let path = path.as_str();
    if is_directory
      && let Some(glob) = self.glob
      && (!glob.should_visit_dir(path)
        || (!context_options.glob_exhaustive
          && is_non_exhaustive_import_meta_glob_skipped_dir(name)
          && !glob.should_visit_skipped_dir(path)))
    {
      return None;
    }
    if context_options
      .exclude
      .as_ref()
      .is_some_and(|exclude| exclude.test(path))
    {
      return None;
    }
    if is_directory {
      return Some(self);
    }
    if context_options
      .include
      .as_ref()
      .is_some_and(|include| !include.test(path))
    {
      return None;
    }

    // The resolver can return a context with or without a trailing slash.
    let ctx = self.options.resource.as_str();
    let relative = normalize_path_separators(&path[ctx.len()..]);
    let relative_request = if relative.starts_with('/') {
      format!(".{relative}")
    } else {
      format!("./{relative}")
    };
    if let Some(glob) = self.glob {
      // Glob maps expose filesystem paths without the resolver's alternative requests.
      self.matches.push(ContextModuleMatchedRequest {
        request: glob.match_request(path)?,
        resource_request: Some(relative_request),
      });
    } else {
      let reg_exp = context_options.pattern.reg_exp()?;
      let requests = alternative_requests(
        self.resolve_options,
        vec![AlternativeRequest::new(ctx.to_string(), relative_request)],
      );
      self.matches.extend(
        requests
          .into_iter()
          .filter(|item| reg_exp.test(&item.request))
          .map(|item| ContextModuleMatchedRequest {
            request: item.request,
            resource_request: None,
          }),
      );
    }
    (!self.matches.is_empty()).then_some(self)
  }
}

#[derive(Debug, Clone)]
pub struct AlternativeRequest {
  pub context: String,
  pub request: String,
}

impl AlternativeRequest {
  pub fn new(context: String, request: String) -> Self {
    Self { context, request }
  }
}

fn alternative_requests(
  resolve_options: &ResolveInnerOptions,
  mut items: Vec<AlternativeRequest>,
) -> Vec<AlternativeRequest> {
  // TODO: should respect fullySpecified resolve options
  for item in std::mem::take(&mut items) {
    if !resolve_options.is_enforce_extension_enabled() {
      items.push(item.clone());
    }
    for ext in resolve_options.extensions() {
      if item.request.ends_with(ext) {
        items.push(AlternativeRequest::new(
          item.context.clone(),
          item.request[..(item.request.len() - ext.len())].to_string(),
        ));
      }
    }
  }

  for item in std::mem::take(&mut items) {
    items.push(item.clone());
    for main_file in resolve_options.main_files() {
      if item.request.ends_with(&format!("/{main_file}")) {
        items.push(AlternativeRequest::new(
          item.context.clone(),
          item.request[..(item.request.len() - main_file.len())].to_string(),
        ));
        items.push(AlternativeRequest::new(
          item.context.clone(),
          item.request[..(item.request.len() - main_file.len() - 1)].to_string(),
        ));
      }
    }
  }

  for item in std::mem::take(&mut items) {
    // webpack's `hideOriginal`: when the request points into a relative
    // resolve.modules directory (e.g. `./node_modules/`), hide the original
    // `./<dir>/...` request and emit only the bare specifier. The bare
    // specifier can still be added under its own key if the context regExp
    // matches it (same as webpack); but for the common `import('./dir/' + x)`
    // the default `^\./` matcher rejects it, so the file drops out of the map.
    let mut hide_original = false;
    for module in resolve_options.modules() {
      let dir = normalize_path_separators(module);
      if item.request.starts_with(&format!("./{dir}/")) {
        items.push(AlternativeRequest::new(
          item.context.clone(),
          item.request[dir.len() + 3..].to_string(),
        ));
        hide_original = true;
      }
    }
    if !hide_original {
      items.push(item);
    }
  }

  items
}
