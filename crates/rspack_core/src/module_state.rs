use std::fmt::Debug;

use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_util::ext::AsAny;

use crate::{BuildInfo, BuildMeta};

/// Type-erased, serializable build output. Factory data stays on the module.
#[cacheable_dyn]
pub trait ModuleState: Debug + Send + Sync + AsAny {}

/// Common build output for modules without additional build-owned fields.
///
/// Cloning isolates mutable build metadata from later compilation mutations so
/// a cache entry can be reused independently of the module graph.
#[cacheable]
#[derive(Debug, Default, Clone)]
pub struct BaseModuleState {
  pub build_info: BuildInfo,
  pub build_meta: BuildMeta,
}

#[cacheable_dyn]
impl ModuleState for BaseModuleState {}

/// Associates a concrete module with its build state. Copies are needed when
/// publishing and restoring independent cache entries, not when sharing entries.
pub trait HasModuleState {
  type State: ModuleState + Clone + 'static;

  fn module_state(&self) -> &Self::State;
  /// Returns the displaced state so cache validation can roll back on a miss.
  fn restore_module_state(&mut self, state: Self::State) -> Self::State;
}

/// Bridges typed module implementations to the dynamic module cache boundary.
pub trait ModuleStateAccess {
  fn capture_state(&self) -> Box<dyn ModuleState>;
  fn restore_state(&mut self, state: &dyn ModuleState) -> Option<Box<dyn ModuleState>>;
}

impl<M: HasModuleState> ModuleStateAccess for M {
  fn capture_state(&self) -> Box<dyn ModuleState> {
    Box::new(self.module_state().clone())
  }

  fn restore_state(&mut self, state: &dyn ModuleState) -> Option<Box<dyn ModuleState>> {
    let state = state.as_any().downcast_ref::<M::State>()?;
    Some(Box::new(self.restore_module_state(state.clone())))
  }
}

/// Implements typed access for modules whose state needs no restoration fixups.
#[macro_export]
macro_rules! impl_module_state {
  ($module:ty, $state:ty) => {
    impl $crate::HasModuleState for $module {
      type State = $state;

      fn module_state(&self) -> &Self::State {
        &self.state
      }

      fn restore_module_state(&mut self, state: Self::State) -> Self::State {
        std::mem::replace(&mut self.state, state)
      }
    }
  };
}

/// Exposes typed state access through the object-safe Module interface.
#[macro_export]
macro_rules! impl_module_state_access {
  () => {
    fn capture_build_state(&self) -> Option<Box<dyn $crate::ModuleState>> {
      Some($crate::ModuleStateAccess::capture_state(self))
    }

    fn restore_build_state(
      &mut self,
      state: &dyn $crate::ModuleState,
    ) -> Option<Box<dyn $crate::ModuleState>> {
      $crate::ModuleStateAccess::restore_state(self, state)
    }
  };
}
