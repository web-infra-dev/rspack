use std::{borrow::Cow, hash::Hash, sync::Arc};

use cow_utils::CowUtils;
use derive_more::Debug;
use futures::future::BoxFuture;
use indoc::formatdoc;
use itertools::Itertools;
use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsOption, AsPreset, AsVec, Unsupported},
};
use rspack_collections::{Identifiable, Identifier};
use rspack_error::{Result, impl_empty_diagnosable_trait};
use rspack_hash::{RspackHash, RspackHashDigest};
use rspack_macros::impl_source_map_config;
use rspack_paths::{ArcPathSet, Utf8PathBuf};
use rspack_regex::RspackRegex;
use rspack_sources::{BoxSource, OriginalSource, RawStringSource, SourceExt};
use rspack_util::{
  fx_hash::FxIndexMap,
  identifier::make_paths_relative,
  itoa, json_stringify,
  source_map::{ModuleSourceMapConfig, SourceMapKind},
};
use rustc_hash::FxHashMap as HashMap;
use swc_core::atoms::Atom;

use crate::{
  AsyncDependenciesBlock, AsyncDependenciesBlockIdentifier, BoxDependency, BuildContext, BuildInfo,
  BuildMeta, BuildMetaDefaultObject, BuildMetaExportsType, BuildResult, ChunkGraph,
  ChunkGroupOptions, CodeGenerationResult, Compilation, ContextElementDependency,
  DependenciesBlock, Dependency, DependencyCategory, DependencyId, DependencyLocation,
  DynamicImportMode, ExportsType, FactoryMeta, FakeNamespaceObjectMode, GroupOptions,
  ImportAttributes, LibIdentOptions, Module, ModuleArgument, ModuleCodeGenerationContext,
  ModuleCodegenRuntimeTemplate, ModuleGraph, ModuleId, ModuleIdsArtifact, ModuleLayer, ModuleType,
  RealDependencyLocation, Resolve, RuntimeGlobals, RuntimeSpec, SourceType, contextify,
  get_exports_type_with_strict, impl_module_meta_info, module_update_hash, to_path,
};

static CHUNK_NAME_INDEX_PLACEHOLDER: &str = "[index]";
static CHUNK_NAME_REQUEST_PLACEHOLDER: &str = "[request]";

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
  #[cacheable(with=AsOption<AsVec<AsVec<AsPreset>>>)]
  pub referenced_exports: Option<Vec<Vec<Atom>>>,
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

pub type ResolveContextModuleDependencies = Arc<
  dyn Fn(ContextModuleOptions) -> BoxFuture<'static, Result<Vec<ContextElementDependency>>>
    + Send
    + Sync,
>;

