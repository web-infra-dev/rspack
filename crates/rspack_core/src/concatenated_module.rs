use std::{
  borrow::Cow,
  collections::hash_map::{DefaultHasher, Entry},
  fmt::Debug,
  hash::{BuildHasherDefault, Hash, Hasher},
  iter,
  os::unix::fs::OpenOptionsExt,
  sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
  },
};

use bitflags::bitflags;
use dashmap::DashMap;
use derivative::Derivative;
use itertools::cloned;
use once_cell::sync::OnceCell;
use oxc_resolver::ResolveOptions;
use rspack_error::{error, Diagnosable, Diagnostic, Result, Severity};
use rspack_hash::RspackHash;
use rspack_identifier::Identifiable;
use rspack_loader_runner::{run_loaders, Content, ResourceData};
use rspack_sources::{
  BoxSource, CachedSource, OriginalSource, RawSource, Source, SourceExt, SourceMap,
  SourceMapSource, WithoutOriginalOptions,
};
use rustc_hash::FxHasher;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use serde_json::json;
use swc_core::ecma::atoms::Atom;

use crate::{
  add_connection_states, contextify, filter_runtime, get_context, merge_runtime_condition,
  merge_runtime_condition_non_false, subtract_runtime_condition, AsyncDependenciesBlockIdentifier,
  BoxDependency, BoxLoader, BoxModule, BuildContext, BuildInfo, BuildMeta, BuildResult,
  CodeGenerationResult, Compilation, CompilerOptions, ConnectionId, ConnectionState, Context,
  DependenciesBlock, DependencyId, DependencyTemplate, FactoryMeta, GenerateContext,
  GeneratorOptions, LibIdentOptions, LoaderRunnerPluginProcessResource, Module, ModuleDependency,
  ModuleGraph, ModuleGraphConnection, ModuleIdentifier, ModuleType, ParseContext, ParseResult,
  ParserAndGenerator, ParserOptions, Resolve, RuntimeCondition, RuntimeSpec, SourceType,
};

#[derive(Debug)]
pub struct RootModuleContext {
  id: ModuleIdentifier,
  readable_identifier: String,
  name_for_condition: Option<Box<str>>,
  lib_indent: Option<String>,
  resolve_options: Option<Box<Resolve>>,
  code_generation_dependencies: Option<Vec<Box<dyn ModuleDependency>>>,
  presentational_dependencies: Option<Vec<Box<dyn DependencyTemplate>>>,
  context: Option<Box<Context>>,
  side_effect_connection_state: ConnectionState,
}

#[derive(Debug, Clone)]
pub struct ConcatenatedInnerModule {
  id: ModuleIdentifier,
  size: f64,
  original_source: Option<BoxSource>,
}

#[derive(Debug)]
pub struct ExternalModuleInfo {
  index: usize,
  module: ModuleIdentifier,
}

pub enum ConcatenationEntryType {
  Concatenated,
  External,
}
pub enum ConnectionOrModuleIdent {
  Module(ModuleIdentifier),
  Connection(ConnectionId),
}

impl ConnectionOrModuleIdent {
  fn get_module_id(&self, mg: &ModuleGraph) -> ModuleIdentifier {
    match self {
      ConnectionOrModuleIdent::Module(m) => *m,
      ConnectionOrModuleIdent::Connection(c) => {
        let con = mg
          .connection_by_connection_id(c)
          .expect("should have connection");
        con.module_identifier
      }
    }
  }
}
pub struct ConcatenationEntry {
  ty: ConcatenationEntryType,
  /// I do want to align with webpack, but https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/optimize/ConcatenatedModule.js#L1018-L1027
  /// you know ..
  connection_or_module_id: ConnectionOrModuleIdent,
  runtime_condition: RuntimeCondition,
}

#[derive(Debug)]
pub struct ConcatenatedModuleImportInfo {
  connection: ModuleGraphConnection,
  source_order: i32,
  range_start: Option<u32>,
}

