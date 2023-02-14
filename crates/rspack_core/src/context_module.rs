use std::{fs, hash::Hash, path::Path};

use regex::Regex;
use rspack_error::{IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};
use rspack_sources::{BoxSource, ConcatSource, RawSource, SourceExt};
use rustc_hash::FxHashMap as HashMap;
use sugar_path::{AsPath, SugarPath};

use crate::{
  runtime_globals, stringify_map, stringify_value_vec_map, AstOrSource, BoxModuleDependency,
  BuildContext, BuildResult, ChunkGraph, CodeGenerationResult, Compilation,
  ContextElementDependency, DependencyCategory, GenerationResult, Identifiable, Identifier, Module,
  ModuleType, SourceType,
};

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum ContextMode {
  Sync,
  Eager,
  Weak,
  AsyncWeak,
  Lazy,
  LazyOnce,
}

#[derive(Debug, Clone)]
pub struct ContextOptions {
  pub mode: ContextMode,
  pub recursive: bool,
  pub reg_exp: Regex,
  pub include: Option<String>,
  pub exclude: Option<String>,
  pub category: DependencyCategory,
  pub request: String,
}

impl PartialEq for ContextOptions {
  fn eq(&self, other: &Self) -> bool {
    self.mode == other.mode
      && self.recursive == other.recursive
    //   && self.regExp == other.regExp
      && self.include == other.include
      && self.exclude == other.exclude
      && self.category == other.category
  }
}

impl Eq for ContextOptions {}

impl Hash for ContextOptions {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.mode.hash(state);
    self.recursive.hash(state);
    // self.regExp.hash(state);
    self.include.hash(state);
    self.exclude.hash(state);
    self.category.hash(state);
  }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ContextModuleOptions {
  pub resource: String,
  pub resource_query: Option<String>,
  pub resource_fragment: Option<String>,
  pub context_options: ContextOptions,
}

#[derive(Debug, Eq)]
pub struct ContextModule {
  identifier: Identifier,
  options: ContextModuleOptions,
}

impl ContextModule {
  pub fn new(options: ContextModuleOptions) -> Self {
    Self {
      identifier: create_identifier(&options),
      options,
    }
  }

