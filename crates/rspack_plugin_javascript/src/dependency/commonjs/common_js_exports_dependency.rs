use rspack_core::{
  module_id, property_access, Dependency, DependencyCategory, DependencyId, DependencyTemplate,
  DependencyType, ErrorSpan, ExportsArgument, ModuleArgument, ModuleDependency, RuntimeGlobals,
  TemplateContext, TemplateReplaceSource, UsedName,
};
use swc_core::ecma::atoms::JsWord;

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
  value_range: (u32, u32),
  base: ExportsBase,
  names: UsedName,
}

impl CommonJsExportsDependency {
  pub fn new(
    range: (u32, u32),
    value_range: (u32, u32),
    base: ExportsBase,
    names: UsedName,
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
      .get_used_name(&compilation.module_graph, *runtime, self.names);

    let exports_argument = mgm.get_exports_argument();
    let module_argument = mgm.get_module_argument();

    let base = if self.base.is_exports() {
      exports_argument.to_string()
    } else if self.base.is_module_exports() {
      format!("{}.exports", module_argument)
    } else if self.base.is_this() {
      "this".to_string()
    } else {
      panic!("Unsupport base type");
    };

    if self.base.is_expression() {
      /*
      if (!used) {
          initFragments.push(
            new InitFragment(
              "var __webpack_unused_export__;\n",
              InitFragment.STAGE_CONSTANTS,
              0,
              "__webpack_unused_export__"
            )
          );
          source.replace(
            dep.range[0],
            dep.range[1] - 1,
            "__webpack_unused_export__"
          );
          return;
        }
        source.replace(
          dep.range[0],
          dep.range[1] - 1,
          `${base}${propertyAccess(used)}`
        );
       */
      if let Some(used) = used {
      } else {
        source.replace(
          self.range.0,
          self.range.1,
          &format!("{}{}", base, property_access(used.iter(), 0)),
          None,
        )
      }
    } else if self.base.is_define_property() {
    } else {
      panic!("Unsupport base type");
    }
  }
}
