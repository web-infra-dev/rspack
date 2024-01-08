use std::{
  borrow::Cow,
  collections::hash_map::{DefaultHasher, Entry},
  env::current_exe,
  fmt::Debug,
  hash::{BuildHasherDefault, Hash, Hasher},
  sync::{Arc, Mutex},
  task::Wake,
};

use dashmap::DashMap;
use indexmap::{IndexMap, IndexSet};
use once_cell::sync::OnceCell;
use rspack_ast::javascript::Ast;
use rspack_error::{
  miette::{ErrReport, MietteError},
  AnyhowError, Diagnosable, Diagnostic, DiagnosticKind, ErrorExt, Result, TraceableError,
};
use rspack_hash::RspackHash;
use rspack_identifier::Identifiable;
use rspack_sources::{BoxSource, ReplaceSource, Source};
use rustc_hash::FxHasher;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use swc_core::{
  common::{FileName, Spanned, SyntaxContext},
  ecma::{
    ast::{EsVersion, Program},
    atoms::Atom,
    parser::{parse_file_as_module, Syntax},
    transforms::base::resolver,
  },
};
use swc_node_comments::SwcComments;

use crate::{
  concatenated_module, filter_runtime, merge_runtime_condition, merge_runtime_condition_non_false,
  reserverd_names::RESERVED_NAMES, subtract_runtime_condition, AsyncDependenciesBlockIdentifier,
  BoxDependency, BuildContext, BuildInfo, BuildMeta, BuildResult, ChunkInitFragments,
  CodeGenerationResult, Compilation, ConcatenationScope, ConnectionId, ConnectionState, Context,
  DependenciesBlock, DependencyId, DependencyTemplate, ErrorSpan, FactoryMeta, LibIdentOptions,
  Module, ModuleDependency, ModuleGraph, ModuleGraphConnection, ModuleIdentifier, ModuleType,
  ParserAndGenerator, Resolve, RuntimeCondition, RuntimeGlobals, RuntimeSpec, SourceType, Template,
  DEFAULT_EXPORT, NAMESPACE_OBJECT_EXPORT,
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
  build_meta: Option<BuildMeta>,
}

#[derive(Debug, Clone)]
pub struct ConcatenatedInnerModule {
  id: ModuleIdentifier,
  size: f64,
  original_source: Option<BoxSource>,
}

