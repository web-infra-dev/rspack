use rspack_core::{
  AsModuleDependency, ConnectionState, Dependency, DependencyId, DependencyTemplate, ModuleGraph,
  ModuleIdentifier, TemplateContext, TemplateReplaceSource, UsageState, UsedByExports, UsedName,
};
use rustc_hash::FxHashSet as HashSet;
#[derive(Debug, Clone)]
pub struct PureExpressionDependency {
  pub start: u32,
  pub end: u32,
  pub used_by_exports: Option<UsedByExports>,
  id: DependencyId,
  pub module_identifier: ModuleIdentifier,
}

impl PureExpressionDependency {
  pub fn new(start: u32, end: u32, module_identifier: ModuleIdentifier) -> Self {
    Self {
      start,
      end,
      used_by_exports: None,
      id: DependencyId::default(),
      module_identifier,
    }
  }
}

impl Dependency for PureExpressionDependency {
  fn id(&self) -> &rspack_core::DependencyId {
    &self.id
  }

  fn set_used_by_exports(&mut self, used_by_exports: Option<UsedByExports>) {
    self.used_by_exports = used_by_exports;
  }

  fn dependency_debug_name(&self) -> &'static str {
    "PureExpressionDependency"
  }
  fn get_module_evaluation_side_effects_state(
    &self,
    _module_graph: &ModuleGraph,
    _module_chain: &mut HashSet<ModuleIdentifier>,
  ) -> ConnectionState {
    ConnectionState::Bool(false)
  }
}

impl AsModuleDependency for PureExpressionDependency {
  fn as_module_dependency(&self) -> Option<&dyn rspack_core::ModuleDependency> {
    None
  }

  fn as_module_dependency_mut(&mut self) -> Option<&mut dyn rspack_core::ModuleDependency> {
    None
  }
}

impl DependencyTemplate for PureExpressionDependency {
  fn apply(&self, source: &mut TemplateReplaceSource, ctx: &mut TemplateContext) {
    match self.used_by_exports {
      Some(UsedByExports::Bool(true)) => {
        unreachable!()
      }
      Some(UsedByExports::Bool(false)) => {}
      Some(UsedByExports::Set(ref set)) => {
        let exports_info = ctx
          .compilation
          .module_graph
          .get_exports_info(&self.module_identifier);
        // TODO: runtime optimization,
        let runtime_condition = set.iter().any(|id| {
          exports_info.get_used(
            UsedName::Str(id.clone()),
            None,
            &ctx.compilation.module_graph,
          ) != UsageState::Unused
        });
        if runtime_condition {
          return;
        }
      }
      None => {
        // https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/dependencies/PureExpressionDependency.js#L32-L33
        // https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/dependencies/PureExpressionDependency.js#L103-L113
        // after check usedExports is not false, webpack ensure that usedExports is a set
        unreachable!()
      }
    }

    source.insert(
      self.start,
      "(/* unused pure expression or super */ null && (",
      None,
    );
    source.insert(self.end, "))", None);
  }
}
