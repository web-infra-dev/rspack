use std::{
  borrow::Cow,
  fmt::{self, Display},
  fs,
  hash::Hash,
  path::{Path, PathBuf},
  sync::Arc,
};

use itertools::Itertools;
use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use rspack_error::{impl_empty_diagnosable_trait, miette::IntoDiagnostic, Diagnostic, Result};
use rspack_hash::RspackHash;
use rspack_identifier::{Identifiable, Identifier};
use rspack_macros::impl_source_map_config;
use rspack_regex::RspackRegex;
use rspack_sources::{BoxSource, ConcatSource, RawSource, SourceExt};
use rspack_util::source_map::SourceMapKind;
use rustc_hash::FxHashMap as HashMap;
use rustc_hash::FxHashSet as HashSet;

use crate::{
  block_promise, contextify, get_exports_type_with_strict, impl_build_info_meta,
  returning_function, stringify_map, to_path, AsyncDependenciesBlock,
  AsyncDependenciesBlockIdentifier, BoxDependency, BuildContext, BuildInfo, BuildMeta, BuildResult,
  ChunkGraph, ChunkGroupOptions, CodeGenerationResult, Compilation, ConcatenationScope,
  ContextElementDependency, DependenciesBlock, Dependency, DependencyCategory, DependencyId,
  ExportsType, FakeNamespaceObjectMode, GroupOptions, LibIdentOptions, Module, ModuleType, Resolve,
  ResolveInnerOptions, ResolveOptionsWithDependencyType, ResolverFactory, RuntimeGlobals,
  RuntimeSpec, SourceType,
};

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

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum ContextMode {
  Sync,
  Eager,
  Weak,
  AsyncWeak,
  Lazy,
  LazyOnce,
}

impl ContextMode {
  pub fn as_str(&self) -> &str {
    match self {
      ContextMode::Sync => "sync",
      ContextMode::Eager => "eager",
      ContextMode::Weak => "weak",
      ContextMode::Lazy => "lazy",
      ContextMode::LazyOnce => "lazy-once",
      ContextMode::AsyncWeak => "async-weak",
    }
  }
}

impl From<&str> for ContextMode {
  fn from(value: &str) -> Self {
    match try_convert_str_to_context_mode(value) {
      Some(m) => m,
      // TODO should give warning
      _ => panic!("unknown context mode"),
    }
  }
}

pub fn try_convert_str_to_context_mode(s: &str) -> Option<ContextMode> {
  match s {
    "sync" => Some(ContextMode::Sync),
    "eager" => Some(ContextMode::Eager),
    "weak" => Some(ContextMode::Weak),
    "lazy" => Some(ContextMode::Lazy),
    "lazy-once" => Some(ContextMode::LazyOnce),
    "async-weak" => Some(ContextMode::AsyncWeak),
    // TODO should give warning
    _ => None,
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ContextNameSpaceObject {
  Bool(bool),
  Strict,
  Unset,
}

impl ContextNameSpaceObject {
  pub fn is_false(&self) -> bool {
    matches!(self, ContextNameSpaceObject::Unset)
      || matches!(self, ContextNameSpaceObject::Bool(v) if !v)
  }
}

pub fn context_reg_exp(expr: &str, flags: &str) -> Option<RspackRegex> {
  if expr.is_empty() {
    return None;
  }
  let regexp = RspackRegex::with_flags(expr, flags).expect("reg failed");
  clean_regexp_in_context_module(regexp)
}

pub fn clean_regexp_in_context_module(regexp: RspackRegex) -> Option<RspackRegex> {
  if regexp.sticky() || regexp.global() {
    // TODO: warning
    None
  } else {
    Some(regexp)
  }
}

#[derive(Debug, Clone)]
pub struct ContextOptions {
  pub mode: ContextMode,
  pub recursive: bool,
  pub reg_exp: Option<RspackRegex>,
  // TODO: remove `reg_str`
  pub reg_str: String, // generate context module id
  pub include: Option<String>,
  pub exclude: Option<String>,
  pub category: DependencyCategory,
  pub request: String,
  pub context: String,
  pub namespace_object: ContextNameSpaceObject,
  pub chunk_name: Option<String>,
  pub start: u32,
  pub end: u32,
}

impl Display for ContextOptions {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "ContextOptions|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}",
      self.mode,
      self.recursive,
      self.reg_exp,
      self.reg_str,
      self.include,
      self.exclude,
      self.category,
      self.request,
      self.context,
      self.namespace_object,
      self.chunk_name
    )
  }
}

