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
  SharedSourceMap, TSEnumValue, TemplateContext, TemplateReplaceSource, UsedName,
};
use rustc_hash::FxHashSet;
use swc_core::ecma::atoms::Atom;

// Create _webpack_require__.d(__webpack_exports__, {}) for each export.
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
  enum_value: Option<TSEnumValue>,
}

impl ESMExportSpecifierDependency {
  pub fn new(
    name: Atom,
    value: Atom,
    inline: Option<EvaluatedInlinableValue>,
    enum_value: Option<TSEnumValue>,
    range: DependencyRange,
    source_map: Option<SharedSourceMap>,
  ) -> Self {
    Self {
      name,
      value,
      inline,
      enum_value,
      range,
      source_map,
      id: DependencyId::new(),
    }
  }

  // REMOVED: get_consume_shared_info() and find_consume_shared_recursive() - No longer needed, using BuildMeta directly
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
        inlinable: self.inline.clone(),
        exports: self.enum_value.as_ref().map(|enum_value| {
          enum_value
            .iter()
            .map(|(enum_name, enum_value)| {
              ExportNameOrSpec::ExportSpec(ExportSpec {
                name: enum_name.clone(),
                inlinable: enum_value.clone(),
                ..Default::default()
              })
            })
            .collect()
        }),
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
    _module_graph_cache: &ModuleGraphCacheArtifact,
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

    // SIMPLIFIED: Use pre-computed ConsumeShared context from BuildMeta
    let consume_shared_info = module.build_meta().consume_shared_key.as_ref();

    // remove the enum decl if all the enum members are inlined
    if let Some(enum_value) = &dep.enum_value {
      let all_enum_member_inlined = enum_value.iter().all(|(enum_key, enum_member)| {
        // if there are enum member need to keep origin/non-inlineable, then we need to keep the enum decl
        if enum_member.is_none() {
          return false;
        }
        let export_name = &[dep.name.clone(), enum_key.clone()];
        let exports_info = module_graph.get_prefetched_exports_info(
          &module.identifier(),
          PrefetchExportsInfoMode::NamedNestedExports(export_name),
        );
        let enum_member_used_name = ExportsInfoGetter::get_used_name(
          GetUsedNameParam::WithNames(&exports_info),
          *runtime,
          export_name,
        );
        matches!(enum_member_used_name, Some(UsedName::Inlined(_)))
      });
      if all_enum_member_inlined {
        return;
      }
    }

    // Get export usage information with proper prefetching
    let exports_info = module_graph.get_prefetched_exports_info(
      &module.identifier(),
      PrefetchExportsInfoMode::NamedExports(FxHashSet::from_iter([&dep.name])),
    );
    let Some(used_name) = ExportsInfoGetter::get_used_name(
      GetUsedNameParam::WithNames(&exports_info),
      *runtime,
      std::slice::from_ref(&dep.name),
    ) else {
      return;
    };

    match used_name {
      UsedName::Normal(ref used_vec) if !used_vec.is_empty() => {
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
      UsedName::Inlined(_) => {
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
      _ => {
        // For normal case with simple value assignment
        if let UsedName::Normal(vec) = used_name {
          let used_name_atom = vec[0].clone();
          init_fragments.push(Box::new(ESMExportInitFragment::new(
            module.get_exports_argument(),
            vec![(used_name_atom, dep.value.clone())],
          )));
        }
      }
    }
  }
}
