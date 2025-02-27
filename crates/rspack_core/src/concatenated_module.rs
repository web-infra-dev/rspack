use std::{
  borrow::Cow,
  collections::hash_map::Entry,
  fmt::Debug,
  hash::{BuildHasherDefault, Hasher},
  sync::{Arc, LazyLock},
};

use dashmap::DashMap;
use indexmap::IndexMap;
use rayon::prelude::*;
use regex::Regex;
use rspack_ast::javascript::Ast;
use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsMap, Skip},
};
use rspack_collections::{
  Identifiable, Identifier, IdentifierIndexMap, IdentifierIndexSet, IdentifierMap, IdentifierSet,
};
use rspack_error::{Diagnosable, Diagnostic, DiagnosticKind, Result, TraceableError};
use rspack_hash::{HashDigest, HashFunction, RspackHash};
use rspack_hook::define_hook;
use rspack_sources::{
  BoxSource, CachedSource, ConcatSource, RawStringSource, ReplaceSource, Source, SourceExt,
};
use rspack_util::{ext::DynHash, itoa, source_map::SourceMapKind, swc::join_atom};
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
  define_es_module_flag_statement, filter_runtime, impl_source_map_config, merge_runtime_condition,
  merge_runtime_condition_non_false, module_update_hash, property_access, property_name,
  reserved_names::RESERVED_NAMES, returning_function, runtime_condition_expression,
  subtract_runtime_condition, to_identifier, AsyncDependenciesBlockIdentifier, BoxDependency,
  BuildContext, BuildInfo, BuildMeta, BuildMetaDefaultObject, BuildMetaExportsType, BuildResult,
  ChunkGraph, ChunkInitFragments, CodeGenerationDataTopLevelDeclarations,
  CodeGenerationExportsFinalNames, CodeGenerationPublicPathAutoReplace, CodeGenerationResult,
  Compilation, ConcatenatedModuleIdent, ConcatenationScope, ConnectionState, Context,
  DependenciesBlock, DependencyId, DependencyTemplate, DependencyType, ErrorSpan,
  ExportInfoProvided, ExportsArgument, ExportsType, FactoryMeta, IdentCollector, LibIdentOptions,
  MaybeDynamicTargetExportInfoHashKey, Module, ModuleArgument, ModuleDependency, ModuleGraph,
  ModuleGraphConnection, ModuleIdentifier, ModuleLayer, ModuleType, Resolve, RuntimeCondition,
  RuntimeGlobals, RuntimeSpec, SourceType, SpanExt, Template, UsageState, UsedName, DEFAULT_EXPORT,
  NAMESPACE_OBJECT_EXPORT,
};

type ExportsDefinitionArgs = Vec<(String, String)>;
define_hook!(ConcatenatedModuleExportsDefinitions: SyncSeriesBail(exports_definitions: &mut ExportsDefinitionArgs, is_entry_module: bool) -> bool);

#[derive(Debug, Default)]
pub struct ConcatenatedModuleHooks {
  pub exports_definitions: ConcatenatedModuleExportsDefinitionsHook,
}

#[cacheable]
#[derive(Debug)]
pub struct RootModuleContext {
  pub id: ModuleIdentifier,
  pub readable_identifier: String,
  pub name_for_condition: Option<Box<str>>,
  pub lib_indent: Option<String>,
  pub resolve_options: Option<Arc<Resolve>>,
  pub code_generation_dependencies: Option<Vec<Box<dyn ModuleDependency>>>,
  pub presentational_dependencies: Option<Vec<Box<dyn DependencyTemplate>>>,
  pub context: Option<Context>,
  pub layer: Option<ModuleLayer>,
  pub side_effect_connection_state: ConnectionState,
  pub factory_meta: Option<FactoryMeta>,
  pub build_meta: BuildMeta,
  pub exports_argument: ExportsArgument,
  pub module_argument: ModuleArgument,
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct RawBinding {
  info_id: ModuleIdentifier,
  raw_name: Atom,
  comment: Option<String>,
  ids: Vec<Atom>,
  export_name: Vec<Atom>,
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct SymbolBinding {
  /// corresponding to a ConcatenatedModuleInfo, ref https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/optimize/ConcatenatedModule.js#L93-L100
  info_id: ModuleIdentifier,
  name: Atom,
  comment: Option<String>,
  ids: Vec<Atom>,
  export_name: Vec<Atom>,
}

#[derive(Debug, Clone)]
pub enum Binding {
  Raw(RawBinding),
  Symbol(SymbolBinding),
}

#[derive(Debug)]
pub enum BindingType {
  Raw,
  Symbol,
}

#[cacheable]
#[derive(Debug, Clone)]
pub struct ConcatenatedInnerModule {
  pub id: ModuleIdentifier,
  pub size: f64,
  pub original_source_hash: Option<u64>,
  pub shorten_id: String,
}

static REGEX: LazyLock<Regex> = LazyLock::new(|| {
  let pattern = r"\.+\/|(\/index)?\.([a-zA-Z0-9]{1,4})($|\s|\?)|\s*\+\s*\d+\s*modules";
  Regex::new(pattern).expect("should construct the regex")
});

#[derive(Debug)]
enum ConcatenationEntry {
  Concatenated(ConcatenationEntryConcatenated),
  External(ConcatenationEntryExternal),
}

impl ConcatenationEntry {
  pub fn module(&self, mg: &ModuleGraph) -> ModuleIdentifier {
    match self {
      ConcatenationEntry::Concatenated(c) => c.module,
      ConcatenationEntry::External(e) => e.module(mg),
    }
  }
}

#[derive(Debug)]
struct ConcatenationEntryConcatenated {
  module: ModuleIdentifier,
}

#[derive(Debug)]
struct ConcatenationEntryExternal {
  dependency: DependencyId,
  runtime_condition: RuntimeCondition,
}

impl ConcatenationEntryExternal {
  pub fn module(&self, mg: &ModuleGraph) -> ModuleIdentifier {
    let con = mg
      .connection_by_dependency_id(&self.dependency)
      .expect("should have connection");
    *con.module_identifier()
  }
}

#[derive(Debug)]
pub struct ConcatenatedModuleImportInfo<'a> {
  connection: &'a ModuleGraphConnection,
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
  pub internal_names: HashMap<Atom, Atom>,
  pub export_map: Option<HashMap<Atom, String>>,
  pub raw_export_map: Option<HashMap<Atom, String>>,
  pub namespace_object_name: Option<Atom>,
  pub interop_namespace_object_used: bool,
  pub interop_namespace_object_name: Option<Atom>,
  pub interop_namespace_object2_used: bool,
  pub interop_namespace_object2_name: Option<Atom>,
  pub interop_default_access_used: bool,
  pub interop_default_access_name: Option<Atom>,
  pub global_scope_ident: Vec<ConcatenatedModuleIdent>,
  pub idents: Vec<ConcatenatedModuleIdent>,
  pub binding_to_ref: HashMap<(Atom, SyntaxContext), Vec<ConcatenatedModuleIdent>>,

  pub public_path_auto_replace: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct ExternalModuleInfo {
  pub index: usize,
  pub module: ModuleIdentifier,
  pub runtime_condition: RuntimeCondition,
  pub interop_namespace_object_used: bool,
  pub interop_namespace_object_name: Option<Atom>,
  pub interop_namespace_object2_used: bool,
  pub interop_namespace_object2_name: Option<Atom>,
  pub interop_default_access_used: bool,
  pub interop_default_access_name: Option<Atom>,
  pub name: Option<Atom>,
}

pub struct ConnectionWithRuntimeCondition<'a> {
  pub connection: &'a ModuleGraphConnection,
  pub runtime_condition: RuntimeCondition,
}

#[derive(Debug, Clone)]
pub enum ModuleInfo {
  External(ExternalModuleInfo),
  Concatenated(Box<ConcatenatedModuleInfo>),
}

impl ModuleInfo {
  pub fn try_as_concatenated_mut(&mut self) -> Option<&mut ConcatenatedModuleInfo> {
    if let Self::Concatenated(ref mut v) = self {
      Some(v)
    } else {
      None
    }
  }

  pub fn try_as_concatenated(&self) -> Option<&ConcatenatedModuleInfo> {
    if let Self::Concatenated(ref v) = self {
      Some(v)
    } else {
      None
    }
  }
  /// # Panic
  /// This method would panic if the conversion is failed.
  pub fn as_concatenated_mut(&mut self) -> &mut ConcatenatedModuleInfo {
    if let Self::Concatenated(ref mut v) = self {
      v
    } else {
      panic!("should convert as concatenated module info")
    }
  }

