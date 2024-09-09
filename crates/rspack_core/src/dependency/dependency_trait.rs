use std::{any::Any, fmt::Debug};

use dyn_clone::{clone_trait_object, DynClone};
use rspack_collections::IdentifierSet;
use rspack_error::Diagnostic;
use rspack_util::ext::AsAny;
use swc_core::ecma::atoms::Atom;

use super::dependency_template::AsDependencyTemplate;
use super::module_dependency::*;
use super::ExportsSpec;
use super::RealDependencyLocation;
use super::{DependencyCategory, DependencyId, DependencyType};
use crate::create_exports_object_referenced;
use crate::AsContextDependency;
use crate::ExtendedReferencedExport;
use crate::ImportAttributes;
use crate::ModuleLayer;
use crate::RuntimeSpec;
use crate::{ConnectionState, Context, ModuleGraph, UsedByExports};

#[derive(Debug, Clone, Copy)]
pub enum AffectType {
  True,
  False,
  Transitive,
}

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
  fn id(&self) -> &DependencyId;

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Unknown
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::Unknown
  }

  // get issuer context
  fn get_context(&self) -> Option<&Context> {
    None
  }

  // get issuer layer
  fn get_layer(&self) -> Option<&ModuleLayer> {
    None
  }

  fn get_attributes(&self) -> Option<&ImportAttributes> {
    None
  }

  fn get_exports(&self, _mg: &ModuleGraph) -> Option<ExportsSpec> {
    None
  }

  fn set_used_by_exports(&mut self, _used_by_exports: Option<UsedByExports>) {}

  fn get_module_evaluation_side_effects_state(
    &self,
    _module_graph: &ModuleGraph,
    _module_chain: &mut IdentifierSet,
  ) -> ConnectionState {
    ConnectionState::Bool(true)
  }

  fn loc(&self) -> Option<String> {
    None
  }

  fn range(&self) -> Option<&RealDependencyLocation> {
    None
  }

  fn source_order(&self) -> Option<i32> {
    None
  }

  // For now only `HarmonyImportSpecifierDependency` and
  // `HarmonyExportImportedSpecifierDependency` can use this method
  fn get_ids(&self, _mg: &ModuleGraph) -> Vec<Atom> {
    unreachable!()
  }

  fn resource_identifier(&self) -> Option<&str> {
    None
  }

  fn get_diagnostics(&self, _module_graph: &ModuleGraph) -> Option<Vec<Diagnostic>> {
    None
  }

  fn get_referenced_exports(
    &self,
    _module_graph: &ModuleGraph,
    _runtime: Option<&RuntimeSpec>,
  ) -> Vec<ExtendedReferencedExport> {
    create_exports_object_referenced()
  }

  fn could_affect_referencing_module(&self) -> AffectType;
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
