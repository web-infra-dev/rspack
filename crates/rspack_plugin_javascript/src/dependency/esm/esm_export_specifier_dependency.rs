use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsPreset, Skip},
};
use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_core::{
  AsContextDependency, AsModuleDependency, Dependency, DependencyCategory,
  DependencyCodeGeneration, DependencyId, DependencyLocation, DependencyRange, DependencyTemplate,
  DependencyTemplateType, DependencyType, ESMExportInitFragment, EvaluatedInlinableValue,
  ExportNameOrSpec, ExportSpec, ExportsInfoGetter, ExportsOfExportsSpec, ExportsSpec,
  GetUsedNameParam, InitFragmentExt, InitFragmentKey, InitFragmentStage, ModuleGraph,
  ModuleGraphCacheArtifact, NormalInitFragment, PrefetchExportsInfoMode, RuntimeGlobals,
  SharedSourceMap, TemplateContext, TemplateReplaceSource, UsedName,
};
use rustc_hash::FxHashSet;
use swc_core::ecma::atoms::Atom;

/// Creates `_webpack_require__.d(__webpack_exports__, {})` for each export specifier.
///
/// Handles both regular export specifiers and ConsumeShared module fallback exports
/// with sophisticated tree-shaking macro integration.
#[cacheable]
#[derive(Debug, Clone)]
pub struct ESMExportSpecifierDependency {
  id: DependencyId,
  range: DependencyRange,
  #[cacheable(with=Skip)]
  source_map: Option<SharedSourceMap>,
  #[cacheable(with=AsPreset)]
  name: Atom,
  #[cacheable(with=AsPreset)]
  value: Atom, // export identifier
  inline: Option<EvaluatedInlinableValue>,
}

impl ESMExportSpecifierDependency {
  /// Creates a new ESM export specifier dependency with comprehensive validation.
  ///
  /// # Arguments
  /// * `name` - Export name (what's exported)
  /// * `value` - Export value identifier (what's being exported)
  /// * `inline` - Optional inline evaluation information for optimization
  /// * `range` - Source code range for error reporting
  /// * `source_map` - Source map for accurate error locations
  pub fn new(
    name: Atom,
    value: Atom,
    inline: Option<EvaluatedInlinableValue>,
    range: DependencyRange,
    source_map: Option<SharedSourceMap>,
  ) -> Self {
    Self {
      name,
      value,
      inline,
      range,
      source_map,
      id: DependencyId::new(),
    }
  }

  /// Determines if this export is related to ConsumeShared module fallback.
  /// Enhanced to traverse the full module graph for reexport scenarios.
  fn get_consume_shared_info(&self, module_graph: &ModuleGraph) -> Option<String> {
    // Check if parent module is ConsumeShared
    if let Some(parent_module_id) = module_graph.get_parent_module(&self.id) {
      if let Some(parent_module) = module_graph.module_by_identifier(parent_module_id) {
        if parent_module.module_type() == &rspack_core::ModuleType::ConsumeShared {
          return parent_module.get_consume_shared_key();
        }
      }
    }

    // Get current module identifier for deeper analysis
    let module_identifier = module_graph
      .get_parent_module(&self.id)
      .and_then(|id| module_graph.module_by_identifier(id))
      .map(|m| m.identifier())?;

    // Check immediate incoming connections for ConsumeShared modules
    for connection in module_graph.get_incoming_connections(&module_identifier) {
      if let Some(origin_module) = connection.original_module_identifier.as_ref() {
        if let Some(origin_module_obj) = module_graph.module_by_identifier(origin_module) {
          if origin_module_obj.module_type() == &rspack_core::ModuleType::ConsumeShared {
            return origin_module_obj.get_consume_shared_key();
          }
        }
      }
    }

    // Enhanced: Check deeper in the module graph for reexport scenarios
    // Traverse incoming connections recursively to find ConsumeShared modules
    // that might be multiple levels up in the module dependency chain
    let mut visited = std::collections::HashSet::new();
    if let Some(share_key) =
      self.find_consume_shared_recursive(&module_identifier, module_graph, &mut visited, 5)
    {
      return Some(share_key);
    }

    None
  }

  /// Recursively search for ConsumeShared modules in the module graph
  fn find_consume_shared_recursive(
    &self,
    current_module: &rspack_core::ModuleIdentifier,
    module_graph: &ModuleGraph,
    visited: &mut std::collections::HashSet<rspack_core::ModuleIdentifier>,
    max_depth: usize,
  ) -> Option<String> {
    if max_depth == 0 || visited.contains(current_module) {
      return None;
    }
    visited.insert(current_module.clone());

    // Check all incoming connections for this module
    for connection in module_graph.get_incoming_connections(current_module) {
      if let Some(origin_module_id) = connection.original_module_identifier.as_ref() {
        if let Some(origin_module) = module_graph.module_by_identifier(origin_module_id) {
          // Found a ConsumeShared module - return its share key
          if origin_module.module_type() == &rspack_core::ModuleType::ConsumeShared {
            return origin_module.get_consume_shared_key();
          }

          // Recursively check this module's incoming connections
          if let Some(share_key) = self.find_consume_shared_recursive(
            origin_module_id,
            module_graph,
            visited,
            max_depth - 1,
          ) {
            return Some(share_key);
          }
        }
      }
    }

    None
  }
}