  pub fn as_concatenated(&self) -> &ConcatenatedModuleInfo {
    if let Self::Concatenated(ref v) = self {
      v
    } else {
      panic!("should convert as concatenated module info")
    }
  }

  pub fn id(&self) -> ModuleIdentifier {
    match self {
      ModuleInfo::External(e) => e.module,
      ModuleInfo::Concatenated(c) => c.module,
    }
  }

  pub fn set_interop_namespace_object_used(&mut self, v: bool) {
    match self {
      ModuleInfo::External(e) => e.interop_namespace_object_used = v,
      ModuleInfo::Concatenated(c) => c.interop_namespace_object_used = v,
    }
  }

  pub fn set_interop_namespace_object_name(&mut self, v: Option<Atom>) {
    match self {
      ModuleInfo::External(e) => e.interop_namespace_object_name = v,
      ModuleInfo::Concatenated(c) => c.interop_namespace_object_name = v,
    }
  }

  pub fn set_interop_namespace_object2_used(&mut self, v: bool) {
    match self {
      ModuleInfo::External(e) => e.interop_namespace_object2_used = v,
      ModuleInfo::Concatenated(c) => c.interop_namespace_object2_used = v,
    }
  }

  pub fn set_interop_namespace_object2_name(&mut self, v: Option<Atom>) {
    match self {
      ModuleInfo::External(e) => e.interop_namespace_object2_name = v,
      ModuleInfo::Concatenated(c) => c.interop_namespace_object2_name = v,
    }
  }

  pub fn get_interop_namespace_object_used(&self) -> bool {
    match self {
      ModuleInfo::External(e) => e.interop_namespace_object_used,
      ModuleInfo::Concatenated(c) => c.interop_namespace_object_used,
    }
  }

  pub fn get_interop_namespace_object_name(&self) -> Option<&Atom> {
    match self {
      ModuleInfo::External(e) => e.interop_namespace_object_name.as_ref(),
      ModuleInfo::Concatenated(c) => c.interop_namespace_object_name.as_ref(),
    }
  }

  pub fn get_interop_namespace_object2_used(&self) -> bool {
    match self {
      ModuleInfo::External(e) => e.interop_namespace_object2_used,
      ModuleInfo::Concatenated(c) => c.interop_namespace_object2_used,
    }
  }

  pub fn get_interop_namespace_object2_name(&self) -> Option<&Atom> {
    match self {
      ModuleInfo::External(e) => e.interop_namespace_object2_name.as_ref(),
      ModuleInfo::Concatenated(c) => c.interop_namespace_object2_name.as_ref(),
    }
  }

  pub fn get_interop_default_access_name(&self) -> Option<&Atom> {
    match self {
      ModuleInfo::External(e) => e.interop_default_access_name.as_ref(),
      ModuleInfo::Concatenated(c) => c.interop_default_access_name.as_ref(),
    }
  }

  pub fn get_interop_default_access_used(&self) -> bool {
    match self {
      ModuleInfo::External(e) => e.interop_default_access_used,
      ModuleInfo::Concatenated(c) => c.interop_default_access_used,
    }
  }
  pub fn set_interop_default_access_used(&mut self, v: bool) {
    match self {
      ModuleInfo::External(e) => e.interop_default_access_used = v,
      ModuleInfo::Concatenated(c) => c.interop_default_access_used = v,
    }
  }

  pub fn set_interop_default_access_name(&mut self, v: Option<Atom>) {
    match self {
      ModuleInfo::External(e) => e.interop_default_access_name = v,
      ModuleInfo::Concatenated(c) => c.interop_default_access_name = v,
    }
  }
}

impl ModuleInfo {
  pub fn index(&self) -> usize {
    match self {
      ModuleInfo::External(e) => e.index,
      ModuleInfo::Concatenated(c) => c.index,
    }
  }
}

#[impl_source_map_config]
#[cacheable]
#[derive(Debug)]
pub struct ConcatenatedModule {
  id: ModuleIdentifier,
  /// Used to implementing [Module] trait for [ConcatenatedModule]
  root_module_ctxt: RootModuleContext,
  modules: Vec<ConcatenatedInnerModule>,
  runtime: Option<RuntimeSpec>,

  blocks: Vec<AsyncDependenciesBlockIdentifier>,
  dependencies: Vec<DependencyId>,
  #[cacheable(with=AsMap)]
  cached_source_sizes: DashMap<SourceType, f64, BuildHasherDefault<FxHasher>>,
  #[cacheable(with=Skip)]
  diagnostics: Vec<Diagnostic>,
  build_info: BuildInfo,
}

#[allow(unused)]
impl ConcatenatedModule {
  pub fn new(
    id: ModuleIdentifier,
    root_module_ctxt: RootModuleContext,
    mut modules: Vec<ConcatenatedInnerModule>,
    runtime: Option<RuntimeSpec>,
  ) -> Self {
    // make the hash consistent
    modules.sort_by(|a, b| a.id.cmp(&b.id));
    let RootModuleContext {
      module_argument,
      exports_argument,
      ..
    } = root_module_ctxt;
    Self {
      id,
      root_module_ctxt,
      modules,
      runtime,
      dependencies: vec![],
      blocks: vec![],
      cached_source_sizes: DashMap::default(),
      diagnostics: vec![],
      build_info: BuildInfo {
        cacheable: true,
        strict: true,
        module_argument,
        exports_argument,
        top_level_declarations: Some(Default::default()),
        ..Default::default()
      },
      source_map_kind: SourceMapKind::empty(),
    }
  }

  // TODO: caching https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/optimize/ConcatenatedModule.js#L663-L664
  pub fn create(
    root_module_ctxt: RootModuleContext,
    modules: Vec<ConcatenatedInnerModule>,
    hash_function: Option<HashFunction>,
    runtime: Option<RuntimeSpec>,
    compilation: &Compilation,
  ) -> Self {
    let id = Self::create_identifier(&root_module_ctxt, &modules, hash_function);
    Self::new(id.as_str().into(), root_module_ctxt, modules, runtime)
  }

  fn create_identifier(
    root_module_ctxt: &RootModuleContext,
    modules: &Vec<ConcatenatedInnerModule>,
    hash_function: Option<HashFunction>,
  ) -> String {
    let mut identifiers = vec![];
    for m in modules {
      identifiers.push(m.shorten_id.as_str());
    }
    identifiers.sort_unstable();
    let mut hash = RspackHash::new(&hash_function.unwrap_or(HashFunction::MD4));
    if let Some(id) = identifiers.first() {
      hash.write(id.as_bytes());
    }
    for id in identifiers.iter().skip(1) {
      hash.write(b" ");
      hash.write(id.as_bytes());
    }
    let res = hash.digest(&HashDigest::Hex);
    format!("{}|{}", root_module_ctxt.id, res.encoded())
  }

  pub fn id(&self) -> ModuleIdentifier {
    self.id
  }

  pub fn get_root(&self) -> ModuleIdentifier {
    self.root_module_ctxt.id
  }

  pub fn get_modules(&self) -> &[ConcatenatedInnerModule] {
    self.modules.as_slice()
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

  fn remove_dependency_id(&mut self, dependency: DependencyId) {
    self.dependencies.retain(|d| d != &dependency)
  }

  fn get_dependencies(&self) -> &[DependencyId] {
    &self.dependencies
  }
}

#[cacheable_dyn]
#[async_trait::async_trait]
impl Module for ConcatenatedModule {
  fn module_type(&self) -> &ModuleType {
    // https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/optimize/ConcatenatedModule.js#L688
    &ModuleType::JsEsm
  }

  fn factory_meta(&self) -> Option<&FactoryMeta> {
    self.root_module_ctxt.factory_meta.as_ref()
  }

  fn set_factory_meta(&mut self, v: FactoryMeta) {
    self.root_module_ctxt.factory_meta = Some(v);
  }

  fn build_info(&self) -> &BuildInfo {
    &self.build_info
  }

  fn build_info_mut(&mut self) -> &mut BuildInfo {
    &mut self.build_info
  }

  fn build_meta(&self) -> &BuildMeta {
    &self.root_module_ctxt.build_meta
  }

  fn build_meta_mut(&mut self) -> &mut BuildMeta {
    &mut self.root_module_ctxt.build_meta
  }

  fn source_types(&self) -> &[SourceType] {
    &[SourceType::JavaScript]
  }

  fn source(&self) -> Option<&BoxSource> {
    None
  }

  fn get_layer(&self) -> Option<&ModuleLayer> {
    self.root_module_ctxt.layer.as_ref()
  }