#[derive(Debug)]
pub struct ConcatenatedModuleInfo {
  pub index: usize,
  pub module: ModuleIdentifier,
  pub export_map: HashMap<Atom, String>,
  pub raw_export_map: HashMap<Atom, String>,
  pub namespace_export_symbol: Option<Atom>,
}

pub struct ConnectionWithRuntimeCondition {
  pub connection: ModuleGraphConnection,
  pub runtime_condition: RuntimeCondition,
}

#[derive(Debug)]
pub enum ModuleInfo {
  External(ExternalModuleInfo),
  Concatenated(ConcatenatedModuleInfo),
}

#[derive(Debug)]
pub enum ModuleInfoOrReference {
  External(ExternalModuleInfo),
  Concatenated(ConcatenatedModuleInfo),
  Reference(ModuleIdentifier),
}

impl ModuleInfo {
  pub fn index(&self) -> usize {
    match self {
      ModuleInfo::External(e) => e.index,
      ModuleInfo::Concatenated(c) => c.index,
    }
  }
}

#[derive(Debug)]
pub struct ConcatenatedModule {
  id: ModuleIdentifier,
  root_module_ctxt: RootModuleContext,
  modules: Vec<ConcatenatedInnerModule>,
  runtime: Option<RuntimeSpec>,
  factory_meta: Option<FactoryMeta>,

  blocks: Vec<AsyncDependenciesBlockIdentifier>,
  dependencies: Vec<DependencyId>,

  cached_source_sizes: DashMap<SourceType, f64, BuildHasherDefault<FxHasher>>,

  diagnostics: Mutex<Vec<Diagnostic>>,
  cached_hash: OnceCell<u64>,
}

impl ConcatenatedModule {
  pub fn new(
    id: ModuleIdentifier,
    root_module_ctxt: RootModuleContext,
    modules: Vec<ConcatenatedInnerModule>,
    runtime: Option<RuntimeSpec>,
    factory_meta: Option<FactoryMeta>,
  ) -> Self {
    Self {
      id,
      root_module_ctxt,
      modules,
      runtime,
      factory_meta,
      dependencies: vec![],
      blocks: vec![],
      cached_source_sizes: DashMap::default(),
      diagnostics: Mutex::new(vec![]),
      cached_hash: OnceCell::default(),
    }
  }

  pub fn id(&self) -> ModuleIdentifier {
    self.id
  }

  fn get_modules(&self) -> Vec<ConcatenatedInnerModule> {
    self.modules.clone()
  }
}

impl Identifiable for ConcatenatedModule {
  #[inline]
  fn identifier(&self) -> ModuleIdentifier {
    self.id
  }
}

impl DependenciesBlock for ConcatenatedModule {
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
impl Module for ConcatenatedModule {
  fn module_type(&self) -> &ModuleType {
    // https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/optimize/ConcatenatedModule.js#L688
    &ModuleType::JsEsm
  }
  fn source_types(&self) -> &[SourceType] {
    &[SourceType::JavaScript]
  }

  fn original_source(&self) -> Option<&dyn Source> {
    None
  }

  fn readable_identifier(&self, context: &Context) -> Cow<str> {
    Cow::Owned(format!(
      "{} + {} modules",
      self.root_module_ctxt.readable_identifier,
      self.modules.len() - 1
    ))
  }

  fn size(&self, source_type: &SourceType) -> f64 {
    if let Some(size_ref) = self.cached_source_sizes.get(source_type) {
      *size_ref
    } else {
      let size = self.modules.iter().fold(0.0, |acc, cur| acc + cur.size);
      self.cached_source_sizes.insert(*source_type, size);
      size
    }
  }

