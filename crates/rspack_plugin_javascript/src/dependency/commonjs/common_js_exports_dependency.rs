use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsPreset, AsVec},
};
use rspack_core::{
  property_access, AsContextDependency, AsModuleDependency, Dependency, DependencyCategory,
  DependencyCodeGeneration, DependencyId, DependencyRange, DependencyTemplate,
  DependencyTemplateType, DependencyType, ExportNameOrSpec, ExportSpec, ExportsInfoGetter,
  ExportsOfExportsSpec, ExportsSpec, GetUsedNameParam, InitFragmentExt, InitFragmentKey,
  InitFragmentStage, ModuleGraph, ModuleGraphCacheArtifact, NormalInitFragment,
  PrefetchExportsInfoMode, RuntimeGlobals, TemplateContext, TemplateReplaceSource, UsedName,
};
use swc_core::atoms::Atom;

#[cacheable]
#[derive(Debug, Clone, Copy)]
pub enum ExportsBase {
  Exports,
  ModuleExports,
  This,
  DefinePropertyExports,
  DefinePropertyModuleExports,
  DefinePropertyThis,
}

impl ExportsBase {
  pub const fn is_exports(&self) -> bool {
    matches!(self, Self::Exports | Self::DefinePropertyExports)
  }

  pub const fn is_module_exports(&self) -> bool {
    matches!(
      self,
      Self::ModuleExports | Self::DefinePropertyModuleExports
    )
  }

  pub const fn is_this(&self) -> bool {
    matches!(self, Self::This | Self::DefinePropertyThis)
  }

  pub const fn is_expression(&self) -> bool {
    matches!(self, Self::Exports | Self::ModuleExports | Self::This)
  }

  pub const fn is_define_property(&self) -> bool {
    matches!(
      self,
      Self::DefinePropertyExports | Self::DefinePropertyModuleExports | Self::DefinePropertyThis
    )
  }
}

#[cacheable]
#[derive(Debug, Clone)]
pub struct CommonJsExportsDependency {
  id: DependencyId,
  range: DependencyRange,
  value_range: Option<DependencyRange>,
  base: ExportsBase,
  #[cacheable(with=AsVec<AsPreset>)]
  names: Vec<Atom>,
}

impl CommonJsExportsDependency {
  pub fn new(
    range: DependencyRange,
    value_range: Option<DependencyRange>,
    base: ExportsBase,
    names: Vec<Atom>,
  ) -> Self {
    Self {
      id: DependencyId::new(),
      range,
      value_range,
      base,
      names,
    }
  }
}

#[cacheable_dyn]
impl Dependency for CommonJsExportsDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn range(&self) -> Option<&DependencyRange> {
    Some(&self.range)
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CommonJS
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CjsExports
  }

  fn get_exports(
    &self,
    _mg: &ModuleGraph,
    _mg_cache: &ModuleGraphCacheArtifact,
  ) -> Option<ExportsSpec> {
    let vec = vec![ExportNameOrSpec::ExportSpec(ExportSpec {
      name: self.names[0].clone(),
      can_mangle: Some(false), // in webpack, object own property may not be mangled
      ..Default::default()
    })];
    Some(ExportsSpec {
      exports: ExportsOfExportsSpec::Names(vec),
      ..Default::default()
    })
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::False
  }
}

impl AsModuleDependency for CommonJsExportsDependency {}

#[cacheable_dyn]
impl DependencyCodeGeneration for CommonJsExportsDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(CommonJsExportsDependencyTemplate::template_type())
  }
}

impl AsContextDependency for CommonJsExportsDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct CommonJsExportsDependencyTemplate;

impl CommonJsExportsDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::CjsExports)
  }
}

