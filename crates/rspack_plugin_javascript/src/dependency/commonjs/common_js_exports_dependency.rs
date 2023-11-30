use std::task::Wake;

use rspack_core::{
  property_access, AsContextDependency, AsModuleDependency, Dependency, DependencyCategory,
  DependencyId, DependencyTemplate, DependencyType, ExportNameOrSpec, ExportsOfExportsSpec,
  ExportsSpec, ExtendedReferencedExport, InitFragmentExt, InitFragmentKey, InitFragmentStage,
  ModuleDependency, ModuleGraph, NormalInitFragment, RuntimeGlobals, TemplateContext,
  TemplateReplaceSource, UsedName,
};
use swc_core::atoms::Atom;

#[derive(Debug, Clone)]
pub enum ExportsBase {
  Exports,
  ModuleExports,
  This,
  DefinePropertyExports,
  DefinePropertyModuleExports,
  DefinePropertyThis,
}

impl ExportsBase {
  pub fn is_exports(&self) -> bool {
    matches!(self, Self::Exports | Self::DefinePropertyExports)
  }

  pub fn is_module_exports(&self) -> bool {
    matches!(
      self,
      Self::ModuleExports | Self::DefinePropertyModuleExports
    )
  }

  pub fn is_this(&self) -> bool {
    matches!(self, Self::This | Self::DefinePropertyThis)
  }

  pub fn is_expression(&self) -> bool {
    matches!(self, Self::Exports | Self::ModuleExports | Self::This)
  }

  pub fn is_define_property(&self) -> bool {
    matches!(
      self,
      Self::DefinePropertyExports | Self::DefinePropertyModuleExports | Self::DefinePropertyThis
    )
  }
}

#[derive(Debug, Clone)]
pub struct CommonJsExportsDependency {
  id: DependencyId,
  range: (u32, u32),
  value_range: Option<(u32, u32)>,
  base: ExportsBase,
  names: Vec<Atom>,
}

impl CommonJsExportsDependency {
  pub fn new(
    range: (u32, u32),
    value_range: Option<(u32, u32)>,
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

impl Dependency for CommonJsExportsDependency {
  fn dependency_debug_name(&self) -> &'static str {
    "CommonJsExportsDependency"
  }

  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CommonJS
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CjsExports
  }

  fn get_exports(&self, _mg: &ModuleGraph) -> Option<ExportsSpec> {
    let vec = vec![ExportNameOrSpec::ExportSpec(rspack_core::ExportSpec {
      name: self.names[0].clone(),
      can_mangle: Some(false), // in webpack, object own property may not be mangled
      ..Default::default()
    })];
    Some(ExportsSpec {
      exports: ExportsOfExportsSpec::Array(vec),
      ..Default::default()
    })
  }
}

impl AsModuleDependency for CommonJsExportsDependency {}

impl DependencyTemplate for CommonJsExportsDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let TemplateContext {
      compilation,
      module,
      runtime,
      init_fragments,
      runtime_requirements,
      ..
    } = code_generatable_context;

    let mgm = compilation
      .module_graph
      .module_graph_module_by_identifier(&module.identifier())
      .expect("should have mgm");

    let used = compilation
      .module_graph
      .get_exports_info(&module.identifier())
      .id
      .get_used_name(
        &compilation.module_graph,
        *runtime,
        UsedName::Vec(self.names.clone()),
      );

    let exports_argument = mgm.get_exports_argument();
    let module_argument = mgm.get_module_argument();

    let base = if self.base.is_exports() {
      runtime_requirements.insert(RuntimeGlobals::EXPORTS);
      exports_argument.to_string()
    } else if self.base.is_module_exports() {
      runtime_requirements.insert(RuntimeGlobals::MODULE);
      format!("{}.exports", module_argument)
    } else if self.base.is_this() {
      runtime_requirements.insert(RuntimeGlobals::THIS_AS_EXPORTS);
      "this".to_string()
    } else {
      panic!("Unexpected base type");
    };

    if self.base.is_expression() {
      if let Some(used) = used {
        source.replace(
          self.range.0,
          self.range.1,
          &format!(
            "{}{}",
            base,
            property_access(
              match used {
                UsedName::Str(name) => vec![name].into_iter(),
                UsedName::Vec(names) => names.into_iter(),
              },
              0
            )
          ),
          None,
        )
      } else {
        init_fragments.push(
          NormalInitFragment::new(
            "var __webpack_unused_export__;\n".to_string(),
            InitFragmentStage::StageConstants,
            0,
            InitFragmentKey::unique(),
            None,
          )
          .boxed(),
        );
        source.replace(
          self.range.0,
          self.range.1,
          "__webpack_unused_export__",
          None,
        );
      }
    } else if self.base.is_define_property() {
      if let Some(value_range) = self.value_range {
        if let Some(used) = used {
          if let UsedName::Vec(used) = used {
            source.replace(
              self.range.0,
              value_range.0,
              &format!(
                "Object.defineProperty({}{}, {}, (",
                base,
                property_access(used[0..used.len() - 1].iter(), 0),
                serde_json::to_string(&used.last())
                  .expect("Unexpected render define property base")
              ),
              None,
            );
            source.replace(value_range.1, self.range.1, "))", None);
          } else {
            panic!("Unexpected base type");
          }
        } else {
          init_fragments.push(
            NormalInitFragment::new(
              "var __webpack_unused_export__;\n".to_string(),
              InitFragmentStage::StageConstants,
              0,
              InitFragmentKey::unique(),
              None,
            )
            .boxed(),
          );
          source.replace(
            self.range.0,
            value_range.0,
            "__webpack_unused_export__ = (",
            None,
          );
          source.replace(value_range.1, self.range.1, ")", None);
        }
      } else {
        panic!("Define property need value range");
      }
    } else {
      panic!("Unexpected base type");
    }
  }
}

impl AsContextDependency for CommonJsExportsDependency {}
