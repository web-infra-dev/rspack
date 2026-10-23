use rspack_intern::Atom;

use crate::{
  CircularModulesInfo, Compilation, ImportedByDeferModulesArtifact, ModuleGraph, ModuleIdentifier,
};

/// Whether a binding from `module` may be emitted as a stable value
/// (export value descriptor or import-side snapshot).
///
/// `local` is the module-local name when known. Pass `None` when there is no
/// user local to check (Dynamic CJS default interop, anonymous default expr).
pub fn can_emit_stable_binding(
  compilation: &Compilation,
  module_identifier: &ModuleIdentifier,
  local: Option<&Atom>,
) -> bool {
  can_emit_stable_binding_with(
    &compilation.circular_modules,
    compilation.get_module_graph(),
    &compilation.imported_by_defer_modules_artifact,
    module_identifier,
    local,
  )
}

pub fn can_emit_stable_binding_with(
  circular_modules: &CircularModulesInfo,
  module_graph: &ModuleGraph,
  imported_by_defer_modules_artifact: &ImportedByDeferModulesArtifact,
  module_identifier: &ModuleIdentifier,
  local: Option<&Atom>,
) -> bool {
  if !matches!(
    circular_modules.is_circular_module(module_identifier),
    Some(false)
  ) {
    return false;
  }

  if module_graph.is_deferred(imported_by_defer_modules_artifact, module_identifier) {
    return false;
  }

  let Some(module) = module_graph.module_by_identifier(module_identifier) else {
    return false;
  };
  let build_info = module.build_info();
  if build_info.has_direct_eval {
    return false;
  }
  if let Some(local) = local
    && build_info.mutated_bindings.contains(local)
  {
    return false;
  }
  true
}