impl DependencyTemplate for CommonJsExportsDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<CommonJsExportsDependency>()
      .expect(
        "CommonJsExportsDependencyTemplate should only be used for CommonJsExportsDependency",
      );

    let TemplateContext {
      compilation,
      module,
      runtime,
      init_fragments,
      runtime_requirements,
      ..
    } = code_generatable_context;

    let module_graph = compilation.get_module_graph();
    let module_identifier = module.identifier();
    let module = module_graph
      .module_by_identifier(&module_identifier)
      .expect("should have mgm");

    // ConsumeShared tree-shaking macro support
    let consume_shared_info: Option<String> = module.get_consume_shared_key();

    let used = if dep.names.is_empty() {
      let exports_info = ExportsInfoGetter::prefetch_used_info_without_name(
        &module_graph.get_exports_info(&module.identifier()),
        &module_graph,
        *runtime,
        false,
      );
      ExportsInfoGetter::get_used_name(
        GetUsedNameParam::WithoutNames(&exports_info),
        *runtime,
        &dep.names,
      )
    } else {
      let exports_info = module_graph.get_prefetched_exports_info(
        &module.identifier(),
        PrefetchExportsInfoMode::Nested(&dep.names),
      );
      ExportsInfoGetter::get_used_name(
        GetUsedNameParam::WithNames(&exports_info),
        *runtime,
        &dep.names,
      )
    };

    let exports_argument = module.get_exports_argument();
    let module_argument = module.get_module_argument();

    let base = if dep.base.is_exports() {
      runtime_requirements.insert(RuntimeGlobals::EXPORTS);
      exports_argument.to_string()
    } else if dep.base.is_module_exports() {
      runtime_requirements.insert(RuntimeGlobals::MODULE);
      format!("{module_argument}.exports")
    } else if dep.base.is_this() {
      runtime_requirements.insert(RuntimeGlobals::THIS_AS_EXPORTS);
      "this".to_string()
    } else {
      panic!("Unexpected base type");
    };

    if dep.base.is_expression() {
      if let Some(UsedName::Normal(used)) = used {
        let export_assignment = format!("{}{}", base, property_access(&used, 0));
        let export_name = used
          .iter()
          .map(|a| a.as_str())
          .collect::<Vec<_>>()
          .join(".");

        // ConsumeShared tree-shaking macro support
        let export_content = if let Some(shared_key) = &consume_shared_info {
          format!(
            "/* @common:if [condition=\"treeShake.{}.{}\"] */ {} /* @common:endif */",
            shared_key, export_name, export_assignment
          )
        } else {
          export_assignment
        };

        source.replace(dep.range.start, dep.range.end, &export_content, None);
      } else {
        // Export a inlinable const from cjs is not possible for now but we compat it here
        let is_inlined = matches!(used, Some(UsedName::Inlined(_)));
        let placeholder_var = format!(
          "__webpack_{}_export__",
          if is_inlined { "inlined" } else { "unused" }
        );
        source.replace(dep.range.start, dep.range.end, &placeholder_var, None);
        init_fragments.push(
          NormalInitFragment::new(
            format!("var {placeholder_var};\n"),
            InitFragmentStage::StageConstants,
            0,
            InitFragmentKey::CommonJsExports(placeholder_var),
            None,
          )
          .boxed(),
        );
      }
    } else if dep.base.is_define_property() {
      if let Some(value_range) = &dep.value_range {
        if let Some(UsedName::Normal(used)) = used {
          if !used.is_empty() {
            let export_name = used.last().unwrap();

            // ConsumeShared tree-shaking macro support
            let (define_property_start, define_property_end) = if let Some(shared_key) =
              &consume_shared_info
            {
              (
                format!(
                  "/* @common:if [condition=\"treeShake.{}.{}\"] */ Object.defineProperty({}{}, {}, (",
                  shared_key,
                  export_name,
                  base,
                  property_access(used[0..used.len() - 1].iter(), 0),
                  serde_json::to_string(&used.last()).expect("Unexpected render define property base")
                ),
                ")) /* @common:endif */)",
              )
            } else {
              (
                format!(
                  "Object.defineProperty({}{}, {}, (",
                  base,
                  property_access(used[0..used.len() - 1].iter(), 0),
                  serde_json::to_string(&used.last())
                    .expect("Unexpected render define property base")
                ),
                "))",
              )
            };
            source.replace(
              dep.range.start,
              value_range.start,
              &define_property_start,
              None,
            );
            source.replace(value_range.end, dep.range.end, &define_property_end, None);
          } else {
            panic!("Unexpected base type");
          }
        } else {
          init_fragments.push(
            NormalInitFragment::new(
              "var __webpack_unused_export__;\n".to_string(),
              InitFragmentStage::StageConstants,
              0,
              InitFragmentKey::CommonJsExports("__webpack_unused_export__".to_owned()),
              None,
            )
            .boxed(),
          );
          source.replace(
            dep.range.start,
            value_range.start,
            "__webpack_unused_export__ = (",
            None,
          );
          source.replace(value_range.end, dep.range.end, ")", None);
        }
      } else {
        panic!("Define property need value range");
      }
    } else {
      panic!("Unexpected base type");
    }
  }
}
