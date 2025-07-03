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
use swc_core::ecma::atoms::Atom;

/// Creates `_webpack_require__.d(__webpack_exports__, {})` for each export specifier.
///
/// Handles both regular export specifiers and ConsumeShared module fallback exports
/// with sophisticated tree-shaking macro integration.
#[cacheable]
#[derive(Debug, Clone)]
pub struct ESMExportSpecifierDependency {
  id: DependencyId,
  pub name: Atom,
  pub value: Atom,
  range: DependencyRange,
  pub value_range: Option<(u32, u32)>,
  pub enum_value: Option<TSEnumValue>,
  #[cacheable(with=Skip)]
  source_map: Option<SharedSourceMap>,
}

impl ESMExportSpecifierDependency {
  pub fn new(
    name: Atom,
    value: Atom,
    range: DependencyRange,
    value_range: Option<(u32, u32)>,
    enum_value: Option<TSEnumValue>,
    source_map: Option<SharedSourceMap>,
  ) -> Self {
    Self {
      id: DependencyId::new(),
      name,
      value,
      range,
      value_range,
      enum_value,
      source_map,
    }
  }

  /// Get ConsumeShared information if this module is a shared module
  fn get_consume_shared_info(&self, module_graph: &ModuleGraph) -> Option<String> {
    let parent_id = module_graph.get_parent_module(&self.id)?;
    let parent_module = module_graph.module_by_identifier(parent_id)?;

    // Extract share key from module identifier if it's a ConsumeShared module
    let identifier_str = parent_module.identifier().to_string();
    if identifier_str.contains("consume-shared-module|") {
      // Extract share key from identifier pattern:
      // consume-shared-module|<share-scope>|<share-key>|<request>
      let parts: Vec<&str> = identifier_str.split('|').collect();
      if parts.len() >= 3 {
        return Some(parts[2].to_string());
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

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::EsmExport
  }

  fn loc(&self) -> Option<DependencyLocation> {
    self.range.to_loc(self.source_map.as_ref())
  }

  fn range(&self) -> Option<&DependencyRange> {
    Some(&self.range)
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }

  fn get_exports(&self, _mg: &ModuleGraph) -> Option<ExportsSpec> {
    Some(ExportsSpec {
      exports: ExportsOfExportsSpec::Array(vec![ExportNameOrSpec::Name(self.name.clone())]),
      ..Default::default()
    })
  }

  fn get_referenced_exports(
    &self,
    _module_graph: &ModuleGraph,
    _module_graph_cache: &ModuleGraphCacheArtifact,
    _runtime: Option<&rspack_core::RuntimeSpec>,
  ) -> Vec<rspack_core::ExtendedReferencedExport> {
    vec![]
  }
}

impl AsModuleDependency for ESMExportSpecifierDependency {}

impl AsContextDependency for ESMExportSpecifierDependency {}

#[cacheable_dyn]
impl DependencyCodeGeneration for ESMExportSpecifierDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(DependencyTemplateType::Dependency(
      DependencyType::EsmExport,
    ))
  }
}