#[derive(Debug, Clone)]
pub struct ExternalModuleInfo {
  index: usize,
  module: ModuleIdentifier,
  runtime_condition: RuntimeCondition,
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

#[derive(Debug, Clone, Default)]
pub struct ConcatenatedModuleInfo {
  pub index: usize,
  pub module: ModuleIdentifier,
  pub namespace_export_symbol: Option<Atom>,
  pub chunk_init_fragments: ChunkInitFragments,
  pub module_ctxt: SyntaxContext,
  pub global_ctxt: SyntaxContext,
  pub runtime_requirements: RuntimeGlobals,
  pub ast: Option<Ast>,
  pub source: Option<ReplaceSource<Arc<dyn Source>>>,
  pub internal_source: Option<Arc<dyn Source>>,
  pub internal_names: HashMap<String, String>,
  pub export_map: Option<HashMap<Atom, String>>,
  pub raw_export_map: Option<HashMap<Atom, String>>,
  pub namespace_object_name: Option<String>,
  pub interop_namespace_object_used: bool,
  pub interop_namespace_object_name: Option<String>,
  pub interop_namespace_object2_used: bool,
  pub interop_namespace_object2_name: Option<String>,
  pub interop_default_access_used: bool,
  pub interop_default_access_name: Option<String>,
}

// * @property {boolean} interopNamespaceObjectUsed
// * @property {string} interopNamespaceObjectName
// * @property {boolean} interopNamespaceObject2Used
// * @property {string} interopNamespaceObject2Name
// * @property {boolean} interopDefaultAccessUsed
// * @property {string} interopDefaultAccessName
pub struct ConnectionWithRuntimeCondition {
  pub connection: ModuleGraphConnection,
  pub runtime_condition: RuntimeCondition,
  pub interop_namespace_object_used: bool,
  pub interop_namespace_object_name: Option<String>,
  pub interop_namespace_object2_used: bool,
  pub interop_namespace_object2_name: Option<String>,
  pub interop_default_access_used: bool,
  pub interop_default_access_name: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ModuleInfo {
  External(ExternalModuleInfo),
  Concatenated(ConcatenatedModuleInfo),
}

#[derive(Debug)]
pub enum ModuleInfoOrReference {
  External(ExternalModuleInfo),
  Concatenated(ConcatenatedModuleInfo),
  Reference {
    /// target in webpack https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/optimize/ConcatenatedModule.js#L1818
    module_info_id: ModuleIdentifier,
    runtime_condition: RuntimeCondition,
  },
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
  build_info: Option<BuildInfo>,
}

impl ConcatenatedModule {
  pub fn new(
    id: ModuleIdentifier,
    root_module_ctxt: RootModuleContext,
    mut modules: Vec<ConcatenatedInnerModule>,
    runtime: Option<RuntimeSpec>,
    factory_meta: Option<FactoryMeta>,
  ) -> Self {
    // make the hash consistant
    modules.sort_by(|a, b| a.id.cmp(&b.id));
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
      build_info: None,
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

  fn get_diagnostics(&self) -> Vec<Diagnostic> {
    let guard = self.diagnostics.lock().expect("should have diagnostics");
    guard.clone()
  }

  fn build_info(&self) -> Option<&BuildInfo> {
    self.build_info.as_ref()
  }

  fn build_meta(&self) -> Option<&BuildMeta> {
    self.root_module_ctxt.build_meta.as_ref()
  }

  fn set_module_build_info_and_meta(&mut self, build_info: BuildInfo, _: BuildMeta) {
    self.build_info = Some(build_info);
  }

  fn source_types(&self) -> &[SourceType] {
    &[SourceType::JavaScript]
  }

  fn original_source(&self) -> Option<&dyn Source> {
    None
  }

  fn readable_identifier(&self, _context: &Context) -> Cow<str> {
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

  /// the compilation is asserted to be `Some(Compilation)`, https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/optimize/ModuleConcatenationPlugin.js#L394-L418
  async fn build(
    &mut self,
    build_context: BuildContext<'_>,
    compilation: Option<&Compilation>,
  ) -> Result<BuildResult> {
    let compilation = compilation.expect("should pass compilation");
    // https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/optimize/ConcatenatedModule.js#L774-L784
    // Some fields does not exists in rspack
    let mut build_info = BuildInfo {
      cacheable: true,
      hash: None,
      strict: true,
      file_dependencies: Default::default(),
      context_dependencies: Default::default(),
      missing_dependencies: Default::default(),
      build_dependencies: Default::default(),
      asset_filenames: Default::default(),
      harmony_named_exports: Default::default(),
      all_star_exports: Default::default(),
      need_create_require: Default::default(),
      json_data: Default::default(),
    };
    self.clear_diagnostics();

    let mut hasher = RspackHash::from(&build_context.compiler_options.output);
    self.update_hash(&mut hasher);
    self.build_meta().hash(&mut hasher);

    build_info.hash = Some(hasher.digest(&build_context.compiler_options.output.hash_digest));

    for m in self.modules.iter() {
      let module = compilation
        .module_graph
        .module_by_identifier(&m.id)
        .expect("should have module");
      let cur_build_info = module.build_info().expect("should have module info");

      // populate cacheable
      if !cur_build_info.cacheable {
        build_info.cacheable = false;
      }

      // populate dependencies
      for dep_id in module.get_dependencies() {
        let dep = dep_id.get_dependency(&compilation.module_graph);
        let module_id_of_dep = compilation
          .module_graph
          .module_identifier_by_dependency_id(dep_id)
          .expect("should have module");
        if !is_harmony_dep_like(dep)
          || !self.modules.iter().any(|item| &item.id == module_id_of_dep)
        {
          self.dependencies.push(*dep_id);
        }
      }

      // populate blocks
      for b in module.get_blocks() {
        self.blocks.push(*b);
      }
      let mut diagnostics_guard = self.diagnostics.lock().expect("should have diagnostics");
      // populate diagnostic
      for d in module.get_diagnostics() {
        diagnostics_guard.push(d.clone());
      }
      // release guard ASAP
      drop(diagnostics_guard);
    }
    // return a dummy result is enough, since we don't build the ConcatenatedModule in make phase
    Ok(BuildResult::default())
  }

  fn code_generation(
    &self,
    compilation: &Compilation,
    runtime: Option<&RuntimeSpec>,
    _: Option<ConcatenationScope>,
  ) -> Result<CodeGenerationResult> {
    let generation_runtime = runtime.cloned().expect("should have runtime");
    let merged_runtime = if let Some(ref runtime) = self.runtime {
      generation_runtime
        .intersection(runtime)
        .cloned()
        .collect::<HashSet<Arc<str>>>()
    } else {
      generation_runtime
    };

    let (modules_with_info, mut module_to_info_map) =
      self.get_modules_with_info(&compilation.module_graph, runtime);

    // Set with modules that need a generated namespace object
    let mut needed_namespace_objects: HashSet<ConcatenatedModuleInfo> = HashSet::default();

    // Generate source code and analyze scopes
    // Prepare a ReplaceSource for the final source
    //
    let mut updated_pairs = vec![];
    let arc_map = Arc::new(module_to_info_map);
    for (id, info) in arc_map.iter() {
      let updated_module_info = self.analyze_module(
        compilation,
        Arc::clone(&arc_map),
        info.clone(),
        Some(&merged_runtime),
      )?;
      updated_pairs.push((*id, updated_module_info));
    }
    let mut module_to_info_map =
      Arc::into_inner(arc_map).expect("reference count should only by one");

    for (id, module_info) in updated_pairs {
      module_to_info_map.insert(id, module_info);
    }

    let all_used_name = HashSet::from_iter(RESERVED_NAMES.iter().map(|item| Atom::from(*item)));

    // TODO: top_level declaration, do we need this?

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

  fn lib_ident(&self, _options: LibIdentOptions) -> Option<Cow<str>> {
    self.root_module_ctxt.lib_indent.clone().map(Cow::Owned)
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
    _module_graph: &ModuleGraph,
    _module_chain: &mut HashSet<ModuleIdentifier>,
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

  // todo: replace self.modules with indexmap or linkedhashset
  fn get_modules_with_info(
    &self,
    mg: &ModuleGraph,
    runtime: Option<&RuntimeSpec>,
  ) -> (
    Vec<ModuleInfoOrReference>,
    IndexMap<ModuleIdentifier, ModuleInfo>,
  ) {
    let ordered_concatenation_list = self.create_concatenation_list(
      self.root_module_ctxt.id,
      IndexSet::from_iter(self.modules.iter().map(|item| item.id)),
      runtime,
      mg,
    );
    let mut list = vec![];
    let mut map: IndexMap<rspack_identifier::Identifier, ModuleInfo> = IndexMap::default();
    for (i, concatenation_entry) in ordered_concatenation_list.into_iter().enumerate() {
      let module_id = concatenation_entry
        .connection_or_module_id
        .get_module_id(mg);
      match map.entry(module_id) {
        indexmap::map::Entry::Occupied(occ) => {
          let info = occ.get();
          list.push(ModuleInfoOrReference::Reference {
            module_info_id: module_id,
            runtime_condition: concatenation_entry.runtime_condition,
          });
        }
        indexmap::map::Entry::Vacant(mut vac) => {
          match concatenation_entry.ty {
            ConcatenationEntryType::Concatenated => {
              let info = ConcatenatedModuleInfo {
                index: i,
                module: module_id,
                ..Default::default()
              };
              vac.insert(ModuleInfo::Concatenated(info.clone()));
              list.push(ModuleInfoOrReference::Concatenated(info));
            }
            ConcatenationEntryType::External => {
              let info = ExternalModuleInfo {
                index: i,
                module: module_id,
                runtime_condition: concatenation_entry.runtime_condition,
              };
              vac.insert(ModuleInfo::External(info.clone()));
              list.push(ModuleInfoOrReference::External(info));
            }
          };
        }
      }
    }
    (list, map)
  }

  fn create_concatenation_list(
    &self,
    root_module: ModuleIdentifier,
    mut module_set: IndexSet<ModuleIdentifier>,
    runtime: Option<&RuntimeSpec>,
    mg: &ModuleGraph,
  ) -> Vec<ConcatenationEntry> {
    let mut list = vec![];
    let mut exists_entries = HashMap::default();
    exists_entries.insert(root_module, RuntimeCondition::Boolean(true));

    let imports = self.get_concatenated_imports(&root_module, &root_module, runtime, mg);
    for i in imports {
      self.enter_module(
        root_module,
        &mut module_set,
        runtime,
        mg,
        i.connection,
        i.runtime_condition,
        &mut exists_entries,
        &mut list,
      );
    }
    list.push(ConcatenationEntry {
      ty: ConcatenationEntryType::Concatenated,
      connection_or_module_id: ConnectionOrModuleIdent::Module(root_module),
      runtime_condition: RuntimeCondition::Boolean(true),
    });
    list
  }

  fn enter_module(
    &self,
    root_module: ModuleIdentifier,
    module_set: &mut IndexSet<ModuleIdentifier>,
    runtime: Option<&RuntimeSpec>,
    mg: &ModuleGraph,
    con: ModuleGraphConnection,
    runtime_condition: RuntimeCondition,
    exists_entry: &mut HashMap<ModuleIdentifier, RuntimeCondition>,
    list: &mut Vec<ConcatenationEntry>,
  ) {
    let module = con.module_identifier;
    let exist_entry = match exists_entry.get(&module) {
      Some(condition) if matches!(condition, RuntimeCondition::Boolean(true)) => return,
      None => None,
      Some(_condtition) => Some(runtime_condition.clone()),
    };
    if module_set.contains(&module) {
      exists_entry.insert(module, RuntimeCondition::Boolean(true));
      if !matches!(runtime_condition, RuntimeCondition::Boolean(true)) {
        panic!(
          "Cannot runtime-conditional concatenate a module ({}) in {}. This should not happen.",
          module, self.root_module_ctxt.id,
        );
      }
      let imports = self.get_concatenated_imports(&module, &root_module, runtime, mg);
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
        connection_or_module_id: ConnectionOrModuleIdent::Connection(*con_id),
      })
    }
  }

  fn get_concatenated_imports(
    &self,
    module_id: &ModuleIdentifier,
    root_module_id: &ModuleIdentifier,
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
        connections.push(*c);
      }
    }

    let mut references = connections
      .into_iter()
      .filter_map(|connection| {
        let dep = connection.dependency_id.get_dependency(mg);
        if !is_harmony_dep_like(dep) {
          return None;
        }

        // TODO: we don't have resolved_original_module
        if !(connection.original_module_identifier == Some(*module_id)
          && connection.is_target_active(mg, self.runtime.as_ref()))
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
          (Some(_), None) => std::cmp::Ordering::Less,
          (Some(a), Some(b)) => a.cmp(&b),
        }
      }
    });

