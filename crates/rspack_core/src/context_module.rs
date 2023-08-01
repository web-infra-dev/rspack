use std::{
  borrow::Cow,
  fmt::{self, Display},
  fs,
  hash::Hash,
  path::{Path, PathBuf},
  sync::Arc,
};

use nodejs_resolver::EnforceExtension;
use rspack_error::{internal_error, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};
use rspack_hash::RspackHash;
use rspack_identifier::{Identifiable, Identifier};
use rspack_regex::RspackRegex;
use rspack_sources::{BoxSource, ConcatSource, RawSource, SourceExt};
use rustc_hash::FxHashMap as HashMap;
use rustc_hash::FxHashSet as HashSet;

use crate::{
  contextify, get_exports_type_with_strict, stringify_map, BoxModuleDependency, BuildContext,
  BuildInfo, BuildMeta, BuildResult, ChunkGraph, CodeGenerationResult, Compilation,
  ContextElementDependency, DependencyCategory, DependencyId, DependencyType, ExportsType,
  FakeNamespaceObjectMode, LibIdentOptions, Module, ModuleType, Resolve,
  ResolveOptionsWithDependencyType, ResolverFactory, RuntimeGlobals, SourceType,
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

#[derive(Debug, Clone)]
pub struct ContextOptions {
  pub mode: ContextMode,
  pub recursive: bool,
  pub reg_exp: RspackRegex,
  pub reg_str: String, // generate context module id
  pub include: Option<String>,
  pub exclude: Option<String>,
  pub category: DependencyCategory,
  pub request: String,
  pub namespace_object: ContextNameSpaceObject,
}

impl Display for ContextOptions {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "({:?}, {}, {},  {:?}, {:?},  {:?}, {}, {:?})",
      self.mode,
      self.recursive,
      self.reg_str,
      self.include,
      self.exclude,
      self.category,
      self.request,
      self.namespace_object
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
    self.namespace_object.hash(state);
  }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ContextModuleOptions {
  pub resource: String,
  pub resource_query: Option<String>,
  pub resource_fragment: Option<String>,
  pub context_options: ContextOptions,
  pub resolve_options: Option<Resolve>,
}

impl Display for ContextModuleOptions {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "{}|{:?}|{:?}|{:?}",
      self.resource, self.resource_query, self.resource_fragment, self.context_options
    )
  }
}

pub enum FakeMapValue {
  Bit(FakeNamespaceObjectMode),
  Map(HashMap<String, FakeNamespaceObjectMode>),
}

