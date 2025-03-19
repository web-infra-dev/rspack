use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AffectType, AsContextDependency, AsDependencyTemplate, Context, Dependency, DependencyId,
  DependencyType, FactorizeInfo, ModuleDependency,
};

use crate::DllEntryPluginOptions;

#[cacheable]
#[derive(Debug, Clone)]
pub struct DllEntryDependency {
  pub context: Context,

  pub entries: Vec<String>,

  // TODO: The fields `name` for serialize & deserialize.
  pub name: String,

  id: DependencyId,

  factorize_info: FactorizeInfo,
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
      factorize_info: Default::default(),
    }
  }
}

// It would not create module by rspack,if dependency is not ModuleDependency.
// So we impl ModuleDependency for [DllEntryDependency]
#[cacheable_dyn]
impl ModuleDependency for DllEntryDependency {
  fn request(&self) -> &str {
    "dll main"
  }

  fn factorize_info(&self) -> &FactorizeInfo {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
    &mut self.factorize_info
  }
}

#[cacheable_dyn]
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

impl AsContextDependency for DllEntryDependency {}

impl AsDependencyTemplate for DllEntryDependency {}
