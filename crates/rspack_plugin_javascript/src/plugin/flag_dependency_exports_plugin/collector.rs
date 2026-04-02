use rspack_core::{ExportsProcessing, ExportsSpec};

use super::types::NormalizedModuleAnalysis;

pub(super) fn normalize_exports_spec(mut spec: ExportsSpec) -> NormalizedModuleAnalysis {
  match std::mem::take(&mut spec.processing) {
    ExportsProcessing::Immediate => NormalizedModuleAnalysis::from_local(spec),
    ExportsProcessing::DeferredReexport(deferred_reexports) => {
      NormalizedModuleAnalysis::from_deferred(spec, deferred_reexports)
    }
  }
}

#[cfg(test)]
mod tests {
  use rspack_core::{
    DeferredReexportItem, DeferredReexportSpec, DependencyId, ExportsOfExportsSpec,
    ExportsProcessing, ExportsSpec, ModuleIdentifier, Nullable,
  };

  use super::*;
  use crate::plugin::flag_dependency_exports_plugin::types::NormalizedModuleAnalysis;

  #[test]
  fn normalize_exports_spec_keeps_deferred_reexports_out_of_local_apply() {
    let target = ModuleIdentifier::from("leaf");
    let spec = ExportsSpec {
      exports: ExportsOfExportsSpec::Names(vec![]),
      processing: ExportsProcessing::DeferredReexport(vec![DeferredReexportSpec::new(
        target,
        DependencyId::from(7),
        vec![DeferredReexportItem {
          exposed_name: "value".into(),
          target_path: Nullable::Value(vec!["value".into()]),
          hidden: false,
        }],
      )]),
      ..Default::default()
    };

    let normalized = normalize_exports_spec(spec);
    assert!(normalized.local_apply.is_empty());
    assert_eq!(normalized.deferred_reexports.len(), 1);
  }

  #[test]
  fn bind_local_apply_preserves_fragment_multiplicity_for_one_dependency() {
    let dep_id = DependencyId::from(9);
    let analysis = NormalizedModuleAnalysis {
      local_apply: vec![
        ExportsSpec {
          exports: ExportsOfExportsSpec::UnknownExports,
          ..Default::default()
        },
        ExportsSpec {
          hide_export: Some(["value".into()].into_iter().collect()),
          ..Default::default()
        },
      ],
      deferred_reexports: Vec::new(),
    };

    let bound =
      NormalizedModuleAnalysis::bind_local_apply_with_dep_id(dep_id, analysis.local_apply);
    assert_eq!(bound.len(), 2);
    assert!(
      bound
        .iter()
        .all(|(bound_dep_id, _)| *bound_dep_id == dep_id)
    );
  }
}