  fn readable_identifier(&self, _context: &Context) -> Cow<str> {
    Cow::Owned(format!(
      "{} + {} modules",
      self.root_module_ctxt.readable_identifier,
      itoa!(self.modules.len() - 1)
    ))
  }

  fn size(&self, source_type: Option<&SourceType>, _compilation: Option<&Compilation>) -> f64 {
    if let Some(size_ref) = source_type.and_then(|st| self.cached_source_sizes.get(st)) {
      *size_ref
    } else {
      let size = self.modules.iter().fold(0.0, |acc, cur| acc + cur.size);
      source_type.and_then(|st| self.cached_source_sizes.insert(*st, size));
      size
    }
  }

  /// the compilation is asserted to be `Some(Compilation)`, https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/optimize/ModuleConcatenationPlugin.js#L394-L418
  async fn build(
    &mut self,
    _build_context: BuildContext,
    compilation: Option<&Compilation>,
  ) -> Result<BuildResult> {
    let compilation = compilation.expect("should pass compilation");

    let module_graph = compilation.get_module_graph();
    let modules = self
      .modules
      .iter()
      .map(|item| Some(&item.id))
      .collect::<HashSet<_>>();
    for m in self.modules.iter() {
      let module = module_graph
        .module_by_identifier(&m.id)
        .expect("should have module");
      let cur_build_info = module.build_info();

      // populate cacheable
      if !cur_build_info.cacheable {
        self.build_info.cacheable = false;
      }

      // populate dependencies
      for dep_id in module.get_dependencies().iter() {
        let dep = module_graph
          .dependency_by_id(dep_id)
          .expect("should have dependency");
        let module_id_of_dep = module_graph.module_identifier_by_dependency_id(dep_id);
        if !is_esm_dep_like(dep) || !modules.contains(&module_id_of_dep) {
          self.dependencies.push(*dep_id);
        }
      }

      // populate blocks
      for b in module.get_blocks() {
        self.blocks.push(*b);
      }
      // populate diagnostic
      self.diagnostics.extend(module.diagnostics().into_owned());

      // populate topLevelDeclarations
      let module_build_info = module.build_info();
      if let Some(decls) = &module_build_info.top_level_declarations
        && let Some(top_level_declarations) = &mut self.build_info.top_level_declarations
      {
        top_level_declarations.extend(decls.iter().cloned());
      } else {
        self.build_info.top_level_declarations = None;
      }

      // populate assets
      self
        .build_info
        .assets
        .extend(module_build_info.assets.clone());
    }
    // return a dummy result is enough, since we don't build the ConcatenatedModule in make phase
    Ok(BuildResult::default())
  }