  async fn build(&mut self, build_context: BuildContext<'_>) -> Result<BuildResult> {
    todo!()
    // self.clear_diagnostics();
    //
    // let mut build_info = BuildInfo::default();
    // let mut build_meta = BuildMeta::default();
    //
    // build_context.plugin_driver.before_loaders(self).await?;
    //
    // let loader_result = run_loaders(
    //   &self.loaders,
    //   &self.resource_data,
    //   &[Box::new(LoaderRunnerPluginProcessResource {
    //     plugin_driver: build_context.plugin_driver.clone(),
    //   })],
    //   build_context.compiler_context,
    // )
    // .await;
    // let (loader_result, ds) = match loader_result {
    //   Ok(r) => r.split_into_parts(),
    //   Err(e) => {
    //     let d = Diagnostic::from(e);
    //     self.source = NormalModuleSource::BuiltFailed(d.clone());
    //     self.add_diagnostic(d);
    //     let mut hasher = RspackHash::from(&build_context.compiler_options.output);
    //     self.update_hash(&mut hasher);
    //     build_meta.hash(&mut hasher);
    //     build_info.hash = Some(hasher.digest(&build_context.compiler_options.output.hash_digest));
    //     return Ok(BuildResult {
    //       build_info,
    //       build_meta: Default::default(),
    //       dependencies: Vec::new(),
    //       blocks: Vec::new(),
    //       analyze_result: Default::default(),
    //     });
    //   }
    // };
    // self.add_diagnostics(ds);
    //
    // let content = if self.module_type().is_binary() {
    //   Content::Buffer(loader_result.content.into_bytes())
    // } else {
    //   Content::String(loader_result.content.into_string_lossy())
    // };
    // let original_source = self.create_source(content, loader_result.source_map)?;
    // let mut code_generation_dependencies: Vec<Box<dyn ModuleDependency>> = Vec::new();
    //
    // let (
    //   ParseResult {
    //     source,
    //     dependencies,
    //     blocks,
    //     presentational_dependencies,
    //     analyze_result,
    //   },
    //   ds,
    // ) = self
    //   .parser_and_generator
    //   .parse(ParseContext {
    //     source: original_source.clone(),
    //     module_identifier: self.identifier(),
    //     module_parser_options: self.parser_options.as_ref(),
    //     module_type: &self.module_type,
    //     module_user_request: &self.user_request,
    //     loaders: &self.loaders,
    //     resource_data: &self.resource_data,
    //     compiler_options: build_context.compiler_options,
    //     additional_data: loader_result.additional_data,
    //     code_generation_dependencies: &mut code_generation_dependencies,
    //     build_info: &mut build_info,
    //     build_meta: &mut build_meta,
    //   })?
    //   .split_into_parts();
    // self.add_diagnostics(ds);
    // // Only side effects used in code_generate can stay here
    // // Other side effects should be set outside use_cache
    // self.original_source = Some(source.clone());
    // self.source = NormalModuleSource::new_built(source, self.clone_diagnostics());
    // self.code_generation_dependencies = Some(code_generation_dependencies);
    // self.presentational_dependencies = Some(presentational_dependencies);
    //
    // let mut hasher = RspackHash::from(&build_context.compiler_options.output);
    // self.update_hash(&mut hasher);
    // build_meta.hash(&mut hasher);
    //
    // build_info.hash = Some(hasher.digest(&build_context.compiler_options.output.hash_digest));
    // build_info.cacheable = loader_result.cacheable;
    // build_info.file_dependencies = loader_result.file_dependencies;
    // build_info.context_dependencies = loader_result.context_dependencies;
    // build_info.missing_dependencies = loader_result.missing_dependencies;
    // build_info.build_dependencies = loader_result.build_dependencies;
    // build_info.asset_filenames = loader_result.asset_filenames;
    //
    // Ok(BuildResult {
    //   build_info,
    //   build_meta,
    //   dependencies,
    //   blocks,
    //   analyze_result,
    // })
  }