impl DependencyTemplate for ESMExportSpecifierDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let TemplateContext {
      runtime,
      concatenation_scope,
      ..
    } = code_generatable_context;

    let module_graph = code_generatable_context.compilation.get_module_graph();

    if let Some(scope) = concatenation_scope {
      scope.register_export(&self.name, &self.value);
      return;
    }

    let content = if let Some(from) = code_generatable_context
      .runtime_requirements
      .get(&RuntimeGlobals::CURRENT_REMOTE_GET_SCOPE)
      && from.len() > 0
    {
      format!(
        "()=>{{ return {}; }}",
        code_generatable_context
          .module
          .get_exports_argument()
          .expect("should have exports argument")
      )
    } else {
      format!("/* ESM export specifier */ {}", self.value)
    };

    // Replace ESMExportSpecifierDependency range with empty string
    source.replace(self.range.start, self.range.end, "", None);

    // Replace value range if exists, else don't do anything.
    if let Some((start, end)) = self.value_range {
      source.replace(start, end, &content, None);
    }
  }

  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let TemplateContext {
      runtime,
      module,
      compilation,
      init_fragments,
      runtime_requirements,
      concatenation_scope,
      ..
    } = code_generatable_context;

    if concatenation_scope.is_some() {
      return;
    }

    let dep = dep
      .as_any()
      .downcast_ref::<ESMExportSpecifierDependency>()
      .expect("should be ESMExportSpecifierDependency");

    let module_graph = compilation.get_module_graph();
    let module_identifier = module.identifier();
    let module = module_graph
      .module_by_identifier(&module_identifier)
      .expect("should have module graph module");

    // Determine ConsumeShared integration
    let consume_shared_info = dep.get_consume_shared_info(&module_graph);

    // Handle enum value exports
    if let Some(enum_value) = &dep.enum_value {
      let all_enum_member_inlined = enum_value.iter().all(|(enum_key, enum_member)| {
        // if there are enum member need to keep origin/non-inlineable, then we need to keep the enum decl
        if enum_member.is_none() {
          return false;
        }
        let export_name = &[dep.name.clone(), enum_key.clone()];
        let exports_info = module_graph.get_prefetched_exports_info(
          &module.identifier(),
          PrefetchExportsInfoMode::Nested(export_name),
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

    let exports_info = module_graph
      .get_prefetched_exports_info(&module.identifier(), PrefetchExportsInfoMode::Default);
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
            "/* @common:if [condition=\"treeShake.{}.{}\"] */ {} /* @common:endif */",
            share_key, dep.name, dep.value
          )
          .into()
        } else {
          dep.value.clone()
        };

        // Handle enum values
        if let Some(enum_value) = &dep.enum_value {
          let mut exports = vec![];
          for (enum_key, enum_member) in enum_value.iter() {
            // Enum member is inlineable
            if let Some(enum_member) = enum_member {
              let export_name = &[dep.name.clone(), enum_key.clone()];
              let exports_info = module_graph.get_prefetched_exports_info(
                &module.identifier(),
                PrefetchExportsInfoMode::Nested(export_name),
              );
              let enum_member_used_name = ExportsInfoGetter::get_used_name(
                GetUsedNameParam::WithNames(&exports_info),
                *runtime,
                export_name,
              );
              if let Some(UsedName::Normal(ref used_vec)) = enum_member_used_name
                && !used_vec.is_empty()
              {
                let enum_member_used_atom = used_vec.last().expect("should have last");

                // Generate enum member export with ConsumeShared macro
                let enum_export_content = if let Some(ref share_key) = consume_shared_info {
                  format!(
                    "/* @common:if [condition=\"treeShake.{}.{}.{}\"] */ {} /* @common:endif */",
                    share_key,
                    dep.name,
                    enum_key,
                    enum_member.to_string()
                  )
                  .into()
                } else {
                  enum_member.to_string().into()
                };

                exports.push((enum_member_used_atom.clone(), enum_export_content));
              }
            }
          }

          if !exports.is_empty() {
            init_fragments.push(Box::new(ESMExportInitFragment::new(
              module.get_exports_argument(),
              exports,
            )));
          }
        } else {
          // Regular export
          let export_fragment = ESMExportInitFragment::new(
            module.get_exports_argument(),
            vec![(used_name_atom, export_content)],
          );
          init_fragments.push(Box::new(export_fragment));
        }

        // Add debug comment fragment if in development mode
        if compilation.options.mode.is_development() {
          let debug_fragment = NormalInitFragment::new(
            format!(
              "/* DEBUG: ESM export '{}' -> '{}' */\n",
              dep.name, dep.value
            ),
            InitFragmentStage::StageConstants,
            -1000, // High priority for debug info
            InitFragmentKey::unique(),
            None,
          );
          init_fragments.push(debug_fragment.boxed());
        }
      }
      Some(UsedName::Inlined(ref value)) => {
        // Handle inlined exports
        if let Some(enum_value) = &dep.enum_value {
          let inlined_value = value.to_string();
          runtime_requirements.insert(RuntimeGlobals::EXPORTS);
          runtime_requirements.insert(RuntimeGlobals::DEFINE_PROPERTY_GETTERS);

          let mut enum_exports = vec![(dep.name.clone(), inlined_value.clone())];
          for (enum_key, enum_member) in enum_value.iter() {
            if let Some(enum_member) = enum_member {
              let export_name = &[dep.name.clone(), enum_key.clone()];
              let exports_info = module_graph.get_prefetched_exports_info(
                &module.identifier(),
                PrefetchExportsInfoMode::Nested(export_name),
              );
              let enum_member_used_name = ExportsInfoGetter::get_used_name(
                GetUsedNameParam::WithNames(&exports_info),
                *runtime,
                export_name,
              );
              if let Some(UsedName::Normal(ref used_vec)) = enum_member_used_name
                && !used_vec.is_empty()
              {
                let enum_member_used_atom = used_vec.last().expect("should have last");
                enum_exports.push((enum_member_used_atom.clone(), enum_member.to_string()));
              }
            }
          }

          init_fragments.push(Box::new(ESMExportInitFragment::new(
            module.get_exports_argument(),
            enum_exports
              .into_iter()
              .map(|(k, v)| (k, v.into()))
              .collect(),
          )));
        }
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

impl ESMExportSpecifierDependency {
  pub fn get_export_names(&self) -> Vec<Atom> {
    vec![self.name.clone()]
  }
}
