use std::{cell::RefCell, ptr::NonNull};

use napi_derive::napi;
use rspack_core::{
  AsyncDependenciesBlock, AsyncDependenciesBlockIdentifier, CompilationId, DependenciesBlock,
  ModuleGraph,
};
use rspack_napi::{napi::bindgen_prelude::*, OneShotRef};
use rustc_hash::FxHashMap as HashMap;

use crate::{JsCompilationWrapper, JsDependencyWrapper};

#[derive(Default)]
#[napi(object)]
pub struct JsFactoryMeta {
  pub side_effect_free: Option<bool>,
}

#[napi]
pub struct JsDependenciesBlock {
  compilation_id: CompilationId,
  block_id: AsyncDependenciesBlockIdentifier,
  block: NonNull<AsyncDependenciesBlock>,
}

impl JsDependenciesBlock {
  pub fn new(block: &AsyncDependenciesBlock, compilation_id: CompilationId) -> Self {
    #[allow(clippy::unwrap_used)]
    Self {
      compilation_id,
      block_id: block.identifier(),
      block: NonNull::new(block as *const AsyncDependenciesBlock as *mut AsyncDependenciesBlock)
        .unwrap(),
    }
  }

  fn as_ref(&mut self) -> napi::Result<(&AsyncDependenciesBlock, ModuleGraph)> {
    if let Some(compilation) = JsCompilationWrapper::compilation_by_id(&self.compilation_id) {
      let module_graph = compilation.get_module_graph();

      let block = unsafe { self.block.as_ref() };
      if block.identifier() == self.block_id {
        return Ok((block, module_graph));
      }

      if let Some(block) = module_graph.block_by_id(&self.block_id) {
        self.block =
          NonNull::new(block as *const AsyncDependenciesBlock as *mut AsyncDependenciesBlock)
            .unwrap();
        return Ok((unsafe { self.block.as_ref() }, module_graph));
      }
    }

    Err(napi::Error::from_reason(format!(
      "Unable to access block with id = {:?} now. The block have been removed on the Rust side.",
      self.block_id
    )))
  }

  fn as_mut(&mut self) -> napi::Result<&mut AsyncDependenciesBlock> {
    let block = unsafe { self.block.as_mut() };
    if block.identifier() == self.block_id {
      return Ok(block);
    }

    Err(napi::Error::from_reason(format!(
      "Unable to access block with id = {:?} now. The block have been removed on the Rust side.",
      self.block_id
    )))
  }
}

#[napi]
impl JsDependenciesBlock {
  #[napi(getter, ts_return_type = "JsDependency")]
  pub fn dependencies(&mut self) -> Result<Vec<JsDependencyWrapper>> {
    let compilation_id = self.compilation_id;
    let (block, module_graph) = self.as_ref()?;

    Ok(
      block
        .get_dependencies()
        .iter()
        .filter_map(|dependency_id| {
          module_graph
            .dependency_by_id(dependency_id)
            .map(|dep| JsDependencyWrapper::new(dep.as_ref(), compilation_id))
        })
        .collect::<Vec<_>>(),
    )
  }

  #[napi(getter)]
  pub fn blocks(&mut self) -> Result<Vec<JsDependenciesBlock>> {
    let compilation_id = self.compilation_id;
    let (block, module_graph) = self.as_ref()?;

    Ok(
      block
        .get_blocks()
        .iter()
        .filter_map(|block_id| {
          module_graph
            .block_by_id(block_id)
            .map(|block| JsDependenciesBlock::new(block, compilation_id))
        })
        .collect::<Vec<_>>(),
    )
  }
}

type BlockInstanceRefs =
  HashMap<AsyncDependenciesBlockIdentifier, OneShotRef<ClassInstance<JsDependenciesBlock>>>;

type BlockInstanceRefsByCompilationId = RefCell<HashMap<CompilationId, BlockInstanceRefs>>;

thread_local! {
  static BLOCK_INSTANCE_REFS: BlockInstanceRefsByCompilationId = Default::default();
}

pub struct JsDependenciesBlockWrapper {
  compilation_id: CompilationId,
  block_id: AsyncDependenciesBlockIdentifier,
  block: NonNull<AsyncDependenciesBlock>,
}

impl JsDependenciesBlockWrapper {
  pub fn new(block: &AsyncDependenciesBlock, compilation_id: CompilationId) -> Self {
    let block_id = block.identifier();

    #[allow(clippy::unwrap_used)]
    Self {
      compilation_id,
      block_id,
      block: NonNull::new(block as *const AsyncDependenciesBlock as *mut AsyncDependenciesBlock)
        .unwrap(),
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
      let mut refs_by_compilation_id = refs.borrow_mut();
      let entry = refs_by_compilation_id.entry(val.compilation_id);
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
          let instance: ClassInstance<JsDependenciesBlock> = JsDependenciesBlock {
            compilation_id: val.compilation_id,
            block_id: val.block_id,
            block: val.block,
          }
          .into_instance(Env::from_raw(env))?;
          let r = vacant_entry.insert(OneShotRef::new(env, instance)?);
          ToNapiValue::to_napi_value(env, r)
        }
      }
    })
  }
}
