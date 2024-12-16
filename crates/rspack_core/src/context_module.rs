use std::sync::Arc;
use std::{borrow::Cow, hash::Hash};

use cow_utils::CowUtils;
use derive_more::Debug;
use indoc::formatdoc;
use itertools::Itertools;
use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsOption, AsPreset, AsVec, Unsupported},
};
use rspack_collections::{Identifiable, Identifier, IdentifierMap};
use rspack_error::{impl_empty_diagnosable_trait, Diagnostic, Result};
use rspack_macros::impl_source_map_config;
use rspack_paths::{ArcPath, Utf8PathBuf};
use rspack_regex::RspackRegex;
use rspack_sources::{BoxSource, ConcatSource, RawStringSource, SourceExt};
use rspack_util::itoa;
use rspack_util::{fx_hash::FxIndexMap, json_stringify, source_map::SourceMapKind};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use swc_core::atoms::Atom;

use crate::{
  block_promise, contextify, get_exports_type_with_strict, impl_module_meta_info,
  module_update_hash, returning_function, to_path, AsyncDependenciesBlock,
  AsyncDependenciesBlockIdentifier, BoxDependency, BuildContext, BuildInfo, BuildMeta,
  BuildMetaDefaultObject, BuildMetaExportsType, BuildResult, ChunkGraph, ChunkGroupOptions,
  CodeGenerationResult, Compilation, ConcatenationScope, ContextElementDependency,
  DependenciesBlock, Dependency, DependencyCategory, DependencyId, DependencyLocation,
  DynamicImportMode, ExportsType, FactoryMeta, FakeNamespaceObjectMode, GroupOptions,
  ImportAttributes, LibIdentOptions, Module, ModuleId, ModuleLayer, ModuleType,
  RealDependencyLocation, Resolve, RuntimeGlobals, RuntimeSpec, SourceType,
};

static WEBPACK_CHUNK_NAME_INDEX_PLACEHOLDER: &str = "[index]";
static WEBPACK_CHUNK_NAME_REQUEST_PLACEHOLDER: &str = "[request]";

#[cacheable]
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

