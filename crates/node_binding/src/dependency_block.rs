use std::{cell::RefCell, ptr::NonNull};

use napi_derive::napi;
use rspack_core::{
  AsyncDependenciesBlock, AsyncDependenciesBlockIdentifier, Compilation, CompilationId,
  DependenciesBlock,
};
use rspack_napi::{napi::bindgen_prelude::*, OneShotRef};
use rustc_hash::FxHashMap as HashMap;

use crate::DependencyWrapper;

#[napi]
pub struct JsDependenciesBlock {
  pub(crate) block_id: AsyncDependenciesBlockIdentifier,
  compilation: NonNull<Compilation>,
}

#[napi]
impl JsDependenciesBlock {
  #[napi(getter, ts_return_type = "Dependency[]")]
  pub fn dependencies(&mut self) -> Vec<DependencyWrapper> {
    let compilation = unsafe { self.compilation.as_ref() };
    let module_graph = compilation.get_module_graph();
    if let Some(block) = module_graph.block_by_id(&self.block_id) {
      block
        .get_dependencies()
        .iter()
        .filter_map(|dependency_id| {
          module_graph
            .dependency_by_id(dependency_id)
            .map(|dep| DependencyWrapper::new(dep.as_ref(), compilation.id(), Some(compilation)))
        })
        .collect::<Vec<_>>()
    } else {
      vec![]
    }
  }

  #[napi(getter, ts_return_type = "JsDependenciesBlock[]")]
  pub fn blocks(&mut self) -> Vec<JsDependenciesBlockWrapper> {
    let compilation = unsafe { self.compilation.as_ref() };
    let module_graph = compilation.get_module_graph();
    if let Some(block) = module_graph.block_by_id(&self.block_id) {
      block
        .get_blocks()
        .iter()
        .filter_map(|block_id| {
          module_graph
            .block_by_id(block_id)
            .map(|block| JsDependenciesBlockWrapper::new(block, compilation))
        })
        .collect::<Vec<_>>()
    } else {
      vec![]
    }
  }
}

type BlockInstanceRefs = HashMap<AsyncDependenciesBlockIdentifier, OneShotRef>;

type BlockInstanceRefsByCompilationId = RefCell<HashMap<CompilationId, BlockInstanceRefs>>;

thread_local! {
  static BLOCK_INSTANCE_REFS: BlockInstanceRefsByCompilationId = Default::default();
}

pub struct JsDependenciesBlockWrapper {
  block_id: AsyncDependenciesBlockIdentifier,
  compilation: NonNull<Compilation>,
}

impl JsDependenciesBlockWrapper {
  pub fn new(block: &AsyncDependenciesBlock, compilation: &Compilation) -> Self {
    let block_id = block.identifier();

    #[allow(clippy::unwrap_used)]
    Self {
      block_id,
      compilation: NonNull::new(compilation as *const Compilation as *mut Compilation).unwrap(),
    }
  }

  pub fn cleanup_last_compilation(compilation_id: CompilationId) {
    BLOCK_INSTANCE_REFS.with(|refs| {
      let mut refs_by_compilation_id = refs.borrow_mut();
      refs_by_compilation_id.remove(&compilation_id)
    });
  }
}

impl ToNapiValue for JsDependenciesBlockWrapper {
  unsafe fn to_napi_value(
    env: napi::sys::napi_env,
    val: Self,
  ) -> napi::Result<napi::sys::napi_value> {
    BLOCK_INSTANCE_REFS.with(|refs| {
      let compilation = unsafe { val.compilation.as_ref() };
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
          let js_block = JsDependenciesBlock {
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