impl PartialEq for ContextOptions {
  fn eq(&self, other: &Self) -> bool {
    self.mode == other.mode
      && self.recursive == other.recursive
      && self.reg_str == other.reg_str
      && self.include == other.include
      && self.exclude == other.exclude
      && self.category == other.category
      && self.request == other.request
      && self.context == other.context
      && self.namespace_object == other.namespace_object
  }
}

impl Eq for ContextOptions {}

impl Hash for ContextOptions {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.mode.hash(state);
    self.recursive.hash(state);
    self.reg_str.hash(state);
    self.include.hash(state);
    self.exclude.hash(state);
    self.category.hash(state);
    self.request.hash(state);
    self.context.hash(state);
    self.namespace_object.hash(state);
  }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ContextModuleOptions {
  pub addon: String,
  pub resource: String,
  pub resource_query: Option<String>,
  pub resource_fragment: Option<String>,
  pub context_options: ContextOptions,
  pub resolve_options: Option<Box<Resolve>>,
}

impl Display for ContextModuleOptions {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "{}|{:?}|{:?}|{}",
      self.resource, self.resource_query, self.resource_fragment, self.context_options
    )
  }
}

pub enum FakeMapValue {
  Bit(FakeNamespaceObjectMode),
  Map(HashMap<String, FakeNamespaceObjectMode>),
}

#[impl_source_map_config]
#[derive(Debug)]
pub struct ContextModule {
  dependencies: Vec<DependencyId>,
  blocks: Vec<AsyncDependenciesBlockIdentifier>,
  identifier: Identifier,
  options: ContextModuleOptions,
  resolve_factory: Arc<ResolverFactory>,
  build_info: Option<BuildInfo>,
  build_meta: Option<BuildMeta>,
}

impl PartialEq for ContextModule {
  fn eq(&self, other: &Self) -> bool {
    self.identifier == other.identifier
  }
}

impl Eq for ContextModule {}

impl ContextModule {
  pub fn new(options: ContextModuleOptions, resolve_factory: Arc<ResolverFactory>) -> Self {
    Self {
      dependencies: Vec::new(),
      blocks: Vec::new(),
      identifier: create_identifier(&options),
      options,
      resolve_factory,
      build_info: None,
      build_meta: None,
      source_map_kind: SourceMapKind::None,
    }
  }

