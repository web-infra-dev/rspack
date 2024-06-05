use rspack_core::{
  filter_runtime, runtime_condition_expression, AsContextDependency, AsModuleDependency,
  ConnectionState, Dependency, DependencyId, DependencyTemplate, ModuleGraph, ModuleIdentifier,
  TemplateContext, TemplateReplaceSource, UsageState, UsedByExports, UsedName,
};
use rustc_hash::FxHashSet as HashSet;
#[derive(Debug, Clone)]
pub struct PureExpressionDependency {
  pub start: u32,
  pub end: u32,
  used_by_exports: Option<UsedByExports>,
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

impl AsModuleDependency for PureExpressionDependency {}

impl DependencyTemplate for PureExpressionDependency {
  fn apply(&self, source: &mut TemplateReplaceSource, ctx: &mut TemplateContext) {
    let condition = match self.used_by_exports {
      Some(UsedByExports::Bool(true)) => {
        unreachable!()
      }
      Some(UsedByExports::Bool(false)) => None,
      Some(UsedByExports::Set(ref set)) => {
        let module_graph = ctx.compilation.get_module_graph();
        let exports_info = module_graph.get_exports_info(&self.module_identifier);
        let runtime = ctx.runtime;
        let runtime_condition = filter_runtime(runtime, |cur_runtime| {
          set.iter().any(|id| {
            exports_info.get_used(UsedName::Str(id.clone()), cur_runtime, &module_graph)
              != UsageState::Unused
          })
        });
        match &runtime_condition {
          rspack_core::RuntimeCondition::Boolean(true) => return,
          rspack_core::RuntimeCondition::Boolean(false) => None,
          rspack_core::RuntimeCondition::Spec(_spec) => Some(runtime_condition_expression(
            &ctx.compilation.chunk_graph,
            Some(&runtime_condition),
            runtime,
            ctx.runtime_requirements,
          )),
        }
      }
      None => {
        // https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/dependencies/PureExpressionDependency.js#L32-L33
        // after check usedExports is not false, webpack ensure that usedExports is a set
        unreachable!()
      }
    };

    if let Some(condition) = condition {
      source.insert(
        self.start,
        &format!("(/* runtime-dependent pure expression or super */ {condition} ? ("),
        None,
      );
      source.insert(self.end, ") : null)", None);
    } else {
      source.insert(
        self.start,
        "(/* unused pure expression or super */ null && (",
        None,
      );
      source.insert(self.end, "))", None);
    }
  }

  fn dependency_id(&self) -> Option<DependencyId> {
    Some(self.id)
  }
}

impl AsContextDependency for PureExpressionDependency {}