  fn code_generation(
    &self,
    compilation: &Compilation,
    runtime: Option<&RuntimeSpec>,
  ) -> Result<CodeGenerationResult> {
    let generation_runtime = runtime.cloned().expect("should have runtime");
    todo!()
    // if let NormalModuleSource::BuiltSucceed(source) = &self.source {
    //   let mut code_generation_result = CodeGenerationResult::default();
    //   for source_type in self.source_types() {
    //     let generation_result = self.parser_and_generator.generate(
    //       source,
    //       self,
    //       &mut GenerateContext {
    //         compilation,
    //         module_generator_options: self.generator_options.as_ref(),
    //         runtime_requirements: &mut code_generation_result.runtime_requirements,
    //         data: &mut code_generation_result.data,
    //         requested_source_type: *source_type,
    //         runtime,
    //       },
    //     )?;
    //     code_generation_result.add(*source_type, CachedSource::new(generation_result).boxed());
    //   }
    //   code_generation_result.set_hash(
    //     &compilation.options.output.hash_function,
    //     &compilation.options.output.hash_digest,
    //     &compilation.options.output.hash_salt,
    //   );
    //   Ok(code_generation_result)
    // } else if let NormalModuleSource::BuiltFailed(error_message) = &self.source {
    //   let mut code_generation_result = CodeGenerationResult::default();
    //
    //   // If the module build failed and the module is able to emit JavaScript source,
    //   // we should emit an error message to the runtime, otherwise we do nothing.
    //   if self.source_types().contains(&SourceType::JavaScript) {
    //     let error = error_message.render_report(compilation.options.stats.colors)?;
    //     code_generation_result.add(
    //       SourceType::JavaScript,
    //       RawSource::from(format!("throw new Error({});\n", json!(error))).boxed(),
    //     );
    //   }
    //   code_generation_result.set_hash(
    //     &compilation.options.output.hash_function,
    //     &compilation.options.output.hash_digest,
    //     &compilation.options.output.hash_salt,
    //   );
    //   Ok(code_generation_result)
    // } else {
    //   Err(error!(
    //     "Failed to generate code because ast or source is not set for module {}",
    //     self.request
    //   ))
    // }
  }

  fn name_for_condition(&self) -> Option<Box<str>> {
    self.root_module_ctxt.name_for_condition.clone()
  }

  fn lib_ident(&self, options: LibIdentOptions) -> Option<Cow<str>> {
    self
      .root_module_ctxt
      .lib_indent
      .clone()
      .map(|item| Cow::Owned(item))
  }

  fn get_resolve_options(&self) -> Option<Box<Resolve>> {
    self.root_module_ctxt.resolve_options.clone()
  }

  fn get_code_generation_dependencies(&self) -> Option<&[Box<dyn ModuleDependency>]> {
    if let Some(deps) = self
      .root_module_ctxt
      .code_generation_dependencies
      .as_deref()
      && !deps.is_empty()
    {
      Some(deps)
    } else {
      None
    }
  }

  fn get_presentational_dependencies(&self) -> Option<&[Box<dyn DependencyTemplate>]> {
    if let Some(deps) = self.root_module_ctxt.presentational_dependencies.as_deref()
      && !deps.is_empty()
    {
      Some(deps)
    } else {
      None
    }
  }

  fn get_context(&self) -> Option<Box<Context>> {
    self.root_module_ctxt.context.clone()
  }

  // Port from https://github.com/webpack/webpack/blob/main/lib/ConcatenatedModule.js#L1120
  fn get_side_effects_connection_state(
    &self,
    module_graph: &ModuleGraph,
    module_chain: &mut HashSet<ModuleIdentifier>,
  ) -> ConnectionState {
    self.root_module_ctxt.side_effect_connection_state
  }
}

impl Diagnosable for ConcatenatedModule {
  fn add_diagnostic(&self, diagnostic: Diagnostic) {
    self
      .diagnostics
      .lock()
      .expect("should be able to lock diagnostics")
      .push(diagnostic);
  }

  fn add_diagnostics(&self, mut diagnostics: Vec<Diagnostic>) {
    self
      .diagnostics
      .lock()
      .expect("should be able to lock diagnostics")
      .append(&mut diagnostics);
  }