  pub fn id<'chunk_graph>(&self, chunk_graph: &'chunk_graph ChunkGraph) -> &'chunk_graph str {
    chunk_graph
      .get_module_id(self.identifier)
      .as_ref()
      .expect("module id not found")
      .as_str()
  }

  fn get_fake_map(
    &self,
    dependencies: impl IntoIterator<Item = &DependencyId>,
    compilation: &Compilation,
  ) -> FakeMapValue {
    let dependencies = dependencies.into_iter();
    if self.options.context_options.namespace_object.is_false() {
      return FakeMapValue::Bit(FakeNamespaceObjectMode::NAMESPACE);
    }
    let mut has_type = 0;
    let mut fake_map = HashMap::default();
    let sorted_modules = dependencies
      .filter_map(|dep_id| {
        compilation
          .get_module_graph()
          .module_identifier_by_dependency_id(dep_id)
          .map(|m| (m, dep_id))
      })
      .filter_map(|(m, dep)| {
        compilation
          .chunk_graph
          .get_module_id(*m)
          .clone()
          .map(|id| (id, dep))
      })
      .sorted_unstable_by_key(|(module_id, _)| module_id.to_string());
    for (module_id, dep) in sorted_modules {
      let exports_type = get_exports_type_with_strict(
        compilation.get_module_graph(),
        dep,
        matches!(
          self.options.context_options.namespace_object,
          ContextNameSpaceObject::Strict
        ),
      );
      match exports_type {
        ExportsType::Namespace => {
          fake_map.insert(module_id, FakeNamespaceObjectMode::NAMESPACE);
          has_type |= 1;
        }
        ExportsType::Dynamic => {
          fake_map.insert(module_id, FakeNamespaceObjectMode::DYNAMIC);
          has_type |= 2;
        }
        ExportsType::DefaultOnly => {
          fake_map.insert(module_id, FakeNamespaceObjectMode::MODULE_ID);
          has_type |= 4;
        }
        ExportsType::DefaultWithNamed => {
          fake_map.insert(module_id, FakeNamespaceObjectMode::DEFAULT_WITH_NAMED);
          has_type |= 8;
        }
      }
    }

    match has_type {
      0 | 1 => FakeMapValue::Bit(FakeNamespaceObjectMode::NAMESPACE),
      2 => FakeMapValue::Bit(FakeNamespaceObjectMode::DYNAMIC),
      4 => FakeMapValue::Bit(FakeNamespaceObjectMode::MODULE_ID),
      8 => FakeMapValue::Bit(FakeNamespaceObjectMode::DEFAULT_WITH_NAMED),
      _ => FakeMapValue::Map(fake_map),
    }
  }

  fn get_return_module_object_source(&self, fake_map: &FakeMapValue, async_module: bool) -> String {
    if let FakeMapValue::Bit(bit) = fake_map {
      return self.get_return(bit, async_module);
    }
    format!(
      "return {}(id, fakeMap[id]{});",
      RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT,
      if async_module { "| 16" } else { "" },
    )
  }

  fn get_return(&self, fake_map_bit: &FakeNamespaceObjectMode, async_module: bool) -> String {
    if *fake_map_bit == FakeNamespaceObjectMode::NAMESPACE {
      return format!("return {}(id);", RuntimeGlobals::REQUIRE);
    }
    format!(
      "return {}(id, {}{});",
      RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT,
      fake_map_bit,
      if async_module { "| 16" } else { "" },
    )
  }

  fn get_user_request_map(
    &self,
    dependencies: impl IntoIterator<Item = &DependencyId>,
    compilation: &Compilation,
  ) -> HashMap<String, String> {
    let dependencies = dependencies.into_iter();
    let mut map = HashMap::default();
    for dependency in dependencies {
      if let Some(module_identifier) = compilation
        .get_module_graph()
        .module_identifier_by_dependency_id(dependency)
      {
        if let Some(dependency) = compilation.get_module_graph().dependency_by_id(dependency) {
          let request = if let Some(d) = dependency.as_module_dependency() {
            Some(d.user_request().to_string())
          } else {
            dependency
              .as_context_dependency()
              .map(|d| d.request().to_string())
          };
          if let Some(request) = request {
            map.insert(
              request,
              if let Some(module_id) = compilation.chunk_graph.get_module_id(*module_identifier) {
                format!("\"{module_id}\"")
              } else {
                "null".to_string()
              },
            );
          }
        }
      }
    }
    map
  }

  fn get_block_promise_map(
    &self,
    blocks: impl IntoIterator<Item = &AsyncDependenciesBlock>,
    compilation: &Compilation,
    runtime_requirements: &mut RuntimeGlobals,
  ) -> HashMap<String, String> {
    let blocks = blocks
      .into_iter()
      .filter_map(|b| b.get_dependencies().first().map(|first| (b, first)));
    let mut map = HashMap::default();
    for (block, dep_id) in blocks {
      if let Some(dependency) = compilation
        .get_module_graph()
        .dependency_by_id(dep_id)
        .and_then(|d| d.as_module_dependency())
      {
        let getter = returning_function(
          &block_promise(Some(&block.identifier()), runtime_requirements, compilation),
          "",
        );
        map.insert(dependency.user_request().to_string(), getter);
      }
    }
    map
  }

  #[inline]
  fn get_source_string(
    &self,
    compilation: &Compilation,
    runtime_requirements: &mut RuntimeGlobals,
  ) -> BoxSource {
    match self.options.context_options.mode {
      ContextMode::Lazy => self.get_lazy_source(compilation, runtime_requirements),
      ContextMode::LazyOnce => {
        let block = self
          .get_blocks()
          .first()
          .expect("LazyOnce ContextModule should have first block");
        let block = compilation
          .get_module_graph()
          .block_by_id(block)
          .expect("should have block");
        self.generate_source(block.get_dependencies(), compilation)
      }
      _ => self.generate_source(self.get_dependencies(), compilation),
    }
  }

  fn get_lazy_source(
    &self,
    compilation: &Compilation,
    runtime_requirements: &mut RuntimeGlobals,
  ) -> BoxSource {
    let blocks = self
      .get_blocks()
      .iter()
      .filter_map(|b| compilation.get_module_graph().block_by_id(b));
    let block_map = self.get_block_promise_map(blocks.clone(), compilation, runtime_requirements);
    let dependencies = blocks.filter_map(|b| b.get_dependencies().first());
    let fake_map = self.get_fake_map(dependencies.clone(), compilation);
    let map = self.get_user_request_map(dependencies, compilation);
    let return_module_object = self.get_return_module_object_source(&fake_map, true);
    let mut source = ConcatSource::default();
    source.add(RawSource::from(format!(
      "var blockMap = {};\n",
      stringify_map(&block_map)
    )));
    source.add(RawSource::from(format!(
      "var map = {};\n",
      stringify_map(&map)
    )));
    if let FakeMapValue::Map(map) = &fake_map {
      source.add(RawSource::from(format!(
        "var fakeMap = {};\n",
        stringify_map(map)
      )));
    }
    source.add(RawSource::from(format!(
      r#"
      function webpackAsyncContext(req) {{
        if(!__webpack_require__.o(map, req)) {{
          return Promise.resolve().then(function() {{
            var e = new Error("Cannot find module '" + req + "'");
            e.code = 'MODULE_NOT_FOUND';
            throw e;
          }});
        }}
        var blockGetter = blockMap[req];
        var id = map[req];
        return blockGetter().then(function() {{
          {return_module_object}
        }});
      }}
      webpackAsyncContext.keys = function() {{
        return Object.keys(map);
      }};
      webpackAsyncContext.id = {:?};
      module.exports = webpackAsyncContext;
      "#,
      self.id(&compilation.chunk_graph)
    )));
    source.boxed()
  }

  fn generate_source(&self, dependencies: &[DependencyId], compilation: &Compilation) -> BoxSource {
    let map = self.get_user_request_map(dependencies, compilation);
    let fake_map = self.get_fake_map(dependencies, compilation);
    let mode = &self.options.context_options.mode;
    let return_module_object = {
      match *mode {
        ContextMode::Sync | ContextMode::Weak | ContextMode::Eager => {
          self.get_return_module_object_source(&fake_map, false)
        }
        ContextMode::AsyncWeak | ContextMode::LazyOnce => {
          self.get_return_module_object_source(&fake_map, true)
        }
        ContextMode::Lazy => {
          unreachable!("lazy mode shouldn't be handled by get_source_string")
        }
      }
    };
    let is_async = matches!(
      mode,
      ContextMode::LazyOnce | ContextMode::AsyncWeak | ContextMode::Eager
    );
    let mut source = ConcatSource::default();
    source.add(RawSource::from(format!(
      "var map = {};\n",
      stringify_map(&map)
    )));
    if let FakeMapValue::Map(map) = &fake_map {
      source.add(RawSource::from(format!(
        "var fakeMap = {};\n",
        stringify_map(map)
      )));
    }

    // webpackContext
    source.add(RawSource::from("function webpackContext(req) {\n"));
    if is_async {
      source.add(RawSource::from(
        "return webpackContextResolve(req).then(function(id) {\n",
      ));
    } else {
      source.add(RawSource::from("var id = webpackContextResolve(req);\n"));
    }
    if matches!(mode, ContextMode::AsyncWeak | ContextMode::Weak) {
      source.add(RawSource::from(
        r#"
        if(!__webpack_require__.m[id]) {
          var e = new Error("Module '" + req + "' ('" + id + "') is not available (weak dependency)");
          e.code = 'MODULE_NOT_FOUND';
          throw e;
        }
        "#,
      ));
    }
    source.add(RawSource::from(format!("\n{return_module_object}\n")));
    if is_async {
      source.add(RawSource::from("\n});\n"));
    }
    source.add(RawSource::from("\n}\n"));

    // webpackContextResolve
    source.add(RawSource::from("function webpackContextResolve(req) {\n"));
    if is_async {
      source.add(RawSource::from(
        r#"
        // Here Promise.resolve().then() is used instead of new Promise() to prevent
        // uncaught exception popping up in devtools
        return Promise.resolve().then(function() {
        "#,
      ));
    }
    source.add(RawSource::from(
      r#"
      if(!__webpack_require__.o(map, req)) {
        var e = new Error("Cannot find module '" + req + "'");
        e.code = 'MODULE_NOT_FOUND';
        throw e;
      }
      return map[req];
    "#,
    ));
    if is_async {
      source.add(RawSource::from("\n});\n"));
    }
    source.add(RawSource::from("\n}\n"));

    source.add(RawSource::from(format!(
      "webpackContext.id = '{}';\n",
      serde_json::to_string(self.id(&compilation.chunk_graph))
        .unwrap_or_else(|e| panic!("{}", e.to_string()))
    )));
    source.add(RawSource::from(
      r#"
      webpackContext.keys = function webpackContextKeys() {
        return Object.keys(map);
      };
      webpackContext.resolve = webpackContextResolve;
      module.exports = webpackContext;
      "#,
    ));
    source.boxed()
  }
}

impl DependenciesBlock for ContextModule {
  fn add_block_id(&mut self, block: AsyncDependenciesBlockIdentifier) {
    self.blocks.push(block)
  }

  fn get_blocks(&self) -> &[AsyncDependenciesBlockIdentifier] {
    &self.blocks
  }

  fn add_dependency_id(&mut self, dependency: DependencyId) {
    self.dependencies.push(dependency)
  }

  fn get_dependencies(&self) -> &[DependencyId] {
    &self.dependencies
  }
}

#[async_trait::async_trait]
impl Module for ContextModule {
  impl_build_info_meta!();

  fn module_type(&self) -> &ModuleType {
    &ModuleType::Js
  }

  fn source_types(&self) -> &[SourceType] {
    &[SourceType::JavaScript]
  }
  fn get_diagnostics(&self) -> Vec<Diagnostic> {
    vec![]
  }
  fn original_source(&self) -> Option<&dyn rspack_sources::Source> {
    None
  }

  fn readable_identifier(&self, _context: &crate::Context) -> std::borrow::Cow<str> {
    self.identifier.as_str().into()
  }

  fn size(&self, _source_type: &crate::SourceType) -> f64 {
    160.0
  }

  fn lib_ident(&self, options: LibIdentOptions) -> Option<Cow<str>> {
    let mut id = contextify(options.context, &self.options.resource);
    id.push_str(format!(" {:?} ", self.options.context_options.mode).as_str());
    if self.options.context_options.recursive {
      id.push_str(" recursive ");
    }
    id.push_str(&self.options.context_options.reg_str);
    Some(Cow::Owned(id))
  }

  async fn build(
    &mut self,
    build_context: BuildContext<'_>,
    _: Option<&Compilation>,
  ) -> Result<BuildResult> {
    self.resolve_dependencies(build_context)
  }

  fn code_generation(
    &self,
    compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
    _: Option<ConcatenationScope>,
  ) -> Result<CodeGenerationResult> {
    let mut code_generation_result = CodeGenerationResult::default();
    code_generation_result
      .runtime_requirements
      .insert(RuntimeGlobals::MODULE);
    code_generation_result
      .runtime_requirements
      .insert(RuntimeGlobals::HAS_OWN_PROPERTY);

    // TODO inject runtime globals by dep size
    code_generation_result
      .runtime_requirements
      .insert(RuntimeGlobals::REQUIRE);
    match self.options.context_options.mode {
      ContextMode::Weak => {
        code_generation_result
          .runtime_requirements
          .insert(RuntimeGlobals::MODULE_FACTORIES);
      }
      ContextMode::AsyncWeak => {
        code_generation_result
          .runtime_requirements
          .insert(RuntimeGlobals::MODULE_FACTORIES);
        code_generation_result
          .runtime_requirements
          .insert(RuntimeGlobals::ENSURE_CHUNK);
      }
      ContextMode::Lazy | ContextMode::LazyOnce => {
        code_generation_result
          .runtime_requirements
          .insert(RuntimeGlobals::ENSURE_CHUNK);
      }
      _ => {}
    }
    let mut all_deps = self.get_dependencies().to_vec();
    for block in self.get_blocks() {
      let block = compilation
        .get_module_graph()
        .block_by_id(block)
        .expect("should have block in ContextModule code_generation");
      all_deps.extend(block.get_dependencies());
    }
    let fake_map = self.get_fake_map(all_deps.iter(), compilation);
    if !matches!(fake_map, FakeMapValue::Bit(bit) if bit == FakeNamespaceObjectMode::NAMESPACE) {
      code_generation_result
        .runtime_requirements
        .insert(RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT);
    }
    let source = self.get_source_string(
      compilation,
      &mut code_generation_result.runtime_requirements,
    );
    code_generation_result.add(SourceType::JavaScript, source);
    code_generation_result.set_hash(
      &compilation.options.output.hash_function,
      &compilation.options.output.hash_digest,
      &compilation.options.output.hash_salt,
    );
    Ok(code_generation_result)
  }
}

impl_empty_diagnosable_trait!(ContextModule);

impl Identifiable for ContextModule {
  fn identifier(&self) -> Identifier {
    self.identifier
  }
}

impl Hash for ContextModule {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    "__rspack_internal__ContextModule".hash(state);
    self.identifier.hash(state);
  }
}