  // #[tracing::instrument("ConcatenatedModule::code_generation", skip_all, fields(identifier = ?self.identifier()))]
  fn code_generation(
    &self,
    compilation: &Compilation,
    generation_runtime: Option<&RuntimeSpec>,
    _: Option<ConcatenationScope>,
  ) -> Result<CodeGenerationResult> {
    let mut runtime_requirements = RuntimeGlobals::default();
    let runtime = if let Some(self_runtime) = &self.runtime
      && let Some(generation_runtime) = generation_runtime
    {
      Some(Cow::Owned(
        generation_runtime
          .intersection(self_runtime)
          .cloned()
          .collect::<RuntimeSpec>(),
      ))
    } else {
      generation_runtime.map(Cow::Borrowed)
    };
    let runtime = runtime.as_deref();
    let context = compilation.options.context.clone();

    let (modules_with_info, module_to_info_map) =
      self.get_modules_with_info(&compilation.get_module_graph(), runtime);

    // Set with modules that need a generated namespace object
    let mut needed_namespace_objects = IdentifierIndexSet::default();

    // Generate source code and analyze scopes
    // Prepare a ReplaceSource for the final source
    //
    let arc_map = Arc::new(module_to_info_map);
    let tmp: Vec<rspack_error::Result<(rspack_collections::Identifier, ModuleInfo)>> = arc_map
      .par_iter()
      .map(|(id, info)| {
        let updated_module_info =
          self.analyze_module(compilation, Arc::clone(&arc_map), info.clone(), runtime)?;
        Ok((*id, updated_module_info))
      })
      .collect::<Vec<_>>();

    let mut updated_pairs = vec![];
    for item in tmp {
      updated_pairs.push(item?);
    }

    let mut module_to_info_map = Arc::into_inner(arc_map).expect("reference count should be one");

    for (id, module_info) in updated_pairs {
      module_to_info_map.insert(id, module_info);
    }

    let mut all_used_names: HashSet<Atom> = RESERVED_NAMES.iter().map(|s| Atom::new(*s)).collect();
    let mut top_level_declarations: HashSet<Atom> = HashSet::default();
    let mut public_path_auto_replace: bool = false;

    for (module_info_id, _raw_condition) in modules_with_info.iter() {
      let Some(ModuleInfo::Concatenated(info)) = module_to_info_map.get_mut(module_info_id) else {
        continue;
      };
      if let Some(ref ast) = info.ast {
        let mut collector = IdentCollector::default();
        ast.visit(|program, _ctxt| {
          program.visit_with(&mut collector);
        });
        for ident in collector.ids {
          if ident.id.ctxt == info.global_ctxt {
            info.global_scope_ident.push(ident.clone());
            all_used_names.insert(ident.id.sym.clone());
          }
          if ident.is_class_expr_with_ident {
            all_used_names.insert(ident.id.sym.clone());
            continue;
          }
          // deconflict naming from inner scope, the module level deconflict will be finished
          // you could see tests/webpack-test/cases/scope-hoisting/renaming-4967 as a example
          // during module eval phase.
          if ident.id.ctxt != info.module_ctxt {
            all_used_names.insert(ident.id.sym.clone());
          }
          info.idents.push(ident);
        }
        let mut binding_to_ref: HashMap<(Atom, SyntaxContext), Vec<ConcatenatedModuleIdent>> =
          HashMap::default();

        for ident in info.idents.iter() {
          match binding_to_ref.entry((ident.id.sym.clone(), ident.id.ctxt)) {
            Entry::Occupied(mut occ) => {
              occ.get_mut().push(ident.clone());
            }
            Entry::Vacant(vac) => {
              vac.insert(vec![ident.clone()]);
            }
          };
        }
        info.binding_to_ref = binding_to_ref;
      }
    }

    let module_graph = compilation.get_module_graph();
    for info in module_to_info_map.values_mut() {
      // Get used names in the scope

      let module = module_graph
        .module_by_identifier(&info.id())
        .expect("should have module identifier");
      let readable_identifier = module.readable_identifier(&context);
      let exports_type: BuildMetaExportsType = module.build_meta().exports_type;
      let default_object: BuildMetaDefaultObject = module.build_meta().default_object;
      match info {
        // Handle concatenated type
        ModuleInfo::Concatenated(info) => {
          // Iterate over variables in moduleScope
          for (id, refs) in info.binding_to_ref.iter() {
            let name = &id.0;
            let ctxt = id.1;
            if ctxt != info.module_ctxt {
              continue;
            }
            // Check if the name is already used
            if all_used_names.contains(name) {
              // Find a new name and update references
              let new_name = find_new_name(name, &all_used_names, None, &readable_identifier);
              all_used_names.insert(new_name.clone());
              info.internal_names.insert(name.clone(), new_name.clone());
              top_level_declarations.insert(new_name.clone());

              // Update source
              let source = info.source.as_mut().expect("should have source");

              for identifier in refs {
                let span = identifier.id.span();
                let low = span.real_lo();
                let high = span.real_hi();
                if identifier.shorthand {
                  source.insert(high, &format!(": {new_name}"), None);
                  continue;
                }

                source.replace(low, high, &new_name, None);
              }
            } else {
              // Handle the case when the name is not already used
              all_used_names.insert(name.clone());
              info.internal_names.insert(name.clone(), name.clone());
              top_level_declarations.insert(name.clone());
            }
          }

          // Handle the name passed through by namespace_export_symbol
          if let Some(ref namespace_export_symbol) = info.namespace_export_symbol {
            if namespace_export_symbol.starts_with(NAMESPACE_OBJECT_EXPORT)
              && namespace_export_symbol.len() > NAMESPACE_OBJECT_EXPORT.len()
            {
              let name =
                Atom::from(namespace_export_symbol[NAMESPACE_OBJECT_EXPORT.len()..].to_string());
              all_used_names.insert(name.clone());
              info
                .internal_names
                .insert(namespace_export_symbol.clone(), name.clone());
              top_level_declarations.insert(name.clone());
            }
          }

          // Handle namespaceObjectName for concatenated type
          let namespace_object_name =
            if let Some(ref namespace_export_symbol) = info.namespace_export_symbol {
              info.internal_names.get(namespace_export_symbol).cloned()
            } else {
              Some(find_new_name(
                "namespaceObject",
                &all_used_names,
                None,
                &readable_identifier,
              ))
            };
          if let Some(namespace_object_name) = namespace_object_name {
            all_used_names.insert(namespace_object_name.clone());
            info.namespace_object_name = Some(namespace_object_name.clone());
            top_level_declarations.insert(namespace_object_name);
          }

          // Handle publicPathAutoReplace for perf
          if let Some(info_auto) = info.public_path_auto_replace {
            public_path_auto_replace = public_path_auto_replace || info_auto;
          }
        }

        // Handle external type
        ModuleInfo::External(info) => {
          let external_name: Atom = find_new_name("", &all_used_names, None, &readable_identifier);
          all_used_names.insert(external_name.clone());
          info.name = Some(external_name.clone());
          top_level_declarations.insert(external_name.clone());
        }
      }
      // Handle additional logic based on module build meta
      if exports_type != BuildMetaExportsType::Namespace {
        let external_name_interop: Atom = find_new_name(
          "namespaceObject",
          &all_used_names,
          None,
          &readable_identifier,
        );
        all_used_names.insert(external_name_interop.clone());
        info.set_interop_namespace_object_name(Some(external_name_interop.clone()));
        top_level_declarations.insert(external_name_interop.clone());
      }

      if exports_type == BuildMetaExportsType::Default
        && !matches!(default_object, BuildMetaDefaultObject::Redirect)
      {
        let external_name_interop: Atom = find_new_name(
          "namespaceObject2",
          &all_used_names,
          None,
          &readable_identifier,
        );
        all_used_names.insert(external_name_interop.clone());
        info.set_interop_namespace_object2_name(Some(external_name_interop.clone()));
        top_level_declarations.insert(external_name_interop.clone());
      }

      if matches!(
        exports_type,
        BuildMetaExportsType::Dynamic | BuildMetaExportsType::Unset
      ) {
        let external_name_interop: Atom =
          find_new_name("default", &all_used_names, None, &readable_identifier);
        all_used_names.insert(external_name_interop.clone());
        info.set_interop_default_access_name(Some(external_name_interop.clone()));
        top_level_declarations.insert(external_name_interop.clone());
      }
    }

    let module_graph = compilation.get_module_graph();
    let mut info_map: IdentifierIndexMap<Vec<_>> = IdentifierIndexMap::default();
    // Find and replace references to modules
    // Splitting read and write to avoid violating rustc borrow rules
    for info in module_to_info_map.values() {
      if let ModuleInfo::Concatenated(info) = info {
        let module = module_graph
          .module_by_identifier(&info.module)
          .expect("should have module");
        let build_meta = module.build_meta();
        let mut refs = vec![];
        for reference in info.global_scope_ident.iter() {
          let name = &reference.id.sym;
          let match_result = ConcatenationScope::match_module_reference(name.as_str());
          if let Some(match_info) = match_result {
            let referenced_info_id = &modules_with_info[match_info.index].0;
            refs.push((
              reference.clone(),
              referenced_info_id,
              match_info
                .ids
                .into_iter()
                .map(|item| Atom::from(item.as_str()))
                .collect::<Vec<_>>(),
              match_info.call,
              !match_info.direct_import,
              build_meta.strict_esm_module,
              match_info.asi_safe,
            ));
          }
        }
        info_map.insert(info.module, refs);
      }
    }

    for (module_info_id, info_params_list) in info_map {
      for (
        reference_ident,
        referenced_info_id,
        export_name,
        call,
        call_context,
        strict_esm_module,
        asi_safe,
      ) in info_params_list
      {
        let final_name = Self::get_final_name(
          &compilation.get_module_graph(),
          referenced_info_id,
          export_name,
          &mut module_to_info_map,
          runtime,
          &mut needed_namespace_objects,
          call,
          call_context,
          strict_esm_module,
          asi_safe,
          &context,
        );
        // We assume this should be concatenated module info because previous loop
        let info = module_to_info_map
          .get_mut(&module_info_id)
          .and_then(|info| info.try_as_concatenated_mut())
          .expect("should have concatenate module info");
        let span = reference_ident.id.span();
        let low = span.real_lo();
        let high = span.real_hi();
        let source = info.source.as_mut().expect("should have source");
        // range is extended by 2 chars to cover the appended "._"
        // https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/optimize/ConcatenatedModule.js#L1411-L1412
        source.replace(low, high + 2, &final_name, None);
      }
    }

    let mut exports_map: HashMap<Atom, String> = HashMap::default();
    let mut unused_exports: HashSet<Atom> = HashSet::default();

    let root_info = module_to_info_map
      .get(&self.root_module_ctxt.id)
      .expect("should have root module");
    let root_module_id = root_info.id();

    let module_graph = compilation.get_module_graph();
    let root_module = module_graph
      .module_by_identifier(&root_module_id)
      .expect("should have box module");
    let strict_esm_module = root_module.build_meta().strict_esm_module;

    let exports_info = module_graph.get_exports_info(&root_module_id);
    let mut exports_final_names: Vec<(String, String)> = vec![];

    for export_info in exports_info.ordered_exports(&module_graph) {
      let name = export_info
        .name(&module_graph)
        .cloned()
        .unwrap_or("".into());
      if matches!(
        export_info.provided(&module_graph),
        Some(ExportInfoProvided::False)
      ) {
        continue;
      }
      let used_name = export_info.get_used_name(&module_graph, None, runtime);

      let Some(used_name) = used_name else {
        unused_exports.insert(name);
        continue;
      };
      exports_map.insert(used_name.clone(), {
        let final_name = Self::get_final_name(
          &compilation.get_module_graph(),
          &root_module_id,
          [name.clone()].to_vec(),
          &mut module_to_info_map,
          runtime,
          &mut needed_namespace_objects,
          false,
          false,
          strict_esm_module,
          Some(true),
          &compilation.options.context,
        );
        exports_final_names.push((used_name.to_string(), final_name.clone()));
        format!(
          "/* {} */ {}",
          if export_info.is_reexport(&module_graph) {
            "reexport"
          } else {
            "binding"
          },
          final_name
        )
      });
    }

    let mut result = ConcatSource::default();
    let mut should_add_esm_flag = false;

    // Add ESM compatibility flag (must be first because of possible circular dependencies)
    if compilation
      .get_module_graph()
      .get_exports_info(&self.id())
      .other_exports_info(&module_graph)
      .get_used(&module_graph, runtime)
      != UsageState::Unused
    {
      should_add_esm_flag = true
    }

    // Assuming the necessary imports and dependencies are declared

    // Define exports
    if !exports_map.is_empty() {
      let mut definitions = Vec::new();
      for (key, value) in exports_map.iter() {
        definitions.push(format!(
          "\n  {}: {}",
          property_name(key).expect("should convert to property_name"),
          returning_function(&compilation.options.output.environment, value, "")
        ));
      }

      let exports_argument = self.get_exports_argument();

      let should_skip_render_definitions = compilation
        .plugin_driver
        .concatenated_module_hooks
        .exports_definitions
        .call(
          &mut exports_final_names,
          compilation.chunk_graph.is_entry_module(&self.id),
        )?;

      if !matches!(should_skip_render_definitions, Some(true)) {
        runtime_requirements.insert(RuntimeGlobals::EXPORTS);
        runtime_requirements.insert(RuntimeGlobals::DEFINE_PROPERTY_GETTERS);

        if should_add_esm_flag {
          result.add(RawStringSource::from_static("// ESM COMPAT FLAG\n"));
          result.add(RawStringSource::from(define_es_module_flag_statement(
            self.get_exports_argument(),
            &mut runtime_requirements,
          )));
        }

        result.add(RawStringSource::from_static("\n// EXPORTS\n"));
        result.add(RawStringSource::from(format!(
          "{}({}, {{{}\n}});\n",
          RuntimeGlobals::DEFINE_PROPERTY_GETTERS,
          exports_argument,
          definitions.join(",")
        )));
      }
    }

    // List unused exports
    if !unused_exports.is_empty() {
      result.add(RawStringSource::from(format!(
        "\n// UNUSED EXPORTS: {}\n",
        join_atom(unused_exports.iter(), ", ")
      )));
    }

    let mut namespace_object_sources: IdentifierMap<String> = IdentifierMap::default();

    let mut visited = HashSet::default();
    // webpack require iterate the needed_namespace_objects and mutate `needed_namespace_objects`
    // at the same time, https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/optimize/ConcatenatedModule.js#L1514
    // Which is impossible in rust, using a fixed point algorithm  to reach the same goal.
    loop {
      let mut changed = false;
      // using the previous round snapshot `needed_namespace_objects` to iterate, and modify the
      // original `needed_namespace_objects` during the iteration,
      // if there is no new id inserted into `needed_namespace_objects`, break the outer loop
      for module_info_id in needed_namespace_objects.clone().iter() {
        if visited.contains(module_info_id) {
          continue;
        }
        visited.insert(*module_info_id);
        changed = true;

        let module_info = module_to_info_map
          .get(module_info_id)
          .map(|m| m.as_concatenated())
          .expect("should have module info");

        let module_graph = compilation.get_module_graph();
        let box_module = module_graph
          .module_by_identifier(module_info_id)
          .expect("should have box module");
        let module_readable_identifier = box_module.readable_identifier(&context);
        let strict_esm_module = box_module.build_meta().strict_esm_module;
        let name_space_name = module_info.namespace_object_name.clone();

        if let Some(ref _namespace_export_symbol) = module_info.namespace_export_symbol {
          continue;
        }

        let mut ns_obj = Vec::new();
        let exports_info = module_graph.get_exports_info(module_info_id);
        for export_info in exports_info.ordered_exports(&module_graph) {
          if matches!(
            export_info.provided(&module_graph),
            Some(ExportInfoProvided::False)
          ) {
            continue;
          }

          if let Some(used_name) = export_info.get_used_name(&module_graph, None, runtime) {
            let final_name = Self::get_final_name(
              &compilation.get_module_graph(),
              module_info_id,
              vec![export_info
                .name(&module_graph)
                .cloned()
                .unwrap_or("".into())],
              &mut module_to_info_map,
              runtime,
              &mut needed_namespace_objects,
              false,
              false,
              strict_esm_module,
              Some(true),
              &context,
            );

            ns_obj.push(format!(
              "\n  {}: {}",
              property_name(&used_name).expect("should have property_name"),
              returning_function(&compilation.options.output.environment, &final_name, "")
            ));
          }
        }
        // https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/optimize/ConcatenatedModule.js#L1539
        let name = name_space_name.expect("should have name_space_name");
        let define_getters = if !ns_obj.is_empty() {
          format!(
            "{}({}, {{ {} }});\n",
            RuntimeGlobals::DEFINE_PROPERTY_GETTERS,
            name,
            ns_obj.join(",")
          )
        } else {
          String::new()
        };

        if !ns_obj.is_empty() {
          runtime_requirements.insert(RuntimeGlobals::DEFINE_PROPERTY_GETTERS);
        }

        namespace_object_sources.insert(
          *module_info_id,
          format!(
            "// NAMESPACE OBJECT: {}\nvar {} = {{}};\n{}({});\n{}\n",
            module_readable_identifier,
            name,
            RuntimeGlobals::MAKE_NAMESPACE_OBJECT,
            name,
            define_getters
          ),
        );

        runtime_requirements.insert(RuntimeGlobals::MAKE_NAMESPACE_OBJECT);
      }
      if !changed {
        break;
      }
    }

    // Define required namespace objects (must be before evaluation modules)
    for info in module_to_info_map.values() {
      let Some(info) = info.try_as_concatenated() else {
        continue;
      };

      if let Some(source) = namespace_object_sources.get(&info.module) {
        result.add(RawStringSource::from(source.as_str()));
      }
    }

    let mut chunk_init_fragments = Vec::new();

    // Evaluate modules in order
    let module_graph = compilation.get_module_graph();
    for (module_info_id, item_runtime_condition) in modules_with_info {
      let name;
      let mut is_conditional = false;
      let info = module_to_info_map
        .get(&module_info_id)
        .expect("should have module info");
      let box_module = module_graph
        .module_by_identifier(&module_info_id)
        .expect("should have box module");
      let module_readable_identifier = box_module.readable_identifier(&context);

      match info {
        ModuleInfo::Concatenated(info) => {
          result.add(RawStringSource::from(
            format!("\n;// CONCATENATED MODULE: {module_readable_identifier}\n").as_str(),
          ));
          // https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/optimize/ConcatenatedModule.js#L1582
          result.add(info.source.clone().expect("should have source"));

          for f in info.chunk_init_fragments.iter() {
            chunk_init_fragments.push(f.clone());
          }

          runtime_requirements = runtime_requirements.union(info.runtime_requirements);
          name = info.namespace_object_name.clone();
        }
        ModuleInfo::External(info) => {
          result.add(RawStringSource::from(format!(
            "\n// EXTERNAL MODULE: {module_readable_identifier}\n"
          )));

          runtime_requirements.insert(RuntimeGlobals::REQUIRE);

          let condition = runtime_condition_expression(
            &compilation.chunk_graph,
            item_runtime_condition.as_ref(),
            runtime,
            &mut runtime_requirements,
          );

          if condition != "true" {
            is_conditional = true;
            result.add(RawStringSource::from(format!("if ({condition}) {{\n")));
          }

          result.add(RawStringSource::from(format!(
            "var {} = {}({});",
            info.name.as_ref().expect("should have name"),
            RuntimeGlobals::REQUIRE,
            serde_json::to_string(
              ChunkGraph::get_module_id(&compilation.module_ids_artifact, info.module)
                .expect("should have module id")
            )
            .expect("should json stringify module id")
          )));

          name = info.name.clone();
        }
      }

      if info.get_interop_namespace_object_used() {
        runtime_requirements.insert(RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT);
        result.add(RawStringSource::from(format!(
          "\nvar {} = /*#__PURE__*/{}({}, 2);",
          info
            .get_interop_namespace_object_name()
            .expect("should have interop_namespace_object_name"),
          RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT,
          name.as_ref().expect("should have name")
        )));
      }

      if info.get_interop_namespace_object2_used() {
        runtime_requirements.insert(RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT);
        result.add(RawStringSource::from(format!(
          "\nvar {} = /*#__PURE__*/{}({});",
          info
            .get_interop_namespace_object2_name()
            .expect("should have interop_namespace_object2_name"),
          RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT,
          name.as_ref().expect("should have name")
        )));
      }

      if info.get_interop_default_access_used() {
        runtime_requirements.insert(RuntimeGlobals::COMPAT_GET_DEFAULT_EXPORT);
        result.add(RawStringSource::from(format!(
          "\nvar {} = /*#__PURE__*/{}({});",
          info
            .get_interop_default_access_name()
            .expect("should have interop_default_access_name"),
          RuntimeGlobals::COMPAT_GET_DEFAULT_EXPORT,
          name.expect("should have name")
        )));
      }

      if is_conditional {
        result.add(RawStringSource::from_static("\n}"));
      }
    }

    let mut code_generation_result = CodeGenerationResult::default();
    code_generation_result.add(SourceType::JavaScript, CachedSource::new(result).boxed());
    code_generation_result.chunk_init_fragments = chunk_init_fragments;
    code_generation_result.runtime_requirements = runtime_requirements;

    if public_path_auto_replace {
      code_generation_result
        .data
        .insert(CodeGenerationPublicPathAutoReplace(true));
    }

    code_generation_result
      .data
      .insert(CodeGenerationDataTopLevelDeclarations::new(
        top_level_declarations,
      ));

    if !exports_final_names.is_empty() {
      let exports_final_names_map: HashMap<String, String> =
        exports_final_names.into_iter().collect();

      code_generation_result
        .data
        .insert(CodeGenerationExportsFinalNames::new(
          exports_final_names_map,
        ));
    }
    code_generation_result.set_hash(
      &compilation.options.output.hash_function,
      &compilation.options.output.hash_digest,
      &compilation.options.output.hash_salt,
    );
    Ok(code_generation_result)
  }

