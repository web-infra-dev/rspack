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

  #[test]
  fn normalize_exports_spec_keeps_deferred_reexports_out_of_local_apply() {
    let target = ModuleIdentifier::from("leaf");
    let spec = ExportsSpec {
      exports: ExportsOfExportsSpec::Names(vec![]),
      processing: ExportsProcessing::DeferredReexport(vec![DeferredReexportSpec {
        target_module: target,
        dep_id: DependencyId::from(7),
        items: vec![DeferredReexportItem {
          exposed_name: "value".into(),
          target_path: Nullable::Value(vec!["value".into()]),
          hidden: false,
        }],
        ..Default::default()
      }]),
      ..Default::default()
    };

    let normalized = normalize_exports_spec(spec);
    assert!(normalized.local_apply.is_empty());
    assert_eq!(normalized.deferred_reexports.len(), 1);
  }
}
