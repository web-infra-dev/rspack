use std::{
  cell::RefCell,
  sync::{Arc, Weak},
};

use napi_derive::napi;
use rspack_core::{AsyncDependenciesBlockRef, CompilationId, DependenciesBlock as _};
use rspack_napi::{OneShotInstanceRef, napi::bindgen_prelude::*};
use rustc_hash::FxHashMap as HashMap;

use crate::dependency::DependencyWrapper;

// The JS wrapper accesses the block directly without extending its lifetime.
#[napi]
pub struct AsyncDependenciesBlock {
  pub(crate) block_id: rspack_core::AsyncDependenciesBlockIdentifier,
  block: Weak<rspack_core::AsyncDependenciesBlock>,
  // Used for dependency graph properties only; reading the block never looks up a compilation.
  compilation_id: Option<CompilationId>,
}

impl AsyncDependenciesBlock {
  fn upgrade(&self) -> napi::Result<AsyncDependenciesBlockRef> {
    self.block.upgrade().ok_or_else(|| {
      napi::Error::from_reason(format!(
        "Unable to access async dependencies block with id = {:?} now. The block has been released on the Rust side.",
        self.block_id
      ))
    })
  }
}

#[napi]
impl AsyncDependenciesBlock {
  #[napi(getter, ts_return_type = "Dependency[]")]
  pub fn dependencies(&self) -> napi::Result<Vec<DependencyWrapper>> {
    Ok(
      self
        .upgrade()?
        .get_dependencies()
        .iter()
        .map(|dep| DependencyWrapper::new(dep.clone(), self.compilation_id))
        .collect(),
    )
  }

  #[napi(getter, ts_return_type = "AsyncDependenciesBlock[]")]
  pub fn blocks(&self) -> napi::Result<Vec<AsyncDependenciesBlockWrapper>> {
    Ok(
      self
        .upgrade()?
        .get_block_refs()
        .iter()
        .map(|block| AsyncDependenciesBlockWrapper::new(block.clone(), self.compilation_id))
        .collect(),
    )
  }
}

type BlockInstanceRefs =
  rspack_core::AsyncDependenciesBlockIdentifierMap<OneShotInstanceRef<AsyncDependenciesBlock>>;

type BlockInstanceRefsByCompilationId =
  RefCell<HashMap<rspack_core::CompilationId, BlockInstanceRefs>>;

thread_local! {
  static BLOCK_INSTANCE_REFS: BlockInstanceRefsByCompilationId = Default::default();
}

pub struct AsyncDependenciesBlockWrapper {
  block: AsyncDependenciesBlockRef,
  compilation_id: Option<CompilationId>,
}

impl AsyncDependenciesBlockWrapper {
  pub fn new(block: AsyncDependenciesBlockRef, compilation_id: Option<CompilationId>) -> Self {
    Self {
      block,
      compilation_id,
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
      let block_id = val.block.identifier();
      let Some(compilation_id) = val.compilation_id else {
        return AsyncDependenciesBlock::to_napi_value(
          env,
          AsyncDependenciesBlock {
            block_id,
            block: Arc::downgrade(&val.block),
            compilation_id: None,
          },
        );
      };
      BLOCK_INSTANCE_REFS.with(|refs| {
        let mut refs_by_compilation_id = refs.borrow_mut();
        let entry = refs_by_compilation_id.entry(compilation_id);
        let refs = match entry {
          std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
          std::collections::hash_map::Entry::Vacant(entry) => {
            let refs = rspack_core::AsyncDependenciesBlockIdentifierMap::default();
            entry.insert(refs)
          }
        };

        match refs.entry(block_id) {
          std::collections::hash_map::Entry::Occupied(mut occupied_entry) => {
            let r = occupied_entry.get_mut();
            // A rebuild can create a new block with the same identifier.
            r.block = Arc::downgrade(&val.block);
            ToNapiValue::to_napi_value(env, r)
          }
          std::collections::hash_map::Entry::Vacant(vacant_entry) => {
            let js_block = AsyncDependenciesBlock {
              block_id,
              block: Arc::downgrade(&val.block),
              compilation_id: Some(compilation_id),
            };
            let r = vacant_entry.insert(OneShotInstanceRef::new(env, js_block)?);
            ToNapiValue::to_napi_value(env, r)
          }
        }
      })
    }
  }
}