static WEBPACK_CHUNK_NAME_PLACEHOLDER: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\[index|request\]").expect("regexp init failed"));
static WEBPACK_CHUNK_NAME_INDEX_PLACEHOLDER: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\[index\]").expect("regexp init failed"));
static WEBPACK_CHUNK_NAME_REQUEST_PLACEHOLDER: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\[request\]").expect("regexp init failed"));

impl ContextModule {
  fn visit_dirs(
    ctx: &str,
    dir: &Path,
    dependencies: &mut Vec<ContextElementDependency>,
    options: &ContextModuleOptions,
    resolve_options: &ResolveInnerOptions,
  ) -> Result<()> {
    if !dir.is_dir() {
      return Ok(());
    }
    for entry in fs::read_dir(dir).into_diagnostic()? {
      let path = entry.into_diagnostic()?.path();
      if path.is_dir() {
        if options.context_options.recursive {
          Self::visit_dirs(ctx, &path, dependencies, options, resolve_options)?;
        }
      } else if path
        .file_name()
        .map_or(false, |name| name.to_string_lossy().starts_with('.'))
      {
        // ignore hidden files
        continue;
      } else {
        // FIXME: nodejs resolver return path of context, sometimes is '/a/b', sometimes is '/a/b/'
        let relative_path = {
          let p = path
            .to_string_lossy()
            .to_string()
            .drain(ctx.len()..)
            .collect::<String>()
            .replace('\\', "/");
          if p.starts_with('/') {
            format!(".{p}")
          } else {
            format!("./{p}")
          }
        };
        let requests = alternative_requests(
          resolve_options,
          vec![AlternativeRequest::new(ctx.to_string(), relative_path)],
        );

        let Some(reg_exp) = &options.context_options.reg_exp else {
          return Ok(());
        };

        requests.iter().for_each(|r| {
          if !reg_exp.test(&r.request) {
            return;
          }
          dependencies.push(ContextElementDependency {
            id: DependencyId::new(),
            request: format!(
              "{}{}{}{}",
              options.addon,
              r.request,
              options.resource_query.clone().unwrap_or_default(),
              options.resource_fragment.clone().unwrap_or_default()
            ),
            user_request: r.request.to_string(),
            category: options.context_options.category,
            context: options.resource.clone().into(),
            options: options.context_options.clone(),
            resource_identifier: format!("context{}|{}", &options.resource, path.to_string_lossy()),
            referenced_exports: None,
          });
        })
      }
    }
    Ok(())
  }

