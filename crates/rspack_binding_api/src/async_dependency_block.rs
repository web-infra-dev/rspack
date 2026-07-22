use std::cell::RefCell;

use napi_derive::napi;
use rspack_core::{Compilation, CompilerId, DependenciesBlock as _, internal};
use rspack_napi::{OneShotRef, napi::bindgen_prelude::*};
use rustc_hash::FxHashMap as HashMap;

use crate::{COMPILER_REFERENCES, dependency::DependencyWrapper};

#[napi]
pub struct AsyncDependenciesBlock {
  pub(crate) block_id: rspack_core::AsyncDependenciesBlockIdentifier,
  compiler_id: CompilerId,
}

impl AsyncDependenciesBlock {
  fn compiler_garbage_collected_error(&self) -> napi::Error {
    napi::Error::from_reason(format!(
      "Unable to access async dependencies block with id = {:?} now. The Compiler has been garbage collected by JavaScript.",
      self.block_id
    ))
  }

  fn block_removed_error(&self) -> napi::Error {
    napi::Error::from_reason(format!(
      "Unable to access async dependencies block with id = {:?} now. The block has been removed on the Rust side.",
      self.block_id
    ))
  }

  fn with_compilation<R>(
    &self,
    f: impl FnOnce(&Compilation) -> napi::Result<R>,
  ) -> napi::Result<R> {
    let compiler_reference = COMPILER_REFERENCES.with(|ref_cell| {
      let references = ref_cell.borrow();
      references.get(&self.compiler_id).cloned()
    });

    let Some(this) = compiler_reference
      .as_ref()
      .and_then(|compiler_reference| compiler_reference.get())
    else {
      return Err(self.compiler_garbage_collected_error());
    };

    f(&this.compiler.compilation)
  }

  fn with_ref<R>(
    &self,
    f: impl FnOnce(&Compilation, &rspack_core::AsyncDependenciesBlock) -> napi::Result<R>,
  ) -> napi::Result<R> {
    self.with_compilation(|compilation| {
      let module_graph = compilation.get_module_graph();
      if let Some(block) = module_graph.block_by_id(&self.block_id) {
        f(compilation, block)
      } else {
        Err(self.block_removed_error())
      }
    })
  }
}

#[napi]
impl AsyncDependenciesBlock {
  #[napi(getter, ts_return_type = "Dependency[]")]
  pub fn dependencies(&self) -> napi::Result<Vec<DependencyWrapper>> {
    self.with_ref(|compilation, block| {
      let module_graph = compilation.get_module_graph();
      Ok(
        block
          .get_dependencies()
          .iter()
          .filter_map(|dependency_id| {
            internal::try_dependency_by_id(module_graph, dependency_id).map(|dep| {
              DependencyWrapper::new(
                (&**dep) as &dyn rspack_core::Dependency,
                compilation.id(),
                Some(compilation),
              )
            })
          })
          .collect::<Vec<_>>(),
      )
    })
  }

  #[napi(getter, ts_return_type = "AsyncDependenciesBlock[]")]
  pub fn blocks(&self) -> napi::Result<Vec<AsyncDependenciesBlockWrapper>> {
    self.with_ref(|compilation, block| {
      let module_graph = compilation.get_module_graph();
      Ok(
        block
          .get_blocks()
          .iter()
          .filter_map(|block_id| {
            module_graph
              .block_by_id(block_id)
              .map(|block| AsyncDependenciesBlockWrapper::new(block, compilation))
          })
          .collect::<Vec<_>>(),
      )
    })
  }
}

type BlockInstanceRefs = rspack_core::AsyncDependenciesBlockIdentifierMap<OneShotRef>;

type BlockInstanceRefsByCompilationId =
  RefCell<HashMap<rspack_core::CompilationId, BlockInstanceRefs>>;

thread_local! {
  static BLOCK_INSTANCE_REFS: BlockInstanceRefsByCompilationId = Default::default();
}

pub struct AsyncDependenciesBlockWrapper {
  block_id: rspack_core::AsyncDependenciesBlockIdentifier,
  compilation_id: rspack_core::CompilationId,
  compiler_id: CompilerId,
}

impl AsyncDependenciesBlockWrapper {
  pub fn new(
    block: &rspack_core::AsyncDependenciesBlock,
    compilation: &rspack_core::Compilation,
  ) -> Self {
    let block_id = block.identifier();

    Self {
      block_id,
      compilation_id: compilation.id(),
      compiler_id: compilation.compiler_id(),
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
        let mut refs_by_compilation_id = refs.borrow_mut();
        let entry = refs_by_compilation_id.entry(val.compilation_id);
        let refs = match entry {
          std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
          std::collections::hash_map::Entry::Vacant(entry) => {
            let refs = rspack_core::AsyncDependenciesBlockIdentifierMap::default();
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
              compiler_id: val.compiler_id,
            };
            let r = vacant_entry.insert(OneShotRef::new(env, js_block)?);
            ToNapiValue::to_napi_value(env, r)
          }
        }
      })
    }
  }
}
