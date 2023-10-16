use rspack_core::{
  AsModuleDependency, Dependency, DependencyId, DependencyTemplate, ModuleDependency,
  TemplateContext, TemplateReplaceSource,
};

pub const DEFAULT_EXPORT: &str = "__WEBPACK_DEFAULT_EXPORT__";
// pub const NAMESPACE_OBJECT_EXPORT: &'static str = "__WEBPACK_NAMESPACE_OBJECT__";

#[derive(Debug, Clone)]
pub struct AnonymousFunctionRangeInfo {
  pub is_async: bool,
  pub is_generator: bool,
  pub body_start: u32,
  pub first_parmas_start: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct HarmonyExportExpressionDependency {
  pub start: u32,
  pub end: u32,
  pub declaration: bool,
  pub function: Option<AnonymousFunctionRangeInfo>,
  pub id: DependencyId,
}

impl HarmonyExportExpressionDependency {
  pub fn new(
    start: u32,
    end: u32,
    declaration: bool,
    function: Option<AnonymousFunctionRangeInfo>,
  ) -> Self {
    Self {
      start,
      end,
      declaration,
      function,
      id: DependencyId::default(),
    }
  }
}

impl Dependency for HarmonyExportExpressionDependency {
  fn id(&self) -> &rspack_core::DependencyId {
    &self.id
  }

  fn get_module_evaluation_side_effects_state(
    &self,
    _module_graph: &rspack_core::ModuleGraph,
    _module_chain: &mut rustc_hash::FxHashSet<rspack_core::ModuleIdentifier>,
  ) -> rspack_core::ConnectionState {
    rspack_core::ConnectionState::Bool(false)
  }
}

impl AsModuleDependency for HarmonyExportExpressionDependency {
  fn as_module_dependency(&self) -> Option<&dyn ModuleDependency> {
    None
  }

  fn as_module_dependency_mut(&mut self) -> Option<&mut dyn ModuleDependency> {
    None
  }
}

impl DependencyTemplate for HarmonyExportExpressionDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    _code_generatable_context: &mut TemplateContext,
  ) {
    if self.declaration {
      source.replace(self.start, self.end, "", None);
    } else if let Some(AnonymousFunctionRangeInfo {
      is_async,
      is_generator,
      body_start,
      first_parmas_start,
    }) = &self.function
    {
      // hoist anonymous function
      let prefix = format!(
        "{}function{} {DEFAULT_EXPORT}",
        if *is_async { "async " } else { "" },
        if *is_generator { "*" } else { "" },
      );
      if let Some(first_parmas_start) = first_parmas_start {
        source.replace(self.start, first_parmas_start - 1, prefix.as_str(), None);
      } else {
        source.replace(
          self.start,
          *body_start,
          format!("{prefix}()").as_str(),
          None,
        );
      }
    } else {
      source.replace(
        self.start,
        self.end,
        format!("var {DEFAULT_EXPORT} = ").as_str(),
        None,
      );
    }
  }
}