  fn clone_diagnostics(&self) -> Vec<Diagnostic> {
    self
      .diagnostics
      .lock()
      .expect("should be able to lock diagnostics")
      .iter()
      .cloned()
      .collect()
  }
}

impl PartialEq for ConcatenatedModule {
  fn eq(&self, other: &Self) -> bool {
    self.identifier() == other.identifier()
  }
}

impl Eq for ConcatenatedModule {}

impl ConcatenatedModule {
  fn clear_diagnostics(&mut self) {
    self
      .diagnostics
      .lock()
      .expect("should be able to lock diagnostics")
      .clear()
  }

  fn get_modules_with_info(
    &self,
    module_graph: ModuleGraph,
    runtime: RuntimeSpec,
  ) -> (
    Vec<ModuleInfoOrReference>,
    HashMap<ModuleIdentifier, ModuleInfo>,
  ) {
    todo!()
  }

  fn create_concatenation_list(
    &self,
    root_module: ModuleIdentifier,
    module_set: HashSet<ModuleIdentifier>,
    runtime: RuntimeSpec,
    mg: &ModuleGraph,
  ) -> Vec<ConcatenationEntry> {
    todo!()
  }

  fn enter_module(
    &self,
    root_module: ModuleIdentifier,
    module_set: &mut HashSet<ModuleIdentifier>,
    runtime: Option<&RuntimeSpec>,
    mg: &ModuleGraph,
    con: ModuleGraphConnection,
    mut runtime_condition: RuntimeCondition,
    exists_entry: &mut HashMap<ModuleIdentifier, RuntimeCondition>,
    list: &mut Vec<ConcatenationEntry>,
  ) {
    let module = con.module_identifier;
    let exist_entry = match exists_entry.get(&module) {
      Some(condition) if matches!(condition, RuntimeCondition::Boolean(true)) => return,
      None => None,
      Some(condtition) => Some(runtime_condition.clone()),
    };
    if module_set.contains(&module) {
      exists_entry.insert(module, RuntimeCondition::Boolean(true));
      if !matches!(runtime_condition, RuntimeCondition::Boolean(true)) {
        panic!(
          "Cannot runtime-conditional concatenate a module ({}) in {}. This should not happen.",
          module, self.root_module_ctxt.id,
        );
      }
      let imports = self.get_concatenated_imports(&module, &root_module, module_set, runtime, mg);
      for import in imports {
        self.enter_module(
          root_module,
          module_set,
          runtime,
          mg,
          import.connection,
          import.runtime_condition,
          exists_entry,
          list,
        );
      }
      list.push(ConcatenationEntry {
        ty: ConcatenationEntryType::Concatenated,
        runtime_condition,
        connection_or_module_id: ConnectionOrModuleIdent::Module(con.module_identifier),
      });
    } else {
      if let Some(cond) = exist_entry {
        let reduced_runtime_condition =
          subtract_runtime_condition(&runtime_condition, &cond, runtime);
        if matches!(reduced_runtime_condition, RuntimeCondition::Boolean(false)) {
          return;
        }
        exists_entry.insert(con.module_identifier, reduced_runtime_condition);
      } else {
        exists_entry.insert(con.module_identifier, runtime_condition.clone());
      }
      if let Some(last) = list.last_mut() {
        if matches!(last.ty, ConcatenationEntryType::External)
          && last.connection_or_module_id.get_module_id(mg) == con.module_identifier
        {
          last.runtime_condition =
            merge_runtime_condition(&last.runtime_condition, &runtime_condition, runtime);
          return;
        }
      }
      let con_id = mg
        .connection_id_by_dependency_id(&con.dependency_id)
        .expect("should have dep id");
      list.push(ConcatenationEntry {
        ty: ConcatenationEntryType::External,
        runtime_condition,
        connection_or_module_id: ConnectionOrModuleIdent::Connection(con_id.clone()),
      })
    }
  }

