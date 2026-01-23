/// Internal helpers for ModuleGraph that should only be used by specific modules.
///
/// **DO NOT USE THESE FUNCTIONS** unless you're in the appropriate restricted context.
///
/// This module provides restricted access to potentially unsafe ModuleGraph operations
/// that should only be used in specific contexts where items may legitimately not exist.
use crate::{BoxDependency, DependencyId, ModuleGraph, ModuleGraphModule, ModuleIdentifier};

/// Try to get a dependency by ID, returning None if not found.
///
/// # Restricted Use - BINDING LAYER ONLY
///
/// **WARNING**: This function should ONLY be used in the `rspack_binding_api` crate
/// for JavaScript/Node.js API bindings that need to handle missing dependencies
/// gracefully for external API consumers.
///
/// **All internal Rust code must use `ModuleGraph::dependency_by_id()`** instead,
/// which enforces the invariant that dependencies exist and provides clear panic
/// messages when this expectation is violated.
///
/// # When to Use
///
/// Only use in binding layer code where:
/// - You're exposing APIs to JavaScript/Node.js
/// - External consumers might query non-existent dependencies
/// - Graceful None handling is appropriate for the external API contract
///
/// If you're writing internal Rust code, use `dependency_by_id()` instead.
#[inline]
pub fn try_dependency_by_id<'a>(
  module_graph: &'a ModuleGraph,
  dependency_id: &DependencyId,
) -> Option<&'a BoxDependency> {
  module_graph.inner.dependencies.get(dependency_id)
}
#[inline]
pub fn try_dependency_by_id_mut<'a>(
  module_graph: &'a mut ModuleGraph,
  dependency_id: &DependencyId,
) -> Option<&'a mut BoxDependency> {
  module_graph.inner.dependencies.get_mut(dependency_id)
}

/// Try to get a mutable module graph module by identifier, returning None if not found.
///
/// # Restricted Use - DO NOT USE
///
/// **WARNING**: This function should ONLY be used in `compilation::build_module_graph`
/// for handling module removal during graph updates where modules may have been removed
/// during incremental compilation.
///
/// **All other code must use `ModuleGraph::module_graph_module_by_identifier_mut()`**
/// which enforces the invariant that the module exists with a clear panic message.
///
/// # When to Use (only in build_module_graph)
///
/// Only use when you have a legitimate reason to expect the module might not exist:
/// - During incremental compilation where modules may have been removed
/// - During graph cleanup operations where referenced modules may already be deleted
///
/// If you're unsure, use `module_graph_module_by_identifier_mut()` instead.
#[inline]
pub(crate) fn try_get_module_graph_module_mut_by_identifier<'a>(
  module_graph: &'a mut ModuleGraph,
  identifier: &ModuleIdentifier,
) -> Option<&'a mut ModuleGraphModule> {
  module_graph.inner.module_graph_modules.get_mut(identifier)
}
