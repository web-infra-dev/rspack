use rspack_core::{
  AffectType, AsContextDependency, AsDependencyTemplate, AsModuleDependency, Context, Dependency,
  DependencyId, DependencyType,
};

use crate::DllEntryPluginOptions;

#[derive(Debug, Clone)]
pub struct DllEntryDependency {
  pub context: Context,

  pub entries: Vec<String>,

  // TODO: pass by serialize & deserialize.
  pub name: String,

  id: DependencyId,
}

impl DllEntryDependency {
  pub fn new(dll_entry_plugin_optinos: &DllEntryPluginOptions) -> Self {
    let DllEntryPluginOptions {
      context,
      entries,
      name,
    } = dll_entry_plugin_optinos.clone();

    Self {
      context,
      entries,
      name,
      id: DependencyId::new(),
    }
  }
}

impl Dependency for DllEntryDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn could_affect_referencing_module(&self) -> AffectType {
    AffectType::Transitive
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::DllEntry
  }
}

impl AsModuleDependency for DllEntryDependency {}

impl AsContextDependency for DllEntryDependency {}

impl AsDependencyTemplate for DllEntryDependency {}