  fn get_concatenated_imports(
    &self,
    module_id: &ModuleIdentifier,
    root_module_id: &ModuleIdentifier,
    module_set: &mut HashSet<ModuleIdentifier>,
    runtime: Option<&RuntimeSpec>,
    mg: &ModuleGraph,
  ) -> Vec<ConnectionWithRuntimeCondition> {
    let box_module = mg
      .module_by_identifier(module_id)
      .expect("should have module");
    let mut connections = mg
      .get_outgoing_connections(box_module)
      .into_iter()
      .cloned()
      .collect::<Vec<_>>();
    if module_id == root_module_id {
      let self_module = mg
        .module_by_identifier(&self.id)
        .expect("should have module");
      for c in mg.get_outgoing_connections(self_module) {
        connections.push(c.clone());
      }
    }

    let mut references = connections
      .into_iter()
      .filter_map(|connection| {
        let dep = connection.dependency_id.get_dependency(&mg);
        if !is_harmony_dep_like(dep) {
          return None;
        }

        // TODO: we don't have resolved_original_module
        if !(connection.original_module_identifier == Some(*module_id)
          && connection.is_target_active(&mg, self.runtime.as_ref()))
        {
          return None;
        }
        // now the dep should be one of `HarmonyExportImportedSpecifierDependency`, `HarmonyImportSideEffectDependency`, `HarmonyImportSpecifierDependency`,
        // the expect is safe now
        Some(ConcatenatedModuleImportInfo {
          connection,
          source_order: dep
            .source_order()
            .expect("source order should not be empty"),
          range_start: dep.span().map(|span| span.start),
        })
      })
      .collect::<Vec<_>>();

    references.sort_by(|a, b| {
      if a.source_order != b.source_order {
        a.source_order.cmp(&b.source_order)
      } else {
        match (a.range_start, b.range_start) {
          (None, None) => std::cmp::Ordering::Equal,
          (None, Some(_)) => std::cmp::Ordering::Greater,
          (Some(_), None) => 1 + 2.0,
          (Some(_), Some(_)) => todo!(),
        }
      }
    });

    let mut references_map = HashMap::default();
    for reference in references {
      let runtime_condition =
        filter_runtime(runtime, |r| reference.connection.is_target_active(&mg, r));
      if matches!(runtime_condition, RuntimeCondition::Boolean(false)) {
        continue;
      }
      let module = reference.connection.module_identifier;
      match references_map.entry(module) {
        Entry::Occupied(mut occ) => {
          let cur: &ConnectionWithRuntimeCondition = occ.get();
          let merged_condition =
            merge_runtime_condition_non_false(&cur.runtime_condition, &runtime_condition, runtime);
          occ.get_mut().runtime_condition = merged_condition;
        }
        Entry::Vacant(vac) => {
          vac.insert(ConnectionWithRuntimeCondition {
            connection: reference.connection,
            runtime_condition,
          });
        }
      }
    }

    references_map.into_values().collect()
  }
}

impl Hash for ConcatenatedModule {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    if let Some(h) = self.cached_hash.get() {
      h.hash(state);
      return;
    }

    let mut temp_state = DefaultHasher::default();

    "__rspack_internal__ConcatenatedModule".hash(&mut temp_state);
    // the module has been sorted, so the has should be consistant
    for module in self.modules.iter() {
      if let Some(ref original_source) = module.original_source {
        original_source.hash(&mut temp_state);
      }
    }
    let res = temp_state.finish();
    res.hash(state);
    self
      .cached_hash
      .set(res)
      .expect("should set hash of ConcatenatedModule")
  }
}

pub fn is_harmony_dep_like(dep: &BoxDependency) -> bool {
  [
    "HarmonyExportImportedSpecifierDependency",
    "HarmonyImportSideEffectDependency",
    "HarmonyImportSpecifierDependency",
  ]
  .contains(&dep.dependency_debug_name())
}
