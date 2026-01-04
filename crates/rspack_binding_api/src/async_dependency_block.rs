use std::{cell::RefCell, ptr::NonNull};

use napi_derive::napi;
use rspack_core::{DependenciesBlock as _, internal};
use rspack_napi::{OneShotRef, napi::bindgen_prelude::*};
use rustc_hash::FxHashMap as HashMap;

use crate::dependency::DependencyWrapper;

#[napi]
pub struct AsyncDependenciesBlock {
  pub(crate) block_id: rspack_core::AsyncDependenciesBlockIdentifier,
  compilation: NonNull<rspack_core::Compilation>,
}

#[napi]
impl AsyncDependenciesBlock {
  #[napi(getter, ts_return_type = "Dependency[]")]
  pub fn dependencies(&mut self) -> Vec<DependencyWrapper> {
    let compilation = unsafe { self.compilation.as_ref() };
    let module_graph = compilation.get_module_graph();
    if let Some(block) = module_graph.block_by_id(&self.block_id) {
      block
        .get_dependencies()
        .iter()
        .filter_map(|dependency_id| {
          internal::try_dependency_by_id(&module_graph, dependency_id).map(|dep| {
            DependencyWrapper::new(
              (&**dep) as &dyn rspack_core::Dependency,
              compilation.id(),
              Some(compilation),
            )
          })
        })
        .collect::<Vec<_>>()
    } else {
      vec![]
    }
  }

  #[napi(getter, ts_return_type = "AsyncDependenciesBlock[]")]
  pub fn blocks(&mut self) -> Vec<AsyncDependenciesBlockWrapper> {
    let compilation = unsafe { self.compilation.as_ref() };
    let module_graph = compilation.get_module_graph();
    if let Some(block) = module_graph.block_by_id(&self.block_id) {
      block
        .get_blocks()
        .iter()
        .filter_map(|block_id| {
          module_graph
            .block_by_id(block_id)
            .map(|block| AsyncDependenciesBlockWrapper::new(block, compilation))
        })
        .collect::<Vec<_>>()
    } else {
      vec![]
    }
  }
}

type BlockInstanceRefs = HashMap<rspack_core::AsyncDependenciesBlockIdentifier, OneShotRef>;

type BlockInstanceRefsByCompilationId =
  RefCell<HashMap<rspack_core::CompilationId, BlockInstanceRefs>>;

thread_local! {
  static BLOCK_INSTANCE_REFS: BlockInstanceRefsByCompilationId = Default::default();
}

pub struct AsyncDependenciesBlockWrapper {
  block_id: rspack_core::AsyncDependenciesBlockIdentifier,
  compilation: NonNull<rspack_core::Compilation>,
}

impl AsyncDependenciesBlockWrapper {
  pub fn new(
    block: &rspack_core::AsyncDependenciesBlock,
    compilation: &rspack_core::Compilation,
  ) -> Self {
    let block_id = block.identifier();

    #[allow(clippy::unwrap_used)]
    Self {
      block_id,
      compilation: NonNull::new(
        compilation as *const rspack_core::Compilation as *mut rspack_core::Compilation,
      )
      .unwrap(),
    }
  }

  pub fn cleanup_last_compilation(compilation_id: rspack_core::CompilationId) {
    BLOCK_INSTANCE_REFS.with(|refs| {
      let mut refs_by_compilation_id = refs.borrow_mut();
      refs_by_compilation_id.remove(&compilation_id)
    });
  }
}

impl ToNapiValue for AsyncDependenciesBlockWrapper {
  unsafe fn to_napi_value(
    env: napi::sys::napi_env,
    val: Self,
  ) -> napi::Result<napi::sys::napi_value> {
    unsafe {
      BLOCK_INSTANCE_REFS.with(|refs| {
        let compilation = val.compilation.as_ref();
        let mut refs_by_compilation_id = refs.borrow_mut();
        let entry = refs_by_compilation_id.entry(compilation.id());
        let refs = match entry {
          std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
          std::collections::hash_map::Entry::Vacant(entry) => {
            let refs = HashMap::default();
            entry.insert(refs)
          }
        };

        match refs.entry(val.block_id) {
          std::collections::hash_map::Entry::Occupied(occupied_entry) => {
            let r = occupied_entry.get();
            ToNapiValue::to_napi_value(env, r)
          }
          std::collections::hash_map::Entry::Vacant(vacant_entry) => {
            let js_block = AsyncDependenciesBlock {
              block_id: val.block_id,
              compilation: val.compilation,
            };
            let r = vacant_entry.insert(OneShotRef::new(env, js_block)?);
            ToNapiValue::to_napi_value(env, r)
          }
        }
      })
    }
  }
}