  fn update_hash(
    &self,
    hasher: &mut dyn std::hash::Hasher,
    compilation: &Compilation,
    generation_runtime: Option<&RuntimeSpec>,
  ) -> Result<()> {
    let runtime = if let Some(self_runtime) = &self.runtime
      && let Some(generation_runtime) = generation_runtime
    {
      Some(Cow::Owned(
        generation_runtime
          .intersection(self_runtime)
          .cloned()
          .collect::<RuntimeSpec>(),
      ))
    } else {
      generation_runtime.map(Cow::Borrowed)
    };
    let runtime = runtime.as_deref();
    for info in self.create_concatenation_list(
      self.root_module_ctxt.id,
      self.modules.iter().map(|item| item.id).collect(),
      runtime,
      &compilation.get_module_graph(),
    ) {
      match info {
        ConcatenationEntry::Concatenated(e) => compilation
          .get_module_graph()
          .module_by_identifier(&e.module)
          .expect("should have module")
          .update_hash(hasher, compilation, generation_runtime)?,
        ConcatenationEntry::External(e) => {
          ChunkGraph::get_module_id(
            &compilation.module_ids_artifact,
            e.module(&compilation.get_module_graph()),
          )
          .dyn_hash(hasher);
        }
      };
    }
    module_update_hash(self, hasher, compilation, generation_runtime);
    Ok(())
  }