impl From<DynamicImportMode> for ContextMode {
  fn from(value: DynamicImportMode) -> Self {
    match value {
      DynamicImportMode::Lazy => Self::Lazy,
      DynamicImportMode::Weak => Self::AsyncWeak,
      DynamicImportMode::Eager => Self::Eager,
      DynamicImportMode::LazyOnce => Self::LazyOnce,
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

#[cacheable]
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

#[cacheable]
#[derive(Debug, Clone, Copy, Hash, PartialEq, PartialOrd, Ord, Eq)]
pub enum ContextTypePrefix {
  Import,
  Normal,
}

#[cacheable]
#[derive(Debug, Clone)]
pub struct ContextOptions {
  pub mode: ContextMode,
  pub recursive: bool,
  pub reg_exp: Option<RspackRegex>,
  pub include: Option<RspackRegex>,
  pub exclude: Option<RspackRegex>,
  pub category: DependencyCategory,
  pub request: String,
  pub context: String,
  pub namespace_object: ContextNameSpaceObject,
  pub group_options: Option<GroupOptions>,
  pub replaces: Vec<(String, u32, u32)>,
  pub start: u32,
  pub end: u32,
  #[cacheable(with=AsOption<AsVec<AsPreset>>)]
  pub referenced_exports: Option<Vec<Atom>>,
  pub attributes: Option<ImportAttributes>,
}

#[cacheable]
#[derive(Debug, Clone)]
pub struct ContextModuleOptions {
  pub addon: String,
  #[cacheable(with=AsPreset)]
  pub resource: Utf8PathBuf,
  pub resource_query: String,
  pub resource_fragment: String,
  pub context_options: ContextOptions,
  pub layer: Option<ModuleLayer>,
  pub resolve_options: Option<Arc<Resolve>>,
  pub type_prefix: ContextTypePrefix,
}

#[derive(Debug)]
pub enum FakeMapValue {
  Bit(FakeNamespaceObjectMode),
  Map(HashMap<String, FakeNamespaceObjectMode>),
}

pub type ResolveContextModuleDependencies =
  Arc<dyn Fn(ContextModuleOptions) -> Result<Vec<ContextElementDependency>> + Send + Sync>;

#[impl_source_map_config]
#[cacheable]
#[derive(Debug)]
pub struct ContextModule {
  dependencies: Vec<DependencyId>,
  blocks: Vec<AsyncDependenciesBlockIdentifier>,
  identifier: Identifier,
  options: ContextModuleOptions,
  factory_meta: Option<FactoryMeta>,
  build_info: Option<BuildInfo>,
  build_meta: Option<BuildMeta>,
  #[debug(skip)]
  #[cacheable(with=Unsupported)]
  resolve_dependencies: ResolveContextModuleDependencies,
}

impl ContextModule {
  pub fn new(
    resolve_dependencies: ResolveContextModuleDependencies,
    options: ContextModuleOptions,
  ) -> Self {
    Self {
      dependencies: Vec::new(),
      blocks: Vec::new(),
      identifier: create_identifier(&options),
      options,
      factory_meta: None,
      build_info: None,
      build_meta: None,
      source_map_kind: SourceMapKind::empty(),
      resolve_dependencies,
    }
  }

  pub fn get_module_id<'a>(&self, module_ids: &'a IdentifierMap<ModuleId>) -> &'a ModuleId {
    ChunkGraph::get_module_id(module_ids, self.identifier).expect("module id not found")
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
    let module_graph = compilation.get_module_graph();
    let sorted_modules = dependencies
      .filter_map(|dep_id| {
        module_graph
          .module_identifier_by_dependency_id(dep_id)
          .map(|m| (m, dep_id))
      })
      .filter_map(|(m, dep)| {
        ChunkGraph::get_module_id(&compilation.module_ids, *m).map(|id| (id.to_string(), dep))
      })
      .sorted_unstable_by_key(|(module_id, _)| module_id.to_string());
    for (module_id, dep) in sorted_modules {
      let exports_type = get_exports_type_with_strict(
        &compilation.get_module_graph(),
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

  fn get_fake_map_init_statement(&self, fake_map: &FakeMapValue) -> String {
    match fake_map {
      FakeMapValue::Bit(_) => String::new(),
      FakeMapValue::Map(map) => format!("var fakeMap = {}", json_stringify(map)),
    }
  }

  fn get_return_module_object_source(
    &self,
    fake_map: &FakeMapValue,
    async_module: bool,
    fake_map_data_expr: &str,
  ) -> String {
    if let FakeMapValue::Bit(bit) = fake_map {
      return self.get_return(bit, async_module);
    }
    format!(
      "return {}(id, {}{});",
      RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT,
      fake_map_data_expr,
      if async_module { " | 16" } else { "" },
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
      if async_module { " | 16" } else { "" },
    )
  }

  fn get_user_request_map(
    &self,
    dependencies: impl IntoIterator<Item = &DependencyId>,
    compilation: &Compilation,
  ) -> FxIndexMap<String, Option<String>> {
    let module_graph = compilation.get_module_graph();
    let dependencies = dependencies.into_iter();
    dependencies
      .filter_map(|dep_id| {
        let dep = module_graph.dependency_by_id(dep_id).and_then(|dep| {
          if let Some(d) = dep.as_module_dependency() {
            Some(d.user_request().to_string())
          } else {
            dep.as_context_dependency().map(|d| d.request().to_string())
          }
        });
        let module_id = module_graph
          .module_identifier_by_dependency_id(dep_id)
          .and_then(|module| ChunkGraph::get_module_id(&compilation.module_ids, *module))
          .map(|s| s.to_string());
        // module_id could be None in weak mode
        dep.map(|dep| (dep, module_id))
      })
      .sorted_by(|(a, _), (b, _)| a.cmp(b))
      .collect()
  }

  fn get_source_for_empty_async_context(&self, compilation: &Compilation) -> BoxSource {
    RawStringSource::from(formatdoc! {r#"
      function webpackEmptyAsyncContext(req) {{
        // Here Promise.resolve().then() is used instead of new Promise() to prevent
        // uncaught exception popping up in devtools
        return Promise.resolve().then(function() {{
          var e = new Error("Cannot find module '" + req + "'");
          e.code = 'MODULE_NOT_FOUND';
          throw e;
        }});
      }}
      webpackEmptyAsyncContext.keys = {keys};
      webpackEmptyAsyncContext.resolve = webpackEmptyAsyncContext;
      webpackEmptyAsyncContext.id = {id};
      module.exports = webpackEmptyAsyncContext;
      "#,
      keys = returning_function(&compilation.options.output.environment, "[]", ""),
      id = json_stringify(self.get_module_id(&compilation.module_ids))
    })
    .boxed()
  }

  fn get_source_for_empty_context(&self, compilation: &Compilation) -> BoxSource {
    RawStringSource::from(formatdoc! {r#"
      function webpackEmptyContext(req) {{
        var e = new Error("Cannot find module '" + req + "'");
        e.code = 'MODULE_NOT_FOUND';
        throw e;
      }}
      webpackEmptyContext.keys = {keys};
      webpackEmptyContext.resolve = webpackEmptyContext;
      webpackEmptyContext.id = {id};
      module.exports = webpackEmptyContext;
      "#,
      keys = returning_function(&compilation.options.output.environment, "[]", ""),
      id = json_stringify(self.get_module_id(&compilation.module_ids))
    })
    .boxed()
  }

  #[inline]
  fn get_source_string(
    &self,
    compilation: &Compilation,
    code_gen_result: &mut CodeGenerationResult,
  ) -> BoxSource {
    match self.options.context_options.mode {
      ContextMode::Lazy => {
        if !self.get_blocks().is_empty() {
          self.get_lazy_source(compilation)
        } else {
          self.get_source_for_empty_async_context(compilation)
        }
      }
      ContextMode::Eager => {
        if !self.get_dependencies().is_empty() {
          self.get_eager_source(compilation)
        } else {
          self.get_source_for_empty_async_context(compilation)
        }
      }
      ContextMode::LazyOnce => {
        if let Some(block) = self.get_blocks().first() {
          self.get_lazy_once_source(compilation, block, code_gen_result)
        } else {
          self.get_source_for_empty_async_context(compilation)
        }
      }
      ContextMode::AsyncWeak => {
        if !self.get_dependencies().is_empty() {
          self.get_async_weak_source(compilation)
        } else {
          self.get_source_for_empty_async_context(compilation)
        }
      }
      ContextMode::Weak => {
        if !self.get_dependencies().is_empty() {
          self.get_sync_weak_source(compilation)
        } else {
          self.get_source_for_empty_context(compilation)
        }
      }
      ContextMode::Sync => {
        if !self.get_dependencies().is_empty() {
          self.get_sync_source(compilation)
        } else {
          self.get_source_for_empty_context(compilation)
        }
      }
    }
  }

  fn get_lazy_source(&self, compilation: &Compilation) -> BoxSource {
    let module_graph = compilation.get_module_graph();
    let blocks = self
      .get_blocks()
      .iter()
      .filter_map(|b| module_graph.block_by_id(b));
    let block_and_first_dependency_list = blocks
      .clone()
      .filter_map(|b| b.get_dependencies().first().map(|d| (b, d)));
    let first_dependencies = block_and_first_dependency_list.clone().map(|(_, d)| d);
    let mut has_multiple_or_no_chunks = false;
    let mut has_no_chunk = true;
    let fake_map = self.get_fake_map(first_dependencies, compilation);
    let has_fake_map = matches!(fake_map, FakeMapValue::Map(_));
    let mut items = block_and_first_dependency_list
      .filter_map(|(b, d)| {
        let chunks = compilation
          .chunk_graph
          .get_block_chunk_group(&b.identifier(), &compilation.chunk_group_by_ukey)
          .map(|chunk_group| {
            let chunks = &chunk_group.chunks;
            if !chunks.is_empty() {
              has_no_chunk = false;
            }
            if chunks.len() != 1 {
              has_multiple_or_no_chunks = true;
            }
            chunks
          });
        let user_request = compilation
          .get_module_graph()
          .dependency_by_id(d)
          .and_then(|dep| {
            dep
              .as_module_dependency()
              .map(|d| d.user_request().to_string())
              .or_else(|| dep.as_context_dependency().map(|d| d.request().to_string()))
          })?;
        let module_id = module_graph
          .module_identifier_by_dependency_id(d)
          .and_then(|m| ChunkGraph::get_module_id(&compilation.module_ids, *m))?;
        Some((chunks, user_request, module_id.to_string()))
      })
      .collect::<Vec<_>>();
    let short_mode = has_no_chunk && !has_fake_map;
    items.sort_unstable_by(|a, b| a.1.cmp(&b.1));
    let map = items
      .into_iter()
      .map(|(chunks, user_request, module_id)| {
        let value = if short_mode {
          serde_json::Value::String(module_id)
        } else {
          let second = if let FakeMapValue::Map(fake_map) = &fake_map {
            Some(fake_map[&module_id])
          } else {
            None
          };
          let mut array_start = vec![serde_json::json!(module_id)];
          if let Some(second) = second {
            array_start.push(serde_json::json!(second.bits()));
          }
          if let Some(chunks) = chunks {
            array_start.extend(chunks.iter().map(|c| {
              let chunk_id = compilation
                .chunk_by_ukey
                .expect_get(c)
                .id(&compilation.chunk_ids)
                .expect("should have chunk id in code generation");
              serde_json::json!(chunk_id)
            }))
          }
          serde_json::json!(array_start)
        };
        (user_request, value)
      })
      .collect::<HashMap<_, _>>();
    let chunks_start_position = if has_fake_map { 2 } else { 1 };
    let request_prefix = if has_no_chunk {
      "Promise.resolve()".to_string()
    } else if has_multiple_or_no_chunks {
      format!(
        "Promise.all(ids.slice({chunks_start_position}).map({}))",
        RuntimeGlobals::ENSURE_CHUNK
      )
    } else {
      format!(
        "{}(ids[{}])",
        RuntimeGlobals::ENSURE_CHUNK,
        itoa!(chunks_start_position)
      )
    };
    let return_module_object = self.get_return_module_object_source(
      &fake_map,
      true,
      if short_mode { "invalid" } else { "ids[1]" },
    );
    let mut source = ConcatSource::default();
    let webpack_async_context = if has_no_chunk {
      formatdoc! {r#"
        function webpackAsyncContext(req) {{
          return Promise.resolve().then(function() {{
            if(!{}(map, req)) {{
              var e = new Error("Cannot find module '" + req + "'");
              e.code = 'MODULE_NOT_FOUND';
              throw e;
            }}

            {}
            {return_module_object}
          }});
        }}
        "#,
        RuntimeGlobals::HAS_OWN_PROPERTY,
        if short_mode {
          "var id = map[req];"
        } else {
          "var ids = map[req], id = ids[0];"
        }
      }
    } else {
      formatdoc! {r#"
        function webpackAsyncContext(req) {{
          if(!{}(map, req)) {{
            return Promise.resolve().then(function() {{
              var e = new Error("Cannot find module '" + req + "'");
              e.code = 'MODULE_NOT_FOUND';
              throw e;
            }});
          }}

          var ids = map[req], id = ids[0];
          return {request_prefix}.then(function() {{
            {return_module_object}
          }});
        }}
        "#,
        RuntimeGlobals::HAS_OWN_PROPERTY,
      }
    };
    source.add(RawStringSource::from(formatdoc! {r#"
      var map = {map};
      {webpack_async_context}
      webpackAsyncContext.keys = {keys};
      webpackAsyncContext.id = {id};
      module.exports = webpackAsyncContext;
      "#,
      map = json_stringify(&map),
      keys = returning_function(&compilation.options.output.environment, "Object.keys(map)", ""),
      id = json_stringify(self.get_module_id(&compilation.module_ids))
    }));
    source.boxed()
  }

  fn get_lazy_once_source(
    &self,
    compilation: &Compilation,
    block_id: &AsyncDependenciesBlockIdentifier,
    code_gen_result: &mut CodeGenerationResult,
  ) -> BoxSource {
    let mg = compilation.get_module_graph();
    let block = mg.block_by_id_expect(block_id);
    let dependencies = block.get_dependencies();
    let promise = block_promise(
      Some(block_id),
      &mut code_gen_result.runtime_requirements,
      compilation,
      "lazy-once context",
    );
    let map = self.get_user_request_map(dependencies, compilation);
    let fake_map = self.get_fake_map(dependencies, compilation);
    let then_function = if !matches!(
      fake_map,
      FakeMapValue::Bit(FakeNamespaceObjectMode::NAMESPACE)
    ) {
      formatdoc! {r#"
        function(id) {{
          {}
        }}
        "#,
        self.get_return_module_object_source(&fake_map, true, "fakeMap[id]"),
      }
    } else {
      RuntimeGlobals::REQUIRE.name().to_string()
    };
    let source = formatdoc! {r#"
      var map = {map};
      {fake_map_init_statement}

      function webpackAsyncContext(req) {{
        return webpackAsyncContextResolve(req).then({then_function});
      }}
      function webpackAsyncContextResolve(req) {{
        return {promise}.then(function() {{
          if(!{has_own_property}(map, req)) {{
            var e = new Error("Cannot find module '" + req + "'");
            e.code = 'MODULE_NOT_FOUND';
            throw e;
          }}
          return map[req];
        }})
      }}
      webpackAsyncContext.keys = {keys};
      webpackAsyncContext.resolve = webpackAsyncContextResolve;
      webpackAsyncContext.id = {id};
      module.exports = webpackAsyncContext;
      "#,
      map = json_stringify(&map),
      fake_map_init_statement = self.get_fake_map_init_statement(&fake_map),
      has_own_property = RuntimeGlobals::HAS_OWN_PROPERTY,
      keys = returning_function(&compilation.options.output.environment, "Object.keys(map)", ""),
      id = json_stringify(self.get_module_id(&compilation.module_ids))
    };
    RawStringSource::from(source).boxed()
  }

  fn get_async_weak_source(&self, compilation: &Compilation) -> BoxSource {
    let dependencies = self.get_dependencies();
    let map = self.get_user_request_map(dependencies, compilation);
    let fake_map = self.get_fake_map(dependencies, compilation);
    let return_module_object = self.get_return_module_object_source(&fake_map, true, "fakeMap[id]");
    let source = formatdoc! {r#"
      var map = {map};
      {fake_map_init_statement}

      function webpackAsyncContext(req) {{
        return webpackAsyncContextResolve(req).then(function(id) {{
          if(!{module_factories}[id]) {{
            var e = new Error("Module '" + req + "' ('" + id + "') is not available (weak dependency)");
            e.code = 'MODULE_NOT_FOUND';
            throw e;
          }}
          {return_module_object}
        }});
      }}
      function webpackAsyncContextResolve(req) {{
        // Here Promise.resolve().then() is used instead of new Promise() to prevent
        // uncaught exception popping up in devtools
        return Promise.resolve().then(function() {{
          if(!{has_own_property}(map, req)) {{
            var e = new Error("Cannot find module '" + req + "'");
            e.code = 'MODULE_NOT_FOUND';
            throw e;
          }}
          return map[req];
        }})
      }}
      webpackAsyncContext.keys = {keys};
      webpackAsyncContext.resolve = webpackAsyncContextResolve;
      webpackAsyncContext.id = {id};
      module.exports = webpackAsyncContext;
      "#,
      map = json_stringify(&map),
      fake_map_init_statement = self.get_fake_map_init_statement(&fake_map),
      module_factories = RuntimeGlobals::MODULE_FACTORIES,
      has_own_property = RuntimeGlobals::HAS_OWN_PROPERTY,
      keys = returning_function(&compilation.options.output.environment, "Object.keys(map)", ""),
      id = json_stringify(self.get_module_id(&compilation.module_ids))
    };
    RawStringSource::from(source).boxed()
  }

  fn get_sync_weak_source(&self, compilation: &Compilation) -> BoxSource {
    let dependencies = self.get_dependencies();
    let map = self.get_user_request_map(dependencies, compilation);
    let fake_map = self.get_fake_map(dependencies, compilation);
    let return_module_object = self.get_return_module_object_source(&fake_map, true, "fakeMap[id]");
    let source = formatdoc! {r#"
      var map = {map};
      {fake_map_init_statement}

      function webpackContext(req) {{
        var id = webpackContextResolve(req);
        if(!{module_factories}[id]) {{
          var e = new Error("Module '" + req + "' ('" + id + "') is not available (weak dependency)");
          e.code = 'MODULE_NOT_FOUND';
          throw e;
        }}
        {return_module_object}
      }}
      function webpackContextResolve(req) {{
        if(!{has_own_property}(map, req)) {{
          var e = new Error("Cannot find module '" + req + "'");
          e.code = 'MODULE_NOT_FOUND';
          throw e;
        }}
        return map[req];
      }}
      webpackContext.keys = {keys};
      webpackContext.resolve = webpackContextResolve;
      webpackContext.id = {id};
      module.exports = webpackContext;
      "#,
      map = json_stringify(&map),
      fake_map_init_statement = self.get_fake_map_init_statement(&fake_map),
      module_factories = RuntimeGlobals::MODULE_FACTORIES,
      has_own_property = RuntimeGlobals::HAS_OWN_PROPERTY,
      keys = returning_function(&compilation.options.output.environment, "Object.keys(map)", ""),
      id = json_stringify(self.get_module_id(&compilation.module_ids))
    };
    RawStringSource::from(source).boxed()
  }

  fn get_eager_source(&self, compilation: &Compilation) -> BoxSource {
    let dependencies = self.get_dependencies();
    let map = self.get_user_request_map(dependencies, compilation);
    let fake_map = self.get_fake_map(dependencies, compilation);
    let then_function = if !matches!(
      fake_map,
      FakeMapValue::Bit(FakeNamespaceObjectMode::NAMESPACE)
    ) {
      formatdoc! {r#"
        function(id) {{
          {}
        }}
        "#,
        self.get_return_module_object_source(&fake_map, true, "fakeMap[id]"),
      }
    } else {
      RuntimeGlobals::REQUIRE.name().to_string()
    };
    let source = formatdoc! {r#"
      var map = {map};
      {fake_map_init_statement}

      function webpackAsyncContext(req) {{
        return webpackAsyncContextResolve(req).then({then_function});
      }}
      function webpackAsyncContextResolve(req) {{
        // Here Promise.resolve().then() is used instead of new Promise() to prevent
        // uncaught exception popping up in devtools
        return Promise.resolve().then(function() {{
          if(!{has_own_property}(map, req)) {{
            var e = new Error("Cannot find module '" + req + "'");
            e.code = 'MODULE_NOT_FOUND';
            throw e;
          }}
          return map[req];
        }})
      }}
      webpackAsyncContext.keys = {keys};
      webpackAsyncContext.resolve = webpackAsyncContextResolve;
      webpackAsyncContext.id = {id};
      module.exports = webpackAsyncContext;
      "#,
      map = json_stringify(&map),
      fake_map_init_statement = self.get_fake_map_init_statement(&fake_map),
      has_own_property = RuntimeGlobals::HAS_OWN_PROPERTY,
      keys = returning_function(&compilation.options.output.environment, "Object.keys(map)", ""),
      id = json_stringify(self.get_module_id(&compilation.module_ids))
    };
    RawStringSource::from(source).boxed()
  }

  fn get_sync_source(&self, compilation: &Compilation) -> BoxSource {
    let dependencies = self.get_dependencies();
    let map = self.get_user_request_map(dependencies, compilation);
    let fake_map = self.get_fake_map(dependencies, compilation);
    let return_module_object =
      self.get_return_module_object_source(&fake_map, false, "fakeMap[id]");
    let source = formatdoc! {r#"
      var map = {map};
      {fake_map_init_statement}

      function webpackContext(req) {{
        var id = webpackContextResolve(req);
        {return_module_object}
      }}
      function webpackContextResolve(req) {{
        if(!{has_own_property}(map, req)) {{
          var e = new Error("Cannot find module '" + req + "'");
          e.code = 'MODULE_NOT_FOUND';
          throw e;
        }}
        return map[req];
      }}
      webpackContext.keys = function webpackContextKeys() {{
        return Object.keys(map);
      }};
      webpackContext.resolve = webpackContextResolve;
      module.exports = webpackContext;
      webpackContext.id = {id};
      "#,
      map = json_stringify(&map),
      fake_map_init_statement = self.get_fake_map_init_statement(&fake_map),
      has_own_property = RuntimeGlobals::HAS_OWN_PROPERTY,
      id = json_stringify(self.get_module_id(&compilation.module_ids))
    };
    RawStringSource::from(source).boxed()
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

  fn remove_dependency_id(&mut self, dependency: DependencyId) {
    self.dependencies.retain(|d| d != &dependency)
  }

  fn get_dependencies(&self) -> &[DependencyId] {
    &self.dependencies
  }
}

#[cacheable_dyn]
#[async_trait::async_trait]
impl Module for ContextModule {
  impl_module_meta_info!();

  fn module_type(&self) -> &ModuleType {
    &ModuleType::JsAuto
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

  fn size(
    &self,
    _source_type: Option<&crate::SourceType>,
    _compilation: Option<&Compilation>,
  ) -> f64 {
    160.0
  }

  fn lib_ident(&self, options: LibIdentOptions) -> Option<Cow<str>> {
    let mut id = String::new();
    if let Some(layer) = &self.options.layer {
      id += "(";
      id += layer;
      id += ")/";
    }
    id += &contextify(options.context, self.options.resource.as_str());
    id.push(' ');
    id.push_str(self.options.context_options.mode.as_str());
    if self.options.context_options.recursive {
      id.push_str(" recursive");
    }
    if let Some(regexp) = &self.options.context_options.reg_exp {
      id.push(' ');
      id.push_str(&regexp.to_pretty_string(true));
    }
    Some(Cow::Owned(id))
  }

  async fn build(
    &mut self,
    _build_context: BuildContext,
    _: Option<&Compilation>,
  ) -> Result<BuildResult> {
    let resolve_dependencies = &self.resolve_dependencies;
    let context_element_dependencies = resolve_dependencies(self.options.clone())?;

    let mut dependencies: Vec<BoxDependency> = vec![];
    let mut blocks = vec![];
    if matches!(self.options.context_options.mode, ContextMode::LazyOnce)
      && !context_element_dependencies.is_empty()
    {
      let loc = DependencyLocation::Real(RealDependencyLocation::new(
        (
          self.options.context_options.start,
          self.options.context_options.end,
        )
          .into(),
        None,
      ));
      let mut block = AsyncDependenciesBlock::new(
        (*self.identifier).into(),
        Some(loc),
        None,
        context_element_dependencies
          .into_iter()
          .map(|dep| Box::new(dep) as Box<dyn Dependency>)
          .collect(),
        None,
      );
      if let Some(group_options) = &self.options.context_options.group_options {
        block.set_group_options(group_options.clone());
      }
      blocks.push(Box::new(block));
    } else if matches!(self.options.context_options.mode, ContextMode::Lazy) {
      let mut index = 0;
      // TODO(shulaoda): add loc for ContextElementDependency and AsyncDependenciesBlock
      for context_element_dependency in context_element_dependencies {
        let group_options = self
          .options
          .context_options
          .group_options
          .as_ref()
          .and_then(|g| g.normal_options());
        let name = group_options
          .and_then(|group_options| group_options.name.as_ref())
          .map(|name| {
            let name = if !(name.contains(WEBPACK_CHUNK_NAME_INDEX_PLACEHOLDER)
              || name.contains(WEBPACK_CHUNK_NAME_REQUEST_PLACEHOLDER))
            {
              Cow::Owned(format!("{name}[index]"))
            } else {
              Cow::Borrowed(name)
            };

            let name = name.cow_replace(WEBPACK_CHUNK_NAME_INDEX_PLACEHOLDER, &index.to_string());
            let name = name.cow_replace(
              WEBPACK_CHUNK_NAME_REQUEST_PLACEHOLDER,
              &to_path(&context_element_dependency.user_request),
            );

            index += 1;
            name.into_owned()
          });
        let preload_order = group_options.and_then(|o| o.preload_order);
        let prefetch_order = group_options.and_then(|o| o.prefetch_order);
        let fetch_priority = group_options.and_then(|o| o.fetch_priority);
        let mut block = AsyncDependenciesBlock::new(
          (*self.identifier).into(),
          None,
          Some(&context_element_dependency.user_request.clone()),
          vec![Box::new(context_element_dependency)],
          Some(self.options.context_options.request.clone()),
        );
        block.set_group_options(GroupOptions::ChunkGroup(ChunkGroupOptions::new(
          name,
          preload_order,
          prefetch_order,
          fetch_priority,
        )));
        blocks.push(Box::new(block));
      }
    } else {
      dependencies = context_element_dependencies
        .into_iter()
        .map(|d| Box::new(d) as BoxDependency)
        .collect();
    }

    let mut context_dependencies: HashSet<ArcPath> = Default::default();
    context_dependencies.insert(self.options.resource.as_std_path().into());

    let build_info = BuildInfo {
      context_dependencies,
      ..Default::default()
    };

    Ok(BuildResult {
      build_info,
      build_meta: BuildMeta {
        exports_type: BuildMetaExportsType::Default,
        default_object: BuildMetaDefaultObject::RedirectWarn { ignore: false },
        ..Default::default()
      },
      dependencies,
      blocks,
      optimization_bailouts: vec![],
    })
  }

  #[tracing::instrument(name = "ContextModule::code_generation", skip_all, fields(identifier = ?self.identifier()))]
  fn code_generation(
    &self,
    compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
    _: Option<ConcatenationScope>,
  ) -> Result<CodeGenerationResult> {
    let mut code_generation_result = CodeGenerationResult::default();
    let source = self.get_source_string(compilation, &mut code_generation_result);
    code_generation_result.add(SourceType::JavaScript, source);
    let mut all_deps = self.get_dependencies().to_vec();
    let module_graph = compilation.get_module_graph();
    for block in self.get_blocks() {
      let block = module_graph
        .block_by_id(block)
        .expect("should have block in ContextModule code_generation");
      all_deps.extend(block.get_dependencies());
    }
    code_generation_result
      .runtime_requirements
      .insert(RuntimeGlobals::MODULE);
    code_generation_result
      .runtime_requirements
      .insert(RuntimeGlobals::HAS_OWN_PROPERTY);
    if !all_deps.is_empty() {
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
      let fake_map = self.get_fake_map(all_deps.iter(), compilation);
      if !matches!(fake_map, FakeMapValue::Bit(bit) if bit == FakeNamespaceObjectMode::NAMESPACE) {
        code_generation_result
          .runtime_requirements
          .insert(RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT);
      }
    }
    Ok(code_generation_result)
  }

  fn update_hash(
    &self,
    hasher: &mut dyn std::hash::Hasher,
    compilation: &Compilation,
    runtime: Option<&RuntimeSpec>,
  ) -> Result<()> {
    module_update_hash(self, hasher, compilation, runtime);
    Ok(())
  }
}

impl_empty_diagnosable_trait!(ContextModule);

impl Identifiable for ContextModule {
  fn identifier(&self) -> Identifier {
    self.identifier
  }
}

fn create_identifier(options: &ContextModuleOptions) -> Identifier {
  let mut id = options.resource.as_str().to_owned();
  if !options.resource_query.is_empty() {
    id += "|";
    id += &options.resource_query;
  }
  if !options.resource_fragment.is_empty() {
    id += "|";
    id += &options.resource_fragment;
  }
  id += "|";
  id += options.context_options.mode.as_str();
  if !options.context_options.recursive {
    id += "|nonrecursive";
  }
  if !options.addon.is_empty() {
    id += "|";
    id += &options.addon;
  }
  if let Some(regexp) = &options.context_options.reg_exp {
    id += "|";
    id += &regexp.to_pretty_string(false);
  }
  if let Some(include) = &options.context_options.include {
    id += "|include: ";
    id += &include.to_source_string();
  }
  if let Some(exclude) = &options.context_options.exclude {
    id += "|exclude: ";
    id += &exclude.to_source_string();
  }
  if let Some(exports) = &options.context_options.referenced_exports {
    id += "|referencedExports: ";
    id += &format!(
      "[{}]",
      exports.iter().map(|x| format!(r#""{x}""#)).join(",")
    );
  }

  if let Some(GroupOptions::ChunkGroup(group)) = &options.context_options.group_options {
    if let Some(chunk_name) = &group.name {
      id += "|chunkName: ";
      id += chunk_name;
    }
    id += "|groupOptions: {";
    if let Some(o) = group.prefetch_order {
      id.push_str(&format!("prefetchOrder: {},", o));
    }
    if let Some(o) = group.preload_order {
      id.push_str(&format!("preloadOrder: {},", o));
    }
    if let Some(o) = group.fetch_priority {
      id.push_str(&format!("fetchPriority: {},", o));
    }
    id += "}";
  }
  id += match options.context_options.namespace_object {
    ContextNameSpaceObject::Strict => "|strict namespace object",
    ContextNameSpaceObject::Bool(true) => "|namespace object",
    _ => "",
  };
  if let Some(layer) = &options.layer {
    id += "|layer: ";
    id += layer;
  }
  id.into()
}