  fn resolve_dependencies(&self, build_context: BuildContext<'_>) -> Result<BuildResult> {
    tracing::trace!("resolving context module path {}", self.options.resource);

    let resolver = &self.resolve_factory.get(ResolveOptionsWithDependencyType {
      resolve_options: self.options.resolve_options.clone(),
      resolve_to_context: false,
      dependency_category: self.options.context_options.category,
    });

    let mut context_element_dependencies = vec![];
    Self::visit_dirs(
      &self.options.resource,
      Path::new(&self.options.resource),
      &mut context_element_dependencies,
      &self.options,
      &resolver.options(),
    )?;
    context_element_dependencies.sort_by_cached_key(|d| d.user_request.to_string());

    tracing::trace!(
      "resolving dependencies for {:?}",
      context_element_dependencies
    );

    let mut dependencies: Vec<BoxDependency> = vec![];
    let mut blocks = vec![];
    if matches!(self.options.context_options.mode, ContextMode::LazyOnce)
      && !context_element_dependencies.is_empty()
    {
      let name = self.options.context_options.chunk_name.clone();
      let mut block = AsyncDependenciesBlock::new(
        self.identifier,
        Some(
          (
            self.options.context_options.start,
            self.options.context_options.end,
          )
            .into(),
        ),
        None,
        context_element_dependencies
          .into_iter()
          .map(|dep| Box::new(dep) as Box<dyn Dependency>)
          .collect(),
      );
      block.set_group_options(GroupOptions::ChunkGroup(ChunkGroupOptions::new(
        name, None, None,
      )));
      blocks.push(block);
    } else if matches!(self.options.context_options.mode, ContextMode::Lazy) {
      let mut index = 0;
      for context_element_dependency in context_element_dependencies {
        let name = self
          .options
          .context_options
          .chunk_name
          .as_ref()
          .map(|name| {
            let name = if !WEBPACK_CHUNK_NAME_PLACEHOLDER.is_match(name) {
              Cow::Owned(format!("{name}[index]"))
            } else {
              Cow::Borrowed(name)
            };
            let name = WEBPACK_CHUNK_NAME_INDEX_PLACEHOLDER
              .replace_all(&name, |_: &Captures| index.to_string());
            index += 1;
            let name = WEBPACK_CHUNK_NAME_REQUEST_PLACEHOLDER.replace_all(&name, |_: &Captures| {
              to_path(&context_element_dependency.user_request)
            });
            name.into_owned()
          });
        let mut block = AsyncDependenciesBlock::new(
          self.identifier,
          Some(
            (
              self.options.context_options.start,
              self.options.context_options.end,
            )
              .into(),
          ),
          Some(&context_element_dependency.user_request.clone()),
          vec![Box::new(context_element_dependency)],
        );
        block.set_group_options(GroupOptions::ChunkGroup(ChunkGroupOptions::new(
          name, None, None,
        )));
        blocks.push(block);
      }
    } else {
      dependencies = context_element_dependencies
        .into_iter()
        .map(|d| Box::new(d) as BoxDependency)
        .collect();
    }

    let mut hasher = RspackHash::from(&build_context.compiler_options.output);
    self.update_hash(&mut hasher);

    let mut context_dependencies: HashSet<PathBuf> = Default::default();
    context_dependencies.insert(PathBuf::from(&self.options.resource));

    let build_info = BuildInfo {
      hash: Some(hasher.digest(&build_context.compiler_options.output.hash_digest)),
      context_dependencies,
      ..Default::default()
    };

    Ok(BuildResult {
      build_info,
      build_meta: BuildMeta::default(),
      dependencies,
      blocks,
      analyze_result: Default::default(),
      optimization_bailouts: vec![],
    })
  }
}

fn create_identifier(options: &ContextModuleOptions) -> Identifier {
  Identifier::from(format!("{options}"))
}

pub fn normalize_context(str: &str) -> String {
  if str == "./" || str == "." {
    return "".to_string();
  }
  if str.ends_with('/') {
    return str.to_string();
  }
  str.to_string() + "/"
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
    items.push(item.clone());
    // TODO resolveOptions.modules can be array
    for module in resolve_options.modules() {
      let dir = module.replace('\\', "/");
      let full_path: String = format!("{}{}", item.context.replace('\\', "/"), &item.request[1..]);
      if full_path.starts_with(&dir) {
        items.push(AlternativeRequest::new(
          item.context.clone(),
          full_path[(dir.len() + 1)..].to_string(),
        ));
      }
    }
  }

  items
}