  fn name_for_condition(&self) -> Option<Box<str>> {
    self.root_module_ctxt.name_for_condition.clone()
  }

  fn lib_ident(&self, _options: LibIdentOptions) -> Option<Cow<str>> {
    self.root_module_ctxt.lib_indent.clone().map(Cow::Owned)
  }

  fn get_resolve_options(&self) -> Option<Arc<Resolve>> {
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
    self.root_module_ctxt.context.clone().map(Box::new)
  }

  // Port from https://github.com/webpack/webpack/blob/main/lib/ConcatenatedModule.js#L1120
  fn get_side_effects_connection_state(
    &self,
    _module_graph: &ModuleGraph,
    _module_chain: &mut IdentifierSet,
  ) -> ConnectionState {
    self.root_module_ctxt.side_effect_connection_state
  }
}

impl Diagnosable for ConcatenatedModule {
  fn add_diagnostic(&mut self, diagnostic: Diagnostic) {
    self.diagnostics.push(diagnostic);
  }

  fn add_diagnostics(&mut self, mut diagnostics: Vec<Diagnostic>) {
    self.diagnostics.append(&mut diagnostics);
  }

  fn diagnostics(&self) -> Cow<[Diagnostic]> {
    Cow::Borrowed(&self.diagnostics)
  }
}

impl ConcatenatedModule {
  // TODO: replace self.modules with indexmap or linkedhashset
  fn get_modules_with_info(
    &self,
    mg: &ModuleGraph,
    runtime: Option<&RuntimeSpec>,
  ) -> (
    Vec<(ModuleIdentifier, Option<RuntimeCondition>)>,
    IdentifierIndexMap<ModuleInfo>,
  ) {
    let ordered_concatenation_list = self.create_concatenation_list(
      self.root_module_ctxt.id,
      self.modules.iter().map(|item| item.id).collect(),
      runtime,
      mg,
    );
    let mut list = vec![];
    let mut map = IdentifierIndexMap::default();
    for (i, concatenation_entry) in ordered_concatenation_list.into_iter().enumerate() {
      let module_id = concatenation_entry.module(mg);
      match map.entry(module_id) {
        indexmap::map::Entry::Occupied(_) => {
          let runtime_condition =
            if let ConcatenationEntry::External(ConcatenationEntryExternal {
              runtime_condition,
              ..
            }) = &concatenation_entry
            {
              Some(runtime_condition.clone())
            } else {
              None
            };
          list.push((module_id, runtime_condition));
        }
        indexmap::map::Entry::Vacant(vac) => {
          match concatenation_entry {
            ConcatenationEntry::Concatenated(_) => {
              let info = ConcatenatedModuleInfo {
                index: i,
                module: module_id,
                ..Default::default()
              };
              vac.insert(ModuleInfo::Concatenated(Box::new(info)));
              list.push((module_id, None));
            }
            ConcatenationEntry::External(e) => {
              let info = ExternalModuleInfo {
                index: i,
                module: module_id,
                runtime_condition: e.runtime_condition.clone(),
                interop_namespace_object_used: false,
                interop_namespace_object_name: None,
                interop_namespace_object2_used: false,
                interop_namespace_object2_name: None,
                interop_default_access_used: false,
                interop_default_access_name: None,
                name: None,
              };
              vac.insert(ModuleInfo::External(info));
              list.push((module_id, Some(e.runtime_condition)))
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
    mut module_set: IdentifierIndexSet,
    runtime: Option<&RuntimeSpec>,
    mg: &ModuleGraph,
  ) -> Vec<ConcatenationEntry> {
    let mut list = vec![];
    let mut exists_entries = IdentifierMap::default();
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
    list.push(ConcatenationEntry::Concatenated(
      ConcatenationEntryConcatenated {
        module: root_module,
      },
    ));
    list
  }

  #[allow(clippy::too_many_arguments)]
  fn enter_module(
    &self,
    root_module: ModuleIdentifier,
    module_set: &mut IdentifierIndexSet,
    runtime: Option<&RuntimeSpec>,
    mg: &ModuleGraph,
    con: &ModuleGraphConnection,
    runtime_condition: RuntimeCondition,
    exists_entry: &mut IdentifierMap<RuntimeCondition>,
    list: &mut Vec<ConcatenationEntry>,
  ) {
    let module = con.module_identifier();
    let exist_entry = match exists_entry.get(module) {
      Some(RuntimeCondition::Boolean(true)) => return,
      None => None,
      Some(condition) => Some(condition.clone()),
    };
    if module_set.contains(module) {
      exists_entry.insert(*module, RuntimeCondition::Boolean(true));
      if !matches!(runtime_condition, RuntimeCondition::Boolean(true)) {
        panic!(
          "Cannot runtime-conditional concatenate a module ({}) in {}. This should not happen.",
          module, self.root_module_ctxt.id,
        );
      }
      let imports = self.get_concatenated_imports(module, &root_module, runtime, mg);
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
      list.push(ConcatenationEntry::Concatenated(
        ConcatenationEntryConcatenated {
          module: *con.module_identifier(),
        },
      ));
    } else {
      let runtime_condition = if let Some(cond) = exist_entry {
        let reduced_runtime_condition =
          subtract_runtime_condition(&runtime_condition, &cond, runtime);
        if matches!(reduced_runtime_condition, RuntimeCondition::Boolean(false)) {
          return;
        }
        exists_entry.insert(
          *con.module_identifier(),
          merge_runtime_condition_non_false(&cond, &reduced_runtime_condition, runtime),
        );
        reduced_runtime_condition
      } else {
        exists_entry.insert(*con.module_identifier(), runtime_condition.clone());
        runtime_condition
      };

      if let Some(ConcatenationEntry::External(last)) = list.last_mut()
        && last.module(mg) == *con.module_identifier()
      {
        last.runtime_condition =
          merge_runtime_condition(&last.runtime_condition, &runtime_condition, runtime);
        return;
      }
      list.push(ConcatenationEntry::External(ConcatenationEntryExternal {
        dependency: con.dependency_id,
        runtime_condition,
      }));
    }
  }

  fn get_concatenated_imports<'a>(
    &self,
    module_id: &ModuleIdentifier,
    root_module_id: &ModuleIdentifier,
    runtime: Option<&RuntimeSpec>,
    mg: &'a ModuleGraph,
  ) -> Vec<ConnectionWithRuntimeCondition<'a>> {
    let mut connections = mg.get_outgoing_connections(module_id).collect::<Vec<_>>();
    if module_id == root_module_id {
      for c in mg.get_outgoing_connections(&self.id) {
        connections.push(c);
      }
    }

    let mut references = connections
      .into_iter()
      .filter_map(|connection| {
        let dep = mg
          .dependency_by_id(&connection.dependency_id)
          .expect("should have dependency");
        if !is_esm_dep_like(dep) {
          return None;
        }

        if !(connection.resolved_original_module_identifier == Some(*module_id)
          && connection.is_target_active(mg, self.runtime.as_ref()))
        {
          return None;
        }
        // now the dep should be one of `ESMExportImportedSpecifierDependency`, `ESMImportSideEffectDependency`, `ESMImportSpecifierDependency`,
        // the expect is safe now
        Some(ConcatenatedModuleImportInfo {
          connection,
          source_order: dep
            .source_order()
            .expect("source order should not be empty"),
          range_start: dep.range().map(|range| range.start),
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

    let mut references_map: IndexMap<ModuleIdentifier, ConnectionWithRuntimeCondition> =
      IndexMap::default();
    for reference in references {
      let runtime_condition =
        filter_runtime(runtime, |r| reference.connection.is_target_active(mg, r));
      if matches!(runtime_condition, RuntimeCondition::Boolean(false)) {
        continue;
      }

      let module: &Identifier = reference.connection.module_identifier();

      match references_map.entry(*module) {
        indexmap::map::Entry::Occupied(mut occ) => {
          let entry: &ConnectionWithRuntimeCondition = occ.get();
          let merged_condition = merge_runtime_condition_non_false(
            &entry.runtime_condition,
            &runtime_condition,
            runtime,
          );
          occ.get_mut().runtime_condition = merged_condition;
        }
        indexmap::map::Entry::Vacant(vac) => {
          vac.insert(ConnectionWithRuntimeCondition {
            connection: reference.connection,
            runtime_condition,
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
    module_info_map: Arc<IdentifierIndexMap<ModuleInfo>>,
    info: ModuleInfo,
    runtime: Option<&RuntimeSpec>,
  ) -> Result<ModuleInfo> {
    if let ModuleInfo::Concatenated(box info) = info {
      let module_id = info.module;

      let concatenation_scope = ConcatenationScope::new(module_info_map, info);
      let module_graph = compilation.get_module_graph();
      let module = module_graph
        .module_by_identifier(&module_id)
        .unwrap_or_else(|| panic!("should have module {module_id}"));
      let codegen_res = module.code_generation(compilation, runtime, Some(concatenation_scope))?;

      let CodeGenerationResult {
        mut inner,
        mut chunk_init_fragments,
        runtime_requirements,
        concatenation_scope,
        ..
      } = codegen_res;

      if let Some(fragments) = codegen_res.data.get::<ChunkInitFragments>() {
        chunk_init_fragments.extend(fragments.iter().cloned());
      }

      let concatenation_scope = concatenation_scope.expect("should have concatenation_scope");
      let source = inner
        .remove(&SourceType::JavaScript)
        .expect("should have javascript source");
      let source_code = source.source();

      let cm: Arc<swc_core::common::SourceMap> = Default::default();
      let fm = cm.new_source_file(
        Arc::new(FileName::Custom(format!(
          "{}",
          self.readable_identifier(&compilation.options.context),
        ))),
        source_code.into(),
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

          // return empty error as we already push error to compilation.diagnostics
          return Err(
            rspack_error::TraceableError::from_source_file(
              &fm,
              span.start as usize,
              span.end as usize,
              "JavaScript parsing error:\n".to_string(),
              err.kind().msg().to_string(),
            )
            .with_kind(DiagnosticKind::JavaScript)
            .into(),
          );
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
      if let Some(CodeGenerationPublicPathAutoReplace(true)) = codegen_res
        .data
        .get::<CodeGenerationPublicPathAutoReplace>(
      ) {
        module_info.public_path_auto_replace = Some(true);
      }
      Ok(ModuleInfo::Concatenated(Box::new(module_info)))
    } else {
      Ok(info)
    }
  }

  #[allow(clippy::too_many_arguments)]
  fn get_final_name(
    module_graph: &ModuleGraph,
    info: &ModuleIdentifier,
    export_name: Vec<Atom>,
    module_to_info_map: &mut IdentifierIndexMap<ModuleInfo>,
    runtime: Option<&RuntimeSpec>,
    needed_namespace_objects: &mut IdentifierIndexSet,
    as_call: bool,
    call_context: bool,
    strict_esm_module: bool,
    asi_safe: Option<bool>,
    context: &Context,
  ) -> String {
    let binding = Self::get_final_binding(
      module_graph,
      info,
      export_name.clone(),
      module_to_info_map,
      runtime,
      needed_namespace_objects,
      as_call,
      strict_esm_module,
      asi_safe,
      &mut HashSet::default(),
    );

    let (ids, comment) = match binding {
      Binding::Raw(ref b) => (&b.ids, b.comment.as_ref()),
      Binding::Symbol(ref b) => (&b.ids, b.comment.as_ref()),
    };

    let (reference, is_property_access) = match binding {
      Binding::Raw(ref b) => {
        let reference = format!(
          "{}{}{}",
          b.raw_name,
          comment.cloned().unwrap_or_default(),
          property_access(ids, 0)
        );
        let is_property_access = !ids.is_empty();
        (reference, is_property_access)
      }
      Binding::Symbol(ref binding) => {
        let export_id = &binding.name;
        let info = module_to_info_map
          .get(&binding.info_id)
          .and_then(|info| info.try_as_concatenated())
          .expect("should have concatenate module info");
        let module = module_graph
          .module_by_identifier(&info.module)
          .expect("should have module");
        let name = info.internal_names.get(export_id).unwrap_or_else(|| {
          panic!(
            "The export \"{}\" in \"{}\" has no internal name (existing names: {})",
            export_id,
            module.readable_identifier(context),
            info
              .internal_names
              .iter()
              .map(|(name, symbol)| format!("{name}: {symbol}"))
              .collect::<Vec<String>>()
              .join(", ")
          )
        });
        let reference = format!(
          "{}{}{}",
          name,
          comment.cloned().unwrap_or_default(),
          property_access(ids, 0)
        );
        let is_property_access = ids.len() > 1;
        (reference, is_property_access)
      }
    };
    if is_property_access && as_call && !call_context {
      return if asi_safe.unwrap_or_default() {
        format!("(0,{reference})")
      } else if let Some(_asi_safe) = asi_safe {
        format!(";(0,{reference})")
      } else {
        format!("/*#__PURE__*/Object({reference})")
      };
    }
    reference
  }

  #[allow(clippy::too_many_arguments)]
  fn get_final_binding(
    mg: &ModuleGraph,
    info_id: &ModuleIdentifier,
    mut export_name: Vec<Atom>,
    module_to_info_map: &mut IdentifierIndexMap<ModuleInfo>,
    runtime: Option<&RuntimeSpec>,
    needed_namespace_objects: &mut IdentifierIndexSet,
    as_call: bool,
    strict_esm_module: bool,
    asi_safe: Option<bool>,
    already_visited: &mut HashSet<MaybeDynamicTargetExportInfoHashKey>,
  ) -> Binding {
    let info = module_to_info_map
      .get(info_id)
      .expect("should have module info");

    let module = mg
      .module_by_identifier(&info.id())
      .expect("should have module");
    let exports_type = module.get_exports_type(mg, strict_esm_module);

    if export_name.is_empty() {
      match exports_type {
        ExportsType::DefaultOnly => {
          // shadowing the previous immutable ref to avoid violating rustc borrow rules
          let info = module_to_info_map
            .get_mut(info_id)
            .expect("should have module info");
          info.set_interop_namespace_object2_used(true);
          let raw_name = info.get_interop_namespace_object2_name();
          return Binding::Raw(RawBinding {
            info_id: info.id(),
            raw_name: raw_name.cloned().expect("should have raw name"),
            ids: export_name.clone(),
            export_name,
            comment: None,
          });
        }
        ExportsType::DefaultWithNamed => {
          // shadowing the previous immutable ref to avoid violating rustc borrow rules
          let info = module_to_info_map
            .get_mut(info_id)
            .expect("should have module info");
          info.set_interop_namespace_object_used(true);
          let raw_name = info
            .get_interop_namespace_object_name()
            .expect("should have interop_namespace_object_name");
          return Binding::Raw(RawBinding {
            info_id: info.id(),
            raw_name: raw_name.clone(),
            ids: export_name.clone(),
            export_name,
            comment: None,
          });
        }
        _ => {}
      }
    } else {
      match exports_type {
        ExportsType::Namespace => {}
        ExportsType::DefaultWithNamed => match export_name.first().map(|atom| atom.as_str()) {
          Some("default") => {
            export_name = export_name[1..].to_vec();
          }
          Some("__esModule") => {
            return Binding::Raw(RawBinding {
              info_id: info.id(),
              raw_name: "/* __esModule */true".into(),
              ids: export_name[1..].to_vec(),
              export_name,
              comment: None,
            });
          }
          _ => {}
        },
        ExportsType::DefaultOnly => {
          if export_name.first().map(|item| item.as_str()) == Some("__esModule") {
            return Binding::Raw(RawBinding {
              info_id: info.id(),
              raw_name: "/* __esModule */true".into(),
              ids: export_name[1..].to_vec(),
              export_name,
              comment: None,
            });
          }

          let first_export_id = export_name.remove(0);
          if first_export_id != "default" {
            return Binding::Raw(RawBinding {
              raw_name: "/* non-default import from default-exporting module */undefined".into(),
              ids: export_name.clone(),
              export_name,
              info_id: info.id(),
              comment: None,
            });
          }
        }
        ExportsType::Dynamic => match export_name.first().map(|atom| atom.as_str()) {
          Some("default") => {
            // shadowing the previous immutable ref to avoid violating rustc borrow rules
            let info = module_to_info_map
              .get_mut(info_id)
              .expect("should have module info");
            info.set_interop_default_access_used(true);
            export_name = export_name[1..].to_vec();
            // https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/optimize/ConcatenatedModule.js#L335-L341
            let default_access_name = info
              .get_interop_default_access_name()
              .cloned()
              .expect("should have default access name");
            let default_export = if as_call {
              format!("{default_access_name}()")
            } else if let Some(true) = asi_safe {
              format!("({default_access_name}())")
            } else if let Some(false) = asi_safe {
              format!(";({default_access_name}())")
            } else {
              format!("{default_access_name}.a")
            };

            return Binding::Raw(RawBinding {
              raw_name: default_export.into(),
              ids: export_name.clone(),
              export_name,
              info_id: info.id(),
              comment: None,
            });
          }
          Some("__esModule") => {
            return Binding::Raw(RawBinding {
              raw_name: "/* __esModule */true".into(),
              ids: export_name[1..].to_vec(),
              export_name,
              info_id: info.id(),
              comment: None,
            });
          }
          _ => {}
        },
      }
    }

    if export_name.is_empty() {
      match info {
        ModuleInfo::Concatenated(info) => {
          needed_namespace_objects.insert(info.module);
          return Binding::Raw(RawBinding {
            raw_name: info
              .namespace_object_name
              .clone()
              .expect("should have namespace_object_name"),
            ids: export_name.clone(),
            export_name,
            info_id: info.module,
            comment: None,
          });
        }
        ModuleInfo::External(info) => {
          return Binding::Raw(RawBinding {
            raw_name: info.name.clone().expect("should have raw name"),
            ids: export_name.clone(),
            export_name,
            info_id: info.module,
            comment: None,
          });
        }
      }
    }

    let exports_info = mg.get_exports_info(&info.id());
    // webpack use `get_exports_info` here, https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/optimize/ConcatenatedModule.js#L377-L377
    // But in our arch, there is no way to modify module graph during code_generation phase, so we use `get_export_info_without_mut_module_graph` instead.`
    let export_info = exports_info.get_export_info_without_mut_module_graph(mg, &export_name[0]);
    let export_info_hash_key = export_info.as_hash_key();

    if already_visited.contains(&export_info_hash_key) {
      return Binding::Raw(RawBinding {
        raw_name: "/* circular reexport */ Object(function x() { x() }())".into(),
        ids: Vec::new(),
        export_name,
        info_id: info.id(),
        comment: None,
      });
    }

    already_visited.insert(export_info_hash_key);

    match info {
      ModuleInfo::Concatenated(info) => {
        let export_id = export_name.first().cloned();
        if matches!(
          export_info.provided(mg),
          Some(crate::ExportInfoProvided::False)
        ) {
          needed_namespace_objects.insert(info.module);
          return Binding::Raw(RawBinding {
            raw_name: info
              .namespace_object_name
              .clone()
              .expect("should have namespace_object_name"),
            ids: export_name.clone(),
            export_name,
            info_id: info.module,
            comment: None,
          });
        }

        if let Some(ref export_id) = export_id
          && let Some(direct_export) = info.export_map.as_ref().and_then(|map| map.get(export_id))
        {
          if let Some(used_name) =
            exports_info.get_used_name(mg, runtime, UsedName::Vec(export_name.clone()))
          {
            // https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/optimize/ConcatenatedModule.js#L402-L404
            let used_name = used_name.to_used_name_vec();

            return Binding::Symbol(SymbolBinding {
              info_id: info.module,
              name: direct_export.as_str().into(),
              ids: used_name[1..].to_vec(),
              export_name,
              comment: None,
            });
          } else {
            return Binding::Raw(RawBinding {
              raw_name: "/* unused export */ undefined".into(),
              ids: export_name[1..].to_vec(),
              export_name,
              info_id: info.module,
              comment: None,
            });
          }
        }

        if let Some(ref export_id) = export_id
          && let Some(raw_export) = info
            .raw_export_map
            .as_ref()
            .and_then(|map| map.get(export_id))
        {
          return Binding::Raw(RawBinding {
            info_id: info.module,
            raw_name: raw_export.as_str().into(),
            ids: export_name.clone(),
            export_name,
            comment: None,
          });
        }

        let reexport = export_info.find_target(
          mg,
          Arc::new(|module: &ModuleIdentifier| module_to_info_map.contains_key(module)),
        );
        match reexport {
          crate::FindTargetRetEnum::Undefined => {}
          crate::FindTargetRetEnum::False => {
            panic!(
              "Target module of reexport is not part of the concatenation (export '{:?}')",
              &export_id
            );
          }
          crate::FindTargetRetEnum::Value(reexport) => {
            if let Some(ref_info) = module_to_info_map.get(&reexport.module) {
              // https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/optimize/ConcatenatedModule.js#L457
              let build_meta = mg
                .module_by_identifier(&ref_info.id())
                .expect("should have module")
                .build_meta();
              return Self::get_final_binding(
                mg,
                &ref_info.id(),
                if let Some(reexport_export) = reexport.export {
                  [reexport_export.clone(), export_name[1..].to_vec()].concat()
                } else {
                  export_name[1..].to_vec()
                },
                module_to_info_map,
                runtime,
                needed_namespace_objects,
                as_call,
                build_meta.strict_esm_module,
                asi_safe,
                already_visited,
              );
            }
          }
        }

        if info.namespace_export_symbol.is_some() {
          // That's how webpack write https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/optimize/ConcatenatedModule.js#L463-L471
          let used_name = exports_info
            .get_used_name(mg, runtime, UsedName::Vec(export_name.clone()))
            .expect("should have export name");
          let used_name = used_name.to_used_name_vec();
          return Binding::Raw(RawBinding {
            info_id: info.module,
            raw_name: info
              .namespace_object_name
              .as_ref()
              .expect("should have raw name")
              .as_str()
              .into(),
            ids: used_name,
            export_name,
            comment: None,
          });
        }

        panic!(
          "Cannot get final name for export '{}'",
          join_atom(export_name.iter(), ".")
        );
      }
      ModuleInfo::External(info) => {
        if let Some(used_name) =
          exports_info.get_used_name(mg, runtime, UsedName::Vec(export_name.clone()))
        {
          let used_name = used_name.to_used_name_vec();
          let comment = if used_name == export_name {
            String::new()
          } else {
            Template::to_normal_comment(&join_atom(export_name.iter(), ","))
          };
          Binding::Raw(RawBinding {
            raw_name: format!(
              "{}{}",
              info.name.as_ref().expect("should have name"),
              comment
            )
            .into(),
            ids: used_name,
            export_name,
            info_id: info.module,
            comment: None,
          })
        } else {
          Binding::Raw(RawBinding {
            raw_name: "/* unused export */ undefined".into(),
            ids: export_name[1..].to_vec(),
            export_name,
            info_id: info.module,
            comment: None,
          })
        }
      }
    }
  }
}

pub fn is_esm_dep_like(dep: &BoxDependency) -> bool {
  matches!(
    dep.dependency_type(),
    DependencyType::EsmImportSpecifier
      | DependencyType::EsmExportImportedSpecifier
      | DependencyType::EsmImport
      | DependencyType::EsmExport
  )
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

pub fn find_new_name(
  old_name: &str,
  used_names1: &HashSet<Atom>,
  used_names2: Option<&HashSet<Atom>>,
  extra_info: &str,
) -> Atom {
  let mut name = Cow::Borrowed(old_name);

  if name == DEFAULT_EXPORT {
    name = Cow::Borrowed("");
  }
  if name == NAMESPACE_OBJECT_EXPORT {
    name = Cow::Borrowed("namespaceObject");
  }

  // Remove uncool stuff
  let extra_info = REGEX.replace_all(extra_info, "");

  let mut splitted_info: Vec<&str> = extra_info.split('/').collect();
  while let Some(info_part) = splitted_info.pop() {
    name = Cow::Owned(format!(
      "{}{}",
      info_part,
      if name.is_empty() {
        String::new()
      } else {
        format!("_{name}")
      }
    ));
    let name_ident = to_identifier(&name).into();
    if !used_names1.contains(&name_ident)
      && (used_names2.is_none()
        || !used_names2
          .expect("should not be none")
          .contains(&name_ident))
    {
      return name_ident;
    }
  }

  let mut i = 0;
  let mut name_with_number = to_identifier(&format!("{}_{}", name, itoa!(i))).into();
  while used_names1.contains(&name_with_number)
    || used_names2
      .map(|map| map.contains(&name_with_number))
      .unwrap_or_default()
  {
    i += 1;
    name_with_number = to_identifier(&format!("{}_{}", name, itoa!(i))).into();
  }

  name_with_number
}