  pub fn id<'chunk_graph>(&self, chunk_graph: &'chunk_graph ChunkGraph) -> &'chunk_graph str {
    chunk_graph
      .get_module_id(self.identifier)
      .as_ref()
      .expect("module id not found")
      .as_str()
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
          if let Some(id) = compilation.chunk_graph.get_module_id(*module_identifier) {
            map.insert(id.to_string(), "".to_string());
          }
        }
      }
    }
    map
  }

  #[inline]
  pub fn get_source_string(&self, compilation: &Compilation) -> BoxSource {
    match self.options.context_options.mode {
      ContextMode::Lazy => self.get_lazy_source(compilation),
      _ => self.generate_source(compilation),
    }
  }

  pub fn get_lazy_source(&self, compilation: &Compilation) -> BoxSource {
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
          if let Some(id) = compilation.chunk_graph.get_module_id(*module_identifier) {
            let chunk_group = compilation
              .chunk_graph
              .get_block_chunk_group(module_identifier, &compilation.chunk_group_by_ukey);

            let chunk_ids = chunk_group
              .chunks
              .iter()
              .map(|chunk_ukey| {
                let chunk = compilation
                  .chunk_by_ukey
                  .get(chunk_ukey)
                  .unwrap_or_else(|| panic!("chunk should exist"));
                chunk.expect_id().to_string()
              })
              .collect::<Vec<_>>();

            map.insert(id.to_string(), chunk_ids);
          }
        }
      }
    }

    RawSource::from(
      include_str!("runtime/lazy_context_module.js")
        .replace("$ID$", self.id(&compilation.chunk_graph))
        .replace("$MAP$", &stringify_value_vec_map(&map)),
    )
    .boxed()
  }

  pub fn generate_source(&self, compilation: &Compilation) -> BoxSource {
    let map = self.get_user_request_map(compilation);
    let mode = &self.options.context_options.mode;
    let is_async = matches!(
      mode,
      ContextMode::LazyOnce | ContextMode::AsyncWeak | ContextMode::Eager
    );
    let mut source = ConcatSource::default();
    source.add(RawSource::from(format!(
      "var map = {};\n",
      stringify_map(&map)
    )));

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
    source.add(RawSource::from("\nreturn __webpack_require__(id);\n"));
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
      // return map[req];
      // This is different from webpack, rspack generate map with module id as key
      return req;
    "#,
    ));
    if is_async {
      source.add(RawSource::from("\n});\n"));
    }
    source.add(RawSource::from("\n}\n"));

    source.add(RawSource::from(format!(
      "webpackContext.id = '{}';\n",
      self.id(&compilation.chunk_graph)
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

  async fn build(
    &mut self,
    _build_context: BuildContext<'_>,
  ) -> Result<TWithDiagnosticArray<BuildResult>> {
    self.resolve_dependencies()
  }

  fn code_generation(&self, compilation: &Compilation) -> Result<CodeGenerationResult> {
    let mut code_generation_result = CodeGenerationResult::default();
    code_generation_result
      .runtime_requirements
      .insert(runtime_globals::MODULE);
    code_generation_result
      .runtime_requirements
      .insert(runtime_globals::HAS_OWN_PROPERTY);

    // TODO inject runtime globals by dep size
    code_generation_result
      .runtime_requirements
      .insert(runtime_globals::REQUIRE);
    match self.options.context_options.mode {
      ContextMode::Weak => {
        code_generation_result
          .runtime_requirements
          .insert(runtime_globals::MODULE_FACTORIES);
      }
      ContextMode::AsyncWeak => {
        code_generation_result
          .runtime_requirements
          .insert(runtime_globals::MODULE_FACTORIES);
        code_generation_result
          .runtime_requirements
          .insert(runtime_globals::ENSURE_CHUNK);
      }
      ContextMode::Lazy | ContextMode::LazyOnce => {
        code_generation_result
          .runtime_requirements
          .insert(runtime_globals::ENSURE_CHUNK);
      }
      _ => {}
    }

    code_generation_result.add(
      SourceType::JavaScript,
      GenerationResult::from(AstOrSource::from(self.get_source_string(compilation))),
    );
    Ok(code_generation_result)
  }
}

impl Identifiable for ContextModule {
  fn identifier(&self) -> Identifier {
    self.identifier
  }
}

impl PartialEq for ContextModule {
  fn eq(&self, other: &Self) -> bool {
    self.identifier == other.identifier
  }
}

impl Hash for ContextModule {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    "__rspack_internal__ContextModule".hash(state);
  }
}

impl ContextModule {
  pub fn resolve_dependencies(&self) -> Result<TWithDiagnosticArray<BuildResult>> {
    let mut dependencies = vec![];

    // println!("resolving context module path {}", self.options.resource);

    fn visit_dirs(
      dir: &Path,
      dependencies: &mut Vec<BoxModuleDependency>,
      options: &ContextModuleOptions,
    ) -> Result<()> {
      if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
          let entry = entry?;
          let path = entry.path();
          if path.is_dir() {
            visit_dirs(&path, dependencies, options)?;
          } else {
            let request = path.relative(options.resource.as_path());
            if let Some(request) = request.to_str() {
              let path = format!("./{request}");
              if options.context_options.reg_exp.is_match(&path) {
                dependencies.push(Box::new(ContextElementDependency {
                  id: None,
                  // TODO query
                  request: path.to_string(),
                  user_request: path.to_string(),
                  category: options.context_options.category,
                  context: options.resource.clone(),
                  options: options.context_options.clone(),
                }));
              }
            }
          }
        }
      }
      Ok(())
    }

    visit_dirs(
      Path::new(&self.options.resource),
      &mut dependencies,
      &self.options,
    )?;

    // println!("resolving dependencies for {:?}", dependencies);

    Ok(
      BuildResult {
        dependencies,
        // TODO
        cacheable: false,
        file_dependencies: Default::default(),
        context_dependencies: Default::default(),
        missing_dependencies: Default::default(),
        build_dependencies: Default::default(),
      }
      .with_diagnostic(vec![]),
    )
  }
}

fn create_identifier(options: &ContextModuleOptions) -> Identifier {
  let mut identifier = options.resource.clone();
  if let Some(resource_query) = &options.resource_query {
    identifier.push('|');
    identifier.push_str(resource_query);
  }
  if let Some(resource_fragment) = &options.resource_fragment {
    identifier.push('|');
    identifier.push_str(resource_fragment);
  }
  identifier.into()
}
