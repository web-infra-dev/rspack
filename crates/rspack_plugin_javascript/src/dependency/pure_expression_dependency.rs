use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_collections::IdentifierSet;
use rspack_core::{
  filter_runtime, runtime_condition_expression, AsContextDependency, AsModuleDependency,
  Compilation, ConnectionState, Dependency, DependencyId, DependencyTemplate, ModuleGraph,
  ModuleIdentifier, RuntimeCondition, RuntimeSpec, TemplateContext, TemplateReplaceSource,
  UsageState, UsedByExports, UsedName,
};
use rspack_util::ext::DynHash;

#[cacheable]
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

  fn get_runtime_condition(
    &self,
    compilation: &Compilation,
    runtime: Option<&RuntimeSpec>,
  ) -> RuntimeCondition {
    match self.used_by_exports {
      Some(UsedByExports::Bool(true)) => {
        unreachable!()
      }
      Some(UsedByExports::Bool(false)) => RuntimeCondition::Boolean(false),
      Some(UsedByExports::Set(ref set)) => {
        let module_graph = compilation.get_module_graph();
        let exports_info = module_graph.get_exports_info(&self.module_identifier);
        filter_runtime(runtime, |cur_runtime| {
          set.iter().any(|id| {
            exports_info.get_used(&module_graph, UsedName::Str(id.clone()), cur_runtime)
              != UsageState::Unused
          })
        })
      }
      None => {
        // https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/dependencies/PureExpressionDependency.js#L32-L33
        // after check usedExports is not false, webpack ensure that usedExports is a set
        unreachable!()
      }
    }
  }
}

#[cacheable_dyn]
impl Dependency for PureExpressionDependency {
  fn id(&self) -> &rspack_core::DependencyId {
    &self.id
  }

  fn set_used_by_exports(&mut self, used_by_exports: Option<UsedByExports>) {
    self.used_by_exports = used_by_exports;
  }

  fn get_module_evaluation_side_effects_state(
    &self,
    _module_graph: &ModuleGraph,
    _module_chain: &mut IdentifierSet,
  ) -> ConnectionState {
    ConnectionState::Bool(false)
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::False
  }
}

impl AsModuleDependency for PureExpressionDependency {}

#[cacheable_dyn]
impl DependencyTemplate for PureExpressionDependency {
  fn apply(&self, source: &mut TemplateReplaceSource, ctx: &mut TemplateContext) {
    let runtime_condition = self.get_runtime_condition(ctx.compilation, ctx.runtime);
    let condition = match &runtime_condition {
      rspack_core::RuntimeCondition::Boolean(true) => return,
      rspack_core::RuntimeCondition::Boolean(false) => None,
      rspack_core::RuntimeCondition::Spec(_spec) => Some(runtime_condition_expression(
        &ctx.compilation.chunk_graph,
        Some(&runtime_condition),
        ctx.runtime,
        ctx.runtime_requirements,
      )),
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

  fn update_hash(
    &self,
    hasher: &mut dyn std::hash::Hasher,
    compilation: &Compilation,
    runtime: Option<&RuntimeSpec>,
  ) {
    let runtime_condition = self.get_runtime_condition(compilation, runtime);
    runtime_condition.dyn_hash(hasher);
  }
}

impl AsContextDependency for PureExpressionDependency {}