#[derive(Debug)]
pub struct ContextModule {
  identifier: Identifier,
  options: ContextModuleOptions,
  resolve_factory: Arc<ResolverFactory>,
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
      identifier: create_identifier(&options),
      options,
      resolve_factory,
    }
  }

  pub fn id<'chunk_graph>(&self, chunk_graph: &'chunk_graph ChunkGraph) -> &'chunk_graph str {
    chunk_graph
      .get_module_id(self.identifier)
      .as_ref()
      .expect("module id not found")
      .as_str()
  }

  pub fn get_fake_map(&self, compilation: &Compilation) -> FakeMapValue {
    if self.options.context_options.namespace_object.is_false() {
      return FakeMapValue::Bit(FakeNamespaceObjectMode::NAMESPACE);
    }
    let mut has_type = 0;
    let mut fake_map = HashMap::default();
    if let Some(dependencies) = compilation
      .module_graph
      .dependencies_by_module_identifier(&self.identifier)
    {
      for dependency_id in dependencies {
        if let Some(module_identifier) = compilation
          .module_graph
          .module_identifier_by_dependency_id(dependency_id)
        {
          if let Some(module_id) = compilation
            .chunk_graph
            .get_module_id(*module_identifier)
            .clone()
          {
            let exports_type = get_exports_type_with_strict(
              &compilation.module_graph,
              dependency_id,
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

  pub fn get_return_module_object_source(
    &self,
    fake_map: &FakeMapValue,
    async_module: bool,
  ) -> String {
    if let FakeMapValue::Bit(bit) = fake_map {
      return self.get_return(bit, async_module);
    }
    format!(
      "return {}(id, fakeMap[id]{});",
      RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT,
      if async_module { "| 16" } else { "" },
    )
  }

  pub fn get_return(&self, fake_map_bit: &FakeNamespaceObjectMode, async_module: bool) -> String {
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

  pub fn get_user_request_map(&self, compilation: &Compilation) -> HashMap<String, String> {
    let mut map = HashMap::default();
    if let Some(dependencies) = compilation
      .module_graph
      .dependencies_by_module_identifier(&self.identifier)
    {
      for dependency in dependencies {
        if let Some(module_identifier) = compilation
          .module_graph
          .module_identifier_by_dependency_id(dependency)
        {
          let dependency = compilation
            .module_graph
            .dependency_by_id(dependency)
            .expect("should have dependency");
          map.insert(
            dependency.user_request().to_string(),
            if let Some(module_id) = compilation.chunk_graph.get_module_id(*module_identifier) {
              format!("\"{module_id}\"")
            } else {
              "null".to_string()
            },
          );
        }
      }
    }
    map
  }

  #[inline]
  pub fn get_source_string(&self, compilation: &Compilation) -> Result<BoxSource> {
    match self.options.context_options.mode {
      ContextMode::Lazy => Ok(self.get_lazy_source(compilation)),
      _ => self.generate_source(compilation),
    }
  }

  pub fn get_lazy_source(&self, compilation: &Compilation) -> BoxSource {
    let map = self.get_user_request_map(compilation);
    let fake_map = self.get_fake_map(compilation);
    let return_module_object = self.get_return_module_object_source(&fake_map, true);
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
        var id = map[req];
        return __webpack_require__.el(id).then(function() {{
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

  pub fn generate_source(&self, compilation: &Compilation) -> Result<BoxSource> {
    let map = self.get_user_request_map(compilation);
    let fake_map = self.get_fake_map(compilation);
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
        .map_err(|e| internal_error!(e.to_string()))?
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
    Ok(source.boxed())
  }
}

#[async_trait::async_trait]
impl Module for ContextModule {
  fn module_type(&self) -> &ModuleType {
    &ModuleType::Js
  }

  fn source_types(&self) -> &[SourceType] {
    &[SourceType::JavaScript]
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
  ) -> Result<TWithDiagnosticArray<BuildResult>> {
    self.resolve_dependencies(build_context)
  }

  fn code_generation(&self, compilation: &Compilation) -> Result<CodeGenerationResult> {
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
        code_generation_result
          .runtime_requirements
          .insert(RuntimeGlobals::LOAD_CHUNK_WITH_MODULE);
      }
      _ => {}
    }
    let fake_map = self.get_fake_map(compilation);
    if !matches!(fake_map, FakeMapValue::Bit(bit) if bit == FakeNamespaceObjectMode::NAMESPACE) {
      code_generation_result
        .runtime_requirements
        .insert(RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT);
    }
    code_generation_result.add(SourceType::JavaScript, self.get_source_string(compilation)?);
    code_generation_result.set_hash(
      &compilation.options.output.hash_function,
      &compilation.options.output.hash_digest,
      &compilation.options.output.hash_salt,
    );
    Ok(code_generation_result)
  }
}

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

impl ContextModule {
  pub fn resolve_dependencies(
    &self,
    build_context: BuildContext<'_>,
  ) -> Result<TWithDiagnosticArray<BuildResult>> {
    let mut dependencies = vec![];

    tracing::trace!("resolving context module path {}", self.options.resource);

    fn visit_dirs(
      ctx: &str,
      dir: &Path,
      dependencies: &mut Vec<BoxModuleDependency>,
      options: &ContextModuleOptions,
      resolve_options: &nodejs_resolver::Options,
    ) -> Result<()> {
      if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
          let entry = entry?;
          let path = entry.path();
          if path.is_dir() {
            if options.context_options.recursive {
              visit_dirs(ctx, &path, dependencies, options, resolve_options)?;
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

            requests.iter().for_each(|r| {
              if options.context_options.reg_exp.test(&r.request) {
                dependencies.push(Box::new(ContextElementDependency {
                  id: DependencyId::new(),
                  request: format!(
                    "{}{}{}",
                    r.request,
                    options.resource_query.clone().unwrap_or_default(),
                    options.resource_fragment.clone().unwrap_or_default()
                  ),
                  user_request: r.request.to_string(),
                  category: options.context_options.category,
                  context: options.resource.clone().into(),
                  options: options.context_options.clone(),
                  resource_identifier: format!(
                    "context{}|{}",
                    &options.resource,
                    path.to_string_lossy()
                  ),
                }));
              }
            })
          }
        }
      }
      Ok(())
    }

    let resolver = &self.resolve_factory.get(ResolveOptionsWithDependencyType {
      resolve_options: self.options.resolve_options.clone(),
      resolve_to_context: false,
      dependency_type: DependencyType::ContextElement,
      dependency_category: self.options.context_options.category,
    });

    visit_dirs(
      &self.options.resource,
      Path::new(&self.options.resource),
      &mut dependencies,
      &self.options,
      resolver.options(),
    )?;

    tracing::trace!("resolving dependencies for {:?}", dependencies);

    let mut hasher = RspackHash::from(&build_context.compiler_options.output);
    self.update_hash(&mut hasher);

    let mut context_dependencies: HashSet<PathBuf> = Default::default();
    context_dependencies.insert(PathBuf::from(&self.options.resource));

    let build_info = BuildInfo {
      hash: Some(hasher.digest(&build_context.compiler_options.output.hash_digest)),
      context_dependencies,
      ..Default::default()
    };

    Ok(
      BuildResult {
        build_info,
        build_meta: BuildMeta::default(),
        dependencies,
        analyze_result: Default::default(),
      }
      .with_diagnostic(vec![]),
    )
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
  resolve_options: &nodejs_resolver::Options,
  mut items: Vec<AlternativeRequest>,
) -> Vec<AlternativeRequest> {
  // TODO: should respect fullySpecified resolve options
  for mut item in std::mem::take(&mut items) {
    if !matches!(resolve_options.enforce_extension, EnforceExtension::Enabled) {
      items.push(item.clone());
    }
    for ext in &resolve_options.extensions {
      if item.request.ends_with(ext) {
        items.push(AlternativeRequest::new(
          item.context.clone(),
          item
            .request
            .drain(..(item.request.len() - ext.len()))
            .collect(),
        ));
      }
    }
  }

  for mut item in std::mem::take(&mut items) {
    items.push(item.clone());
    for main_file in &resolve_options.main_files {
      if item.request.ends_with(&format!("/{main_file}")) {
        items.push(AlternativeRequest::new(
          item.context.clone(),
          item
            .request
            .clone()
            .drain(..(item.request.len() - main_file.len()))
            .collect(),
        ));
        items.push(AlternativeRequest::new(
          item.context.clone(),
          item
            .request
            .drain(..(item.request.len() - main_file.len() - 1))
            .collect(),
        ));
      }
    }
  }

  for mut item in std::mem::take(&mut items) {
    items.push(item.clone());
    // TODO resolveOptions.modules can be array
    for module in &resolve_options.modules {
      let dir = module.replace('\\', "/");
      let mut full_path: String = format!(
        "{}{}",
        item.context.replace('\\', "/"),
        item.request.drain(1..).collect::<String>(),
      );
      if full_path.starts_with(&dir) {
        items.push(AlternativeRequest::new(
          item.context.clone(),
          full_path.drain((dir.len() + 1)..).collect(),
        ));
      }
    }
  }

  items
}

pub fn create_resource_identifier_for_context_dependency(options: &ContextOptions) -> String {
  format!("{options}")
}
