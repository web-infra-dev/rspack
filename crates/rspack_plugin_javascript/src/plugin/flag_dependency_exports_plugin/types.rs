use rspack_collections::IdentifierMap;
use rspack_core::{
  DeferredReexportSpec, DependencyId, ExportsOfExportsSpec, ExportsProcessing, ExportsSpec,
  ModuleIdentifier,
};

#[derive(Debug, Default)]
pub(super) struct NormalizedModuleAnalysis {
  pub local_apply: Vec<ExportsSpec>,
  pub deferred_reexports: Vec<DeferredReexportSpec>,
}

#[derive(Debug, Default)]
pub(super) struct ModuleCollectedAnalysis {
  pub flat_local_apply: Vec<(DependencyId, ExportsSpec)>,
  pub structured_local_apply: Vec<(DependencyId, ExportsSpec)>,
  pub deferred_reexports: Vec<DeferredReexportSpec>,
}

#[derive(Debug, Default)]
pub(super) struct CollectedDependencyExportsAnalysis {
  modules: IdentifierMap<ModuleCollectedAnalysis>,
}

impl CollectedDependencyExportsAnalysis {
  pub fn insert(
    &mut self,
    module_identifier: ModuleIdentifier,
    analysis: ModuleCollectedAnalysis,
  ) -> Option<ModuleCollectedAnalysis> {
    self.modules.insert(module_identifier, analysis)
  }

  pub fn get(&self, module_identifier: &ModuleIdentifier) -> Option<&ModuleCollectedAnalysis> {
    self.modules.get(module_identifier)
  }
}

impl NormalizedModuleAnalysis {
  pub(super) fn from_local(spec: ExportsSpec) -> Self {
    Self {
      local_apply: vec![spec],
      deferred_reexports: Vec::new(),
    }
  }

  pub(super) fn from_deferred(
    mut spec: ExportsSpec,
    deferred_reexports: Vec<DeferredReexportSpec>,
  ) -> Self {
    spec.processing = ExportsProcessing::Immediate;

    let local_apply = if has_local_apply_work(&spec) {
      vec![spec]
    } else {
      Vec::new()
    };

    Self {
      local_apply,
      deferred_reexports,
    }
  }

  pub(super) fn bind_local_apply_with_dep_id(
    dep_id: DependencyId,
    local_apply: Vec<ExportsSpec>,
  ) -> Vec<(DependencyId, ExportsSpec)> {
    local_apply.into_iter().map(|spec| (dep_id, spec)).collect()
  }
}

fn has_local_apply_work(spec: &ExportsSpec) -> bool {
  (match &spec.exports {
    ExportsOfExportsSpec::UnknownExports => true,
    ExportsOfExportsSpec::NoExports => false,
    ExportsOfExportsSpec::Names(exports) => !exports.is_empty(),
  }) || spec
    .hide_export
    .as_ref()
    .is_some_and(|exports| !exports.is_empty())
    || spec
      .exclude_exports
      .as_ref()
      .is_some_and(|exports| !exports.is_empty())
}
