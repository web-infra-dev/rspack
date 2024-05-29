use std::{any::Any, fmt::Debug};

use dyn_clone::{clone_trait_object, DynClone};
use rspack_error::Diagnostic;
use rspack_util::ext::AsAny;
use rustc_hash::FxHashSet as HashSet;
use swc_core::{common::Span, ecma::atoms::Atom};

use super::dependency_template::AsDependencyTemplate;
use super::module_dependency::*;
use super::ExportsSpec;
use super::{DependencyCategory, DependencyId, DependencyType};
use crate::AsContextDependency;
use crate::Compilation;
use crate::{ConnectionState, Context, ErrorSpan, ModuleGraph, ModuleIdentifier, UsedByExports};

pub trait Dependency:
  AsDependencyTemplate
  + AsContextDependency
  + AsModuleDependency
  + AsAny
  + DynClone
  + Send
  + Sync
  + Debug
{
  /// name of the original struct or enum
  fn dependency_debug_name(&self) -> &'static str;

  fn id(&self) -> &DependencyId;

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Unknown
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::Unknown
  }

  fn get_context(&self) -> Option<&Context> {
    None
  }

  fn get_exports(&self, _mg: &ModuleGraph) -> Option<ExportsSpec> {
    None
  }

  fn set_used_by_exports(&mut self, _used_by_exports: Option<UsedByExports>) {}

  fn get_module_evaluation_side_effects_state(
    &self,
    _module_graph: &ModuleGraph,
    _module_chain: &mut HashSet<ModuleIdentifier>,
  ) -> ConnectionState {
    ConnectionState::Bool(true)
  }

  fn span(&self) -> Option<ErrorSpan> {
    None
  }

  fn source_order(&self) -> Option<i32> {
    None
  }

  /// `Span` used for Dependency search in `on_usage` in `InnerGraph`
  fn span_for_on_usage_search(&self) -> Option<ErrorSpan> {
    self.span()
  }

  fn is_span_equal(&self, other: &Span) -> bool {
    if let Some(err_span) = self.span_for_on_usage_search() {
      let other = ErrorSpan::from(*other);
      other == err_span
    } else {
      false
    }
  }

  // For now only `HarmonyImportSpecifierDependency` and
  // `HarmonyExportImportedSpecifierDependency` can use this method
  fn get_ids(&self, _mg: &ModuleGraph) -> Vec<Atom> {
    unreachable!()
  }

  fn resource_identifier(&self) -> Option<&str> {
    None
  }
}

impl dyn Dependency + '_ {
  pub fn downcast_ref<D: Any>(&self) -> Option<&D> {
    self.as_any().downcast_ref::<D>()
  }

  pub fn downcast_mut<D: Any>(&mut self) -> Option<&mut D> {
    self.as_any_mut().downcast_mut::<D>()
  }

  pub fn is<D: Any>(&self) -> bool {
    self.downcast_ref::<D>().is_some()
  }
}

pub trait AsDependency {
  fn as_dependency(&self) -> Option<Box<&dyn Dependency>> {
    None
  }
}

impl<T: Dependency> AsDependency for T {
  fn as_dependency(&self) -> Option<Box<&dyn Dependency>> {
    Some(Box::new(self))
  }
}

clone_trait_object!(Dependency);

pub type BoxDependency = Box<dyn Dependency>;