#[cacheable_dyn]
impl Dependency for ESMExportSpecifierDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn loc(&self) -> Option<DependencyLocation> {
    self.range.to_loc(self.source_map.as_ref())
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::EsmExportSpecifier
  }

  fn get_exports(
    &self,
    _mg: &ModuleGraph,
    _mg_cache: &ModuleGraphCacheArtifact,
  ) -> Option<ExportsSpec> {
    Some(ExportsSpec {
      exports: ExportsOfExportsSpec::Names(vec![ExportNameOrSpec::ExportSpec(ExportSpec {
        name: self.name.clone(),
        inlinable: self.inline,
        ..Default::default()
      })]),
      priority: Some(1),
      can_mangle: None,
      terminal_binding: Some(true),
      from: None,
      dependencies: None,
      hide_export: None,
      exclude_exports: None,
    })
  }

  fn get_module_evaluation_side_effects_state(
    &self,
    _module_graph: &rspack_core::ModuleGraph,
    _module_chain: &mut IdentifierSet,
    _connection_state_cache: &mut IdentifierMap<rspack_core::ConnectionState>,
  ) -> rspack_core::ConnectionState {
    rspack_core::ConnectionState::Active(false)
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::False
  }
}

impl AsModuleDependency for ESMExportSpecifierDependency {}

#[cacheable_dyn]
impl DependencyCodeGeneration for ESMExportSpecifierDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(ESMExportSpecifierDependencyTemplate::template_type())
  }
}

impl AsContextDependency for ESMExportSpecifierDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct ESMExportSpecifierDependencyTemplate;

impl ESMExportSpecifierDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::EsmExportSpecifier)
  }
}

impl DependencyTemplate for ESMExportSpecifierDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    _source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<ESMExportSpecifierDependency>()
      .expect(
        "ESMExportSpecifierDependencyTemplate should only be used for ESMExportSpecifierDependency",
      );

    let TemplateContext {
      init_fragments,
      compilation,
      module,
      runtime,
      runtime_requirements,
      concatenation_scope,
      ..
    } = code_generatable_context;

    // Handle concatenation scope for module concatenation optimization
    if let Some(scope) = concatenation_scope {
      scope.register_export(dep.name.clone(), dep.value.to_string());
      return;
    }

    let module_graph = compilation.get_module_graph();
    let module_identifier = module.identifier();
    let module = module_graph
      .module_by_identifier(&module_identifier)
      .expect("should have module graph module");

    // Determine ConsumeShared integration
    let consume_shared_info = dep.get_consume_shared_info(&module_graph);

    // Get export usage information with proper prefetching
    let exports_info = module_graph.get_prefetched_exports_info(
      &module.identifier(),
      PrefetchExportsInfoMode::NamedExports(FxHashSet::from_iter([&dep.name])),
    );

    let used_name = ExportsInfoGetter::get_used_name(
      GetUsedNameParam::WithNames(&exports_info),
      *runtime,
      std::slice::from_ref(&dep.name),
    );

    match used_name {
      Some(UsedName::Normal(ref used_vec)) if !used_vec.is_empty() => {
        let used_name_atom = used_vec[0].clone();

        // Add runtime requirements
        runtime_requirements.insert(RuntimeGlobals::EXPORTS);
        runtime_requirements.insert(RuntimeGlobals::DEFINE_PROPERTY_GETTERS);

        // Generate export content with ConsumeShared macro integration
        let export_content = if let Some(ref share_key) = consume_shared_info {
          format!(
            "/* @common:if [condition=\"treeShake.{}.{}\"] */ /* ESM export specifier */ {} /* @common:endif */",
            share_key, dep.name, dep.value
          )
        } else {
          format!("/* ESM export specifier */ {}", dep.value)
        };

        // Create export init fragment
        let export_fragment = ESMExportInitFragment::new(
          module.get_exports_argument(),
          vec![(used_name_atom, export_content.into())],
        );

        init_fragments.push(Box::new(export_fragment));
      }
      Some(UsedName::Inlined(_)) => {
        // Export is inlined, add comment for clarity
        let comment_fragment = NormalInitFragment::new(
          format!("/* inlined ESM export '{}' */\n", dep.name),
          InitFragmentStage::StageConstants,
          0,
          InitFragmentKey::unique(),
          None,
        );
        init_fragments.push(comment_fragment.boxed());
      }
      None => {
        // Export is unused, add debug comment in development
        if compilation.options.mode.is_development() {
          let unused_fragment = NormalInitFragment::new(
            format!("/* unused ESM export '{}' */\n", dep.name),
            InitFragmentStage::StageConstants,
            0,
            InitFragmentKey::unique(),
            None,
          );
          init_fragments.push(unused_fragment.boxed());
        }
      }
      _ => {
        // Unexpected case, add warning fragment
        let warning_fragment = NormalInitFragment::new(
          format!(
            "/* WARNING: unexpected export state for '{}' */\n",
            dep.name
          ),
          InitFragmentStage::StageConstants,
          0,
          InitFragmentKey::unique(),
          None,
        );
        init_fragments.push(warning_fragment.boxed());
      }
    }
  }
}
