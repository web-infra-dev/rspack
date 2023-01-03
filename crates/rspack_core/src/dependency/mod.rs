mod code_generatable;
pub use code_generatable::*;

mod commonjs;
pub use commonjs::*;
mod esm;
pub use esm::*;

use std::{any::Any, fmt::Debug};

use dyn_clone::{clone_trait_object, DynClone};

use crate::{AsAny, DynHash, ModuleIdentifier};

pub trait Dependency: AsAny + DynHash + DynClone + Debug {
  fn parent_module_identifier(&self) -> Option<&ModuleIdentifier>;
  fn as_module_dependency(&self) -> Option<&dyn ModuleDependency> {
    None
  }
}

pub enum ModuleDependencyCategory {
  ESM,
  CommonJS,
  URL,
}

pub trait ModuleDependency: Dependency {
  fn request(&self) -> &str;
  fn user_request(&self) -> &str;
  fn category(&self) -> ModuleDependencyCategory;
}

impl dyn Dependency + '_ {
  pub fn downcast_ref<D: Any>(&self) -> Option<&D> {
    self.as_any().downcast_ref::<D>()
  }

  pub fn downcast_mut<D: Any>(&mut self) -> Option<&mut D> {
    self.as_any_mut().downcast_mut::<D>()
  }
}

clone_trait_object!(Dependency);
clone_trait_object!(ModuleDependency);