#[impl_source_map_config]
#[cacheable]
#[derive(Debug)]
pub struct ContextModule {
  dependencies: Vec<DependencyId>,
  blocks: Vec<AsyncDependenciesBlockIdentifier>,
  identifier: Identifier,
  options: ContextModuleOptions,
  factory_meta: Option<FactoryMeta>,
  build_info: BuildInfo,
  build_meta: BuildMeta,
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
      identifier: create_identifier(&options, None),
      options,
      factory_meta: None,
      build_info: Default::default(),
      build_meta: BuildMeta {
        exports_type: BuildMetaExportsType::Default,
        default_object: BuildMetaDefaultObject::RedirectWarn { ignore: false },
        ..Default::default()
      },
      source_map_kind: SourceMapKind::empty(),
      resolve_dependencies,
    }
  }

  pub fn get_module_id<'a>(&self, module_ids: &'a ModuleIdsArtifact) -> &'a ModuleId {
    ChunkGraph::get_module_id(module_ids, self.identifier).expect("module id not found")
  }

  pub fn get_context_options(&self) -> &ContextOptions {
    &self.options.context_options
  }

  fn get_fake_map<'a>(
    &self,
    dependencies: impl IntoIterator<Item = &'a DependencyId>,
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
        ChunkGraph::get_module_id(&compilation.module_ids_artifact, *m)
          .map(|id| (id.to_string(), dep))
      })
      .sorted_unstable_by_key(|(module_id, _)| module_id.clone());
    for (module_id, dep) in sorted_modules {
      let exports_type = get_exports_type_with_strict(
        compilation.get_module_graph(),
        &compilation.module_graph_cache_artifact,
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
    runtime_template: &mut ModuleCodegenRuntimeTemplate,
  ) -> String {
    if let FakeMapValue::Bit(bit) = fake_map {
      return self.get_return(*bit, async_module, runtime_template);
    }
    format!(
      "return {}(id, {}{});",
      runtime_template.render_runtime_globals(&RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT),
      fake_map_data_expr,
      if async_module { " | 16" } else { "" },
    )
  }

  fn get_return(
    &self,
    fake_map_bit: FakeNamespaceObjectMode,
    async_module: bool,
    runtime_template: &mut ModuleCodegenRuntimeTemplate,
  ) -> String {
    if fake_map_bit == FakeNamespaceObjectMode::NAMESPACE {
      return format!(
        "return {}(id);",
        runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE)
      );
    }
    format!(
      "return {}(id, {}{});",
      runtime_template.render_runtime_globals(&RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT),
      fake_map_bit,
      if async_module { " | 16" } else { "" },
    )
  }

  fn get_user_request_map<'a>(
    &self,
    dependencies: impl IntoIterator<Item = &'a DependencyId>,
    compilation: &Compilation,
  ) -> FxIndexMap<String, Option<String>> {
    let module_graph = compilation.get_module_graph();
    let dependencies = dependencies.into_iter();
    dependencies
      .filter_map(|dep_id| {
        let dependency = module_graph.dependency_by_id(dep_id);
        let dep = if let Some(d) = dependency.as_module_dependency() {
          Some(d.user_request().to_string())
        } else {
          dependency
            .as_context_dependency()
            .map(|d| d.request().to_string())
        };
        let module_id = module_graph
          .module_identifier_by_dependency_id(dep_id)
          .and_then(|module| ChunkGraph::get_module_id(&compilation.module_ids_artifact, *module))
          .map(|s| s.to_string());
        // module_id could be None in weak mode
        dep.map(|dep| (dep, module_id))
      })
      .sorted_by(|(a, _), (b, _)| a.cmp(b))
      .collect()
  }

  fn get_source_for_empty_async_context(
    &self,
    compilation: &Compilation,
    runtime_template: &mut ModuleCodegenRuntimeTemplate,
  ) -> String {
    formatdoc! {r#"
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
      {module}.exports = webpackEmptyAsyncContext;
      "#,
      module = runtime_template.render_module_argument(ModuleArgument::Module),
      keys = runtime_template.returning_function("[]", ""),
      id = json_stringify(self.get_module_id(&compilation.module_ids_artifact))
    }
  }

  fn get_source_for_empty_context(
    &self,
    compilation: &Compilation,
    runtime_template: &mut ModuleCodegenRuntimeTemplate,
  ) -> String {
    formatdoc! {r#"
      function webpackEmptyContext(req) {{
        var e = new Error("Cannot find module '" + req + "'");
        e.code = 'MODULE_NOT_FOUND';
        throw e;
      }}
      webpackEmptyContext.keys = {keys};
      webpackEmptyContext.resolve = webpackEmptyContext;
      webpackEmptyContext.id = {id};
      {module}.exports = webpackEmptyContext;
      "#,
      module = runtime_template.render_module_argument(ModuleArgument::Module),
      keys = runtime_template.returning_function("[]", ""),
      id = json_stringify(self.get_module_id(&compilation.module_ids_artifact))
    }
  }

  #[inline]
  fn get_source_string(
    &self,
    compilation: &Compilation,
    runtime_template: &mut ModuleCodegenRuntimeTemplate,
  ) -> String {
    match self.options.context_options.mode {
      ContextMode::Lazy => {
        if !self.get_blocks().is_empty() {
          self.get_lazy_source(compilation, runtime_template)
        } else {
          self.get_source_for_empty_async_context(compilation, runtime_template)
        }
      }
      ContextMode::Eager => {
        if !self.get_dependencies().is_empty() {
          self.get_eager_source(compilation, runtime_template)
        } else {
          self.get_source_for_empty_async_context(compilation, runtime_template)
        }
      }
      ContextMode::LazyOnce => {
        if let Some(block) = self.get_blocks().first() {
          self.get_lazy_once_source(compilation, *block, runtime_template)
        } else {
          self.get_source_for_empty_async_context(compilation, runtime_template)
        }
      }
      ContextMode::AsyncWeak => {
        if !self.get_dependencies().is_empty() {
          self.get_async_weak_source(compilation, runtime_template)
        } else {
          self.get_source_for_empty_async_context(compilation, runtime_template)
        }
      }
      ContextMode::Weak => {
        if !self.get_dependencies().is_empty() {
          self.get_sync_weak_source(compilation, runtime_template)
        } else {
          self.get_source_for_empty_context(compilation, runtime_template)
        }
      }
      ContextMode::Sync => {
        if !self.get_dependencies().is_empty() {
          self.get_sync_source(compilation, runtime_template)
        } else {
          self.get_source_for_empty_context(compilation, runtime_template)
        }
      }
    }
  }

  fn get_lazy_source(
    &self,
    compilation: &Compilation,
    runtime_template: &mut ModuleCodegenRuntimeTemplate,
  ) -> String {
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
          })
          .or_else(|| {
            has_multiple_or_no_chunks = true;
            None
          });
        let dependency = compilation.get_module_graph().dependency_by_id(d);
        let user_request = dependency
          .as_module_dependency()
          .map(|d| d.user_request().to_string())
          .or_else(|| {
            dependency
              .as_context_dependency()
              .map(|d| d.request().to_string())
          })?;
        let module_id = module_graph
          .module_identifier_by_dependency_id(d)
          .and_then(|m| ChunkGraph::get_module_id(&compilation.module_ids_artifact, *m))?;
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
                .id()
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
        runtime_template.render_runtime_globals(&RuntimeGlobals::ENSURE_CHUNK)
      )
    } else {
      let mut chunks_start_position_buffer = itoa::Buffer::new();
      let chunks_start_position_str = chunks_start_position_buffer.format(chunks_start_position);
      format!(
        "{}(ids[{}])",
        runtime_template.render_runtime_globals(&RuntimeGlobals::ENSURE_CHUNK),
        chunks_start_position_str
      )
    };
    let return_module_object = self.get_return_module_object_source(
      &fake_map,
      true,
      if short_mode { "invalid" } else { "ids[1]" },
      runtime_template,
    );
    let async_context = if has_no_chunk {
      formatdoc! {r#"
        function __rspack_async_context(req) {{
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
        runtime_template.render_runtime_globals(&RuntimeGlobals::HAS_OWN_PROPERTY),
        if short_mode {
          "var id = map[req];"
        } else {
          "var ids = map[req], id = ids[0];"
        }
      }
    } else {
      formatdoc! {r#"
        function __rspack_async_context(req) {{
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
        runtime_template.render_runtime_globals(&RuntimeGlobals::HAS_OWN_PROPERTY),
      }
    };
    formatdoc! {r#"
      var map = {map};
      {async_context}
      __rspack_async_context.keys = {keys};
      __rspack_async_context.id = {id};
      {module}.exports = __rspack_async_context;
      "#,
      module = runtime_template.render_module_argument(ModuleArgument::Module),
      map = json_stringify(&map),
      keys = runtime_template.returning_function("Object.keys(map)", ""),
      id = json_stringify(self.get_module_id(&compilation.module_ids_artifact))
    }
  }

  fn get_lazy_once_source(
    &self,
    compilation: &Compilation,
    block_id: AsyncDependenciesBlockIdentifier,
    runtime_template: &mut ModuleCodegenRuntimeTemplate,
  ) -> String {
    let mg = compilation.get_module_graph();
    let block = mg.block_by_id_expect(&block_id);
    let dependencies = block.get_dependencies();
    let promise = runtime_template.block_promise(Some(&block_id), compilation, "lazy-once context");
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
        self.get_return_module_object_source(&fake_map, true, "fakeMap[id]", runtime_template),
      }
    } else {
      runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE)
    };
    formatdoc! {r#"
      var map = {map};
      {fake_map_init_statement}

      function __rspack_async_context(req) {{
        return __rspack_async_context_resolve(req).then({then_function});
      }}
      function __rspack_async_context_resolve(req) {{
        return {promise}.then(function() {{
          if(!{has_own_property}(map, req)) {{
            var e = new Error("Cannot find module '" + req + "'");
            e.code = 'MODULE_NOT_FOUND';
            throw e;
          }}
          return map[req];
        }})
      }}
      __rspack_async_context.keys = {keys};
      __rspack_async_context.resolve = __rspack_async_context_resolve;
      __rspack_async_context.id = {id};
      {module}.exports = __rspack_async_context;
      "#,
      module = runtime_template.render_module_argument(ModuleArgument::Module),
      map = json_stringify(&map),
      fake_map_init_statement = self.get_fake_map_init_statement(&fake_map),
      has_own_property = runtime_template.render_runtime_globals(&RuntimeGlobals::HAS_OWN_PROPERTY),
      keys = runtime_template.returning_function("Object.keys(map)", ""),
      id = json_stringify(self.get_module_id(&compilation.module_ids_artifact))
    }
  }

  fn get_async_weak_source(
    &self,
    compilation: &Compilation,
    runtime_template: &mut ModuleCodegenRuntimeTemplate,
  ) -> String {
    let dependencies = self.get_dependencies();
    let map = self.get_user_request_map(dependencies, compilation);
    let fake_map = self.get_fake_map(dependencies, compilation);
    let return_module_object =
      self.get_return_module_object_source(&fake_map, true, "fakeMap[id]", runtime_template);
    formatdoc! {r#"
      var map = {map};
      {fake_map_init_statement}

      function __rspack_async_context(req) {{
        return __rspack_async_context_resolve(req).then(function(id) {{
          if(!{module_factories}[id]) {{
            var e = new Error("Module '" + req + "' ('" + id + "') is not available (weak dependency)");
            e.code = 'MODULE_NOT_FOUND';
            throw e;
          }}
          {return_module_object}
        }});
      }}
      function __rspack_async_context_resolve(req) {{
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
      __rspack_async_context.keys = {keys};
      __rspack_async_context.resolve = __rspack_async_context_resolve;
      __rspack_async_context.id = {id};
      {module}.exports = __rspack_async_context;
      "#,
      module = runtime_template.render_module_argument(ModuleArgument::Module),
      map = json_stringify(&map),
      fake_map_init_statement = self.get_fake_map_init_statement(&fake_map),
      module_factories = runtime_template.render_runtime_globals(&RuntimeGlobals::MODULE_FACTORIES),
      has_own_property = runtime_template.render_runtime_globals(&RuntimeGlobals::HAS_OWN_PROPERTY),
      keys = runtime_template.returning_function("Object.keys(map)", ""),
      id = json_stringify(self.get_module_id(&compilation.module_ids_artifact))
    }
  }

  fn get_sync_weak_source(
    &self,
    compilation: &Compilation,
    runtime_template: &mut ModuleCodegenRuntimeTemplate,
  ) -> String {
    let dependencies = self.get_dependencies();
    let map = self.get_user_request_map(dependencies, compilation);
    let fake_map = self.get_fake_map(dependencies, compilation);
    let return_module_object =
      self.get_return_module_object_source(&fake_map, true, "fakeMap[id]", runtime_template);
    formatdoc! {r#"
      var map = {map};
      {fake_map_init_statement}

      function __rspack_context(req) {{
        var id = __rspack_context_resolve(req);
        if(!{module_factories}[id]) {{
          var e = new Error("Module '" + req + "' ('" + id + "') is not available (weak dependency)");
          e.code = 'MODULE_NOT_FOUND';
          throw e;
        }}
        {return_module_object}
      }}
      function __rspack_context_resolve(req) {{
        if(!{has_own_property}(map, req)) {{
          var e = new Error("Cannot find module '" + req + "'");
          e.code = 'MODULE_NOT_FOUND';
          throw e;
        }}
        return map[req];
      }}
      __rspack_context.keys = {keys};
      __rspack_context.resolve = __rspack_context_resolve;
      __rspack_context.id = {id};
      {module}.exports = __rspack_context;
      "#,
      module = runtime_template.render_module_argument(ModuleArgument::Module),
      map = json_stringify(&map),
      fake_map_init_statement = self.get_fake_map_init_statement(&fake_map),
      module_factories = runtime_template.render_runtime_globals(&RuntimeGlobals::MODULE_FACTORIES),
      has_own_property = runtime_template.render_runtime_globals(&RuntimeGlobals::HAS_OWN_PROPERTY),
      keys = runtime_template.returning_function("Object.keys(map)", ""),
      id = json_stringify(self.get_module_id(&compilation.module_ids_artifact))
    }
  }

  fn get_eager_source(
    &self,
    compilation: &Compilation,
    runtime_template: &mut ModuleCodegenRuntimeTemplate,
  ) -> String {
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
        self.get_return_module_object_source(&fake_map, true, "fakeMap[id]", runtime_template),
      }
    } else {
      runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE)
    };
    formatdoc! {r#"
      var map = {map};
      {fake_map_init_statement}

      function __rspack_async_context(req) {{
        return __rspack_async_context_resolve(req).then({then_function});
      }}
      function __rspack_async_context_resolve(req) {{
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
      __rspack_async_context.keys = {keys};
      __rspack_async_context.resolve = __rspack_async_context_resolve;
      __rspack_async_context.id = {id};
      {module}.exports = __rspack_async_context;
      "#,
      module = runtime_template.render_module_argument(ModuleArgument::Module),
      map = json_stringify(&map),
      fake_map_init_statement = self.get_fake_map_init_statement(&fake_map),
      has_own_property = runtime_template.render_runtime_globals(&RuntimeGlobals::HAS_OWN_PROPERTY),
      keys = runtime_template.returning_function("Object.keys(map)", ""),
      id = json_stringify(self.get_module_id(&compilation.module_ids_artifact))
    }
  }

  fn get_sync_source(
    &self,
    compilation: &Compilation,
    runtime_template: &mut ModuleCodegenRuntimeTemplate,
  ) -> String {
    let dependencies = self.get_dependencies();
    let map = self.get_user_request_map(dependencies, compilation);
    let fake_map = self.get_fake_map(dependencies, compilation);
    let return_module_object =
      self.get_return_module_object_source(&fake_map, false, "fakeMap[id]", runtime_template);
    formatdoc! {r#"
      var map = {map};
      {fake_map_init_statement}

      function __rspack_context(req) {{
        var id = __rspack_context_resolve(req);
        {return_module_object}
      }}
      function __rspack_context_resolve(req) {{
        if(!{has_own_property}(map, req)) {{
          var e = new Error("Cannot find module '" + req + "'");
          e.code = 'MODULE_NOT_FOUND';
          throw e;
        }}
        return map[req];
      }}
      __rspack_context.keys = function webpackContextKeys() {{
        return Object.keys(map);
      }};
      __rspack_context.resolve = __rspack_context_resolve;
      {module}.exports = __rspack_context;
      __rspack_context.id = {id};
      "#,
      module = runtime_template.render_module_argument(ModuleArgument::Module),
      map = json_stringify(&map),
      fake_map_init_statement = self.get_fake_map_init_statement(&fake_map),
      has_own_property = runtime_template.render_runtime_globals(&RuntimeGlobals::HAS_OWN_PROPERTY),
      id = json_stringify(self.get_module_id(&compilation.module_ids_artifact))
    }
  }

  fn get_source(&self, source_string: String, compilation: &Compilation) -> BoxSource {
    let source_map_kind = self.get_source_map_kind();
    if source_map_kind.enabled() {
      OriginalSource::new(
        source_string,
        format!(
          "webpack://{}",
          make_paths_relative(&compilation.options.context, self.identifier.as_str(),)
        ),
      )
      .boxed()
    } else {
      RawStringSource::from(source_string).boxed()
    }
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

  fn source_types(&self, _module_graph: &ModuleGraph) -> &[SourceType] {
    &[SourceType::JavaScript]
  }

  fn source(&self) -> Option<&rspack_sources::BoxSource> {
    None
  }

  fn readable_identifier(&self, context: &crate::Context) -> std::borrow::Cow<'_, str> {
    let identifier = contextify(
      context,
      if self.options.resource.as_str().is_empty() {
        "false"
      } else {
        self.options.resource.as_str()
      },
    );
    create_identifier(&self.options, Some(identifier.as_str()))
      .to_string()
      .into()
  }

  fn size(
    &self,
    _source_type: Option<&crate::SourceType>,
    _compilation: Option<&Compilation>,
  ) -> f64 {
    160.0
  }

  fn lib_ident(&self, options: LibIdentOptions) -> Option<Cow<'_, str>> {
    let mut id = String::new();
    if let Some(layer) = &self.options.layer {
      id += "(";
      id += layer;
      id += ")/";
    }
    id += &contextify(
      options.context,
      if self.options.resource.as_str().is_empty() {
        "false"
      } else {
        self.options.resource.as_str()
      },
    );
    id += " ";
    id += self.options.context_options.mode.as_str();
    if self.options.context_options.recursive {
      id += " recursive";
    }
    if !self.options.addon.is_empty() {
      id += " ";
      id += &self.options.addon;
    }
    if let Some(regexp) = &self.options.context_options.reg_exp {
      id += " ";
      id += &regexp.to_pretty_string(true);
    }
    if let Some(include) = &self.options.context_options.include {
      id += " include: ";
      id += &include.to_pretty_string(true);
    }
    if let Some(exclude) = &self.options.context_options.exclude {
      id += " exclude: ";
      id += &exclude.to_pretty_string(true);
    }
    if let Some(exports) = &self.options.context_options.referenced_exports {
      id += " referencedExports: ";
      id += &exports.iter().map(|ids| ids.iter().join(".")).join(", ");
    }
    Some(Cow::Owned(id))
  }

  async fn build(
    &mut self,
    _build_context: BuildContext,
    _: Option<&Compilation>,
  ) -> Result<BuildResult> {
    let resolve_dependencies = &self.resolve_dependencies;
    let context_element_dependencies = resolve_dependencies(self.options.clone()).await?;

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
            let name = if !(name.contains(CHUNK_NAME_INDEX_PLACEHOLDER)
              || name.contains(CHUNK_NAME_REQUEST_PLACEHOLDER))
            {
              Cow::Owned(format!("{name}{CHUNK_NAME_INDEX_PLACEHOLDER}"))
            } else {
              Cow::Borrowed(name)
            };

            let name = name.cow_replace(CHUNK_NAME_INDEX_PLACEHOLDER, &index.to_string());
            let name = name.cow_replace(
              CHUNK_NAME_REQUEST_PLACEHOLDER,
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

    if !self.options.resource.as_str().is_empty() {
      let mut context_dependencies: ArcPathSet = Default::default();
      context_dependencies.insert(self.options.resource.as_std_path().into());
      self.build_info.context_dependencies = context_dependencies;
    }

    Ok(BuildResult {
      dependencies,
      blocks,
      optimization_bailouts: vec![],
    })
  }

  // #[tracing::instrument("ContextModule::code_generation", skip_all, fields(identifier = ?self.identifier()))]
  async fn code_generation(
    &self,
    code_generation_context: &mut ModuleCodeGenerationContext,
  ) -> Result<CodeGenerationResult> {
    let ModuleCodeGenerationContext {
      compilation,
      runtime_template,
      ..
    } = code_generation_context;
    let mut code_generation_result = CodeGenerationResult::default();
    let source = self.get_source(
      self.get_source_string(compilation, runtime_template),
      compilation,
    );
    code_generation_result.add(SourceType::JavaScript, source);
    let mut all_deps = self.get_dependencies().to_vec();
    let module_graph = compilation.get_module_graph();
    for block in self.get_blocks() {
      let block = module_graph
        .block_by_id(block)
        .expect("should have block in ContextModule code_generation");
      all_deps.extend(block.get_dependencies());
    }

    Ok(code_generation_result)
  }

  async fn get_runtime_hash(
    &self,
    compilation: &Compilation,
    runtime: Option<&RuntimeSpec>,
  ) -> Result<RspackHashDigest> {
    let mut hasher = RspackHash::from(&compilation.options.output);
    module_update_hash(self, &mut hasher, compilation, runtime);
    Ok(hasher.digest(&compilation.options.output.hash_digest))
  }
}