    let mut references_map = HashMap::default();
    for reference in references {
      let runtime_condition =
        filter_runtime(runtime, |r| reference.connection.is_target_active(mg, r));
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
            interop_namespace_object_used: false,
            interop_namespace_object_name: None,
            interop_namespace_object2_used: false,
            interop_namespace_object2_name: None,
            interop_default_access_used: false,
            interop_default_access_name: None,
          });
        }
      }
    }

    references_map.into_values().collect()
  }

  /// Using `ModuleIdentifier` instead of `ModuleInfo` to work around rustc borrow checker
  fn analyze_module(
    &self,
    compilation: &Compilation,
    module_info_map: Arc<IndexMap<ModuleIdentifier, ModuleInfo>>,
    info: ModuleInfo,
    runtime: Option<&RuntimeSpec>,
  ) -> Result<ModuleInfo> {
    if let ModuleInfo::Concatenated(mut info) = info {
      let module_id = info.module;
      let mut concatenation_scope = ConcatenationScope::new(module_info_map, info);
      let module = compilation
        .module_graph
        .module_by_identifier(&module_id)
        .expect("should have module");
      let mut codegen_res =
        module.code_generation(compilation, runtime, Some(concatenation_scope))?;
      let CodeGenerationResult {
        mut inner,
        data,
        chunk_init_fragments,
        runtime_requirements,
        hash,
        id,
        concatenation_scope,
      } = codegen_res;
      let concatenation_scope = concatenation_scope.expect("should have concatenation_scope");
      let source = inner
        .remove(&SourceType::JavaScript)
        .expect("should have javascript source");
      let source_code = source.source().to_string();

      let cm: Arc<swc_core::common::SourceMap> = Default::default();
      let fm = cm.new_source_file(
        FileName::Custom(format!(
          "{}",
          self.readable_identifier(&compilation.options.context),
        )),
        source_code,
      );
      let comments = SwcComments::default();
      let mut module_info = concatenation_scope.current_module;
      let mut errors = vec![];
      let program = match parse_file_as_module(
        &fm,
        Syntax::default(),
        EsVersion::EsNext,
        Some(&comments),
        &mut errors,
      ) {
        Ok(res) => Program::Module(res),
        Err(err) => {
          let span: ErrorSpan = err.span().into();
          self.diagnostics.lock().unwrap().append(
            &mut map_box_diagnostics_to_module_parse_diagnostics(vec![
              rspack_error::TraceableError::from_source_file(
                &fm,
                span.start as usize,
                span.end as usize,
                format!("JavaScript parsing error"),
                err.kind().msg().to_string(),
              )
              .with_kind(DiagnosticKind::JavaScript),
            ]),
          );
          return Ok(ModuleInfo::Concatenated(module_info));
        }
      };
      let mut ast = Ast::new(program, cm, Some(comments));

      let mut global_ctxt = SyntaxContext::empty();
      let mut module_ctxt = SyntaxContext::empty();

      ast.transform(|program, context| {
        global_ctxt = global_ctxt.apply_mark(context.unresolved_mark);
        module_ctxt = module_ctxt.apply_mark(context.top_level_mark);
        program.visit_mut_with(&mut resolver(
          context.unresolved_mark,
          context.top_level_mark,
          false,
        ));
      });

      let result_source = ReplaceSource::new(source.clone());
      module_info.module_ctxt = module_ctxt;
      module_info.global_ctxt = global_ctxt;
      module_info.ast = Some(ast);
      module_info.runtime_requirements = runtime_requirements;
      module_info.internal_source = Some(source);
      module_info.source = Some(result_source);
      module_info.chunk_init_fragments = chunk_init_fragments;
      return Ok(ModuleInfo::Concatenated(module_info));
    } else {
      Ok(info)
    }
  }

  fn find_new_name(
    old_name: &str,
    used_names1: &HashSet<String>,
    used_names2: Option<&HashSet<String>>,
    extra_info: String,
  ) -> String {
    let mut name = old_name.to_string();

    if name == DEFAULT_EXPORT {
      name = String::new();
    }
    if name == NAMESPACE_OBJECT_EXPORT {
      name = "namespaceObject".to_string();
    }

    // Remove uncool stuff
    let extra_info = extra_info
      .replace(
        |c: char| c == '.' || c == '/' || c == '+' || c.is_ascii_whitespace(),
        "",
      )
      .to_string();

    let mut splitted_info: Vec<&str> = extra_info.split('/').collect();
    while let Some(info_part) = splitted_info.pop() {
      name = format!("{}_{}", info_part, name);
      let name_ident = Template::to_identifier(&name);
      if !used_names1.contains(&name_ident)
        && (used_names2.is_none() || !used_names2.unwrap().contains(&name_ident))
      {
        return name_ident;
      }
    }

    let mut i = 0;
    let mut name_with_number = Template::to_identifier(&format!("{}_{}", name, i));
    while used_names1.contains(&name_with_number)
      || used_names2
        .map(|map| map.contains(&name_with_number))
        .unwrap_or_default()
    {
      i += 1;
      name_with_number = Template::to_identifier(&format!("{}_{}", name, i));
    }

    name_with_number
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

/// Mark boxed errors as [crate::diagnostics::ModuleParseError],
/// then, map it to diagnostics
pub fn map_box_diagnostics_to_module_parse_diagnostics(
  errors: Vec<TraceableError>,
) -> Vec<rspack_error::Diagnostic> {
  errors
    .into_iter()
    .map(|e| rspack_error::miette::Error::new(e).into())
    .collect()
}