impl_empty_diagnosable_trait!(ContextModule);

impl Identifiable for ContextModule {
  fn identifier(&self) -> Identifier {
    self.identifier
  }
}

fn create_identifier(options: &ContextModuleOptions, resource: Option<&str>) -> Identifier {
  let mut id = resource
    .unwrap_or(if options.resource.as_str().is_empty() {
      "false"
    } else {
      options.resource.as_str()
    })
    .to_owned();
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
    id += &exports.iter().map(|ids| ids.iter().join(".")).join(", ");
  }

  if let Some(GroupOptions::ChunkGroup(group)) = &options.context_options.group_options {
    if let Some(chunk_name) = &group.name {
      id += "|chunkName: ";
      id += chunk_name;
    }
    id += "|groupOptions: {";
    if let Some(o) = group.prefetch_order {
      id.push_str(&format!("prefetchOrder: {o},"));
    }
    if let Some(o) = group.preload_order {
      id.push_str(&format!("preloadOrder: {o},"));
    }
    if let Some(o) = group.fetch_priority {
      id.push_str(&format!("fetchPriority: {o},"));
    }
    id += "}";
  }
  id += match options.context_options.namespace_object {
    ContextNameSpaceObject::Strict => "|strict namespace object",
    ContextNameSpaceObject::Bool(true) => "|namespace object",
    _ => "",
  };
  if let Some(attributes) = &options.context_options.attributes {
    id += "|importAttributes: ";
    id += &serde_json::to_string(attributes).expect("json stringify failed");
  }
  if let Some(layer) = &options.layer {
    id += "|layer: ";
    id += layer;
  }
  id.into()
}
