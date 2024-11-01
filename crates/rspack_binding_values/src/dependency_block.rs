use std::{cell::RefCell, ptr::NonNull, sync::Arc};

use napi_derive::napi;
use rspack_collections::IdentifierMap;
use rspack_core::{
  AsyncDependenciesBlock, AsyncDependenciesBlockIdentifier, Compilation, CompilationId,
  DependenciesBlock, Module, ModuleGraph, ModuleIdentifier, RuntimeModuleStage, SourceType,
};
use rspack_napi::{napi::bindgen_prelude::*, threadsafe_function::ThreadsafeFunction, OneShotRef};
use rspack_plugin_runtime::RuntimeModuleFromJs;
use rspack_util::source_map::SourceMapKind;
use rustc_hash::FxHashMap as HashMap;

use super::{JsCompatSource, ToJsCompatSource};
use crate::{JsChunk, JsCodegenerationResults, JsCompilationWrapper, JsDependency, JsDependencyWrapper};

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
      block: NonNull::new(block as *const AsyncDependenciesBlock as *mut AsyncDependenciesBlock).unwrap(),
    }
  }

  fn as_ref(&mut self) -> napi::Result<&AsyncDependenciesBlock> {
    let block = unsafe { self.block.as_ref() };
    if block.identifier() == self.block_id {
      return Ok(block);
    }

    if let Some(compilation) = JsCompilationWrapper::compilation_by_id(&self.compilation_id) {
      let module_graph = compilation.get_module_graph();
      if let Some(block) = module_graph.block_by_id(&self.block_id) {
        self.block =
          NonNull::new(block as *const AsyncDependenciesBlock as *mut AsyncDependenciesBlock)
            .unwrap();
        return Ok(unsafe { self.block.as_ref() });
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
  pub fn dependencies(&self) -> Vec<JsDependencyWrapper> {
    let compilation = unsafe { self.compilation.as_ref() };

    let module_graph = compilation.get_module_graph();
    let block = self.block(&module_graph);
    block
      .get_dependencies()
      .iter()
      .filter_map(|dependency_id| {
        module_graph
          .dependency_by_id(dependency_id)
          .map(|dep| JsDependencyWrapper::new(dep, self.compilation))
      })
      .collect::<Vec<_>>()
  }

  #[napi(getter)]
  pub fn blocks(&self) -> Vec<JsDependenciesBlock> {
    let compilation = unsafe { self.compilation.as_ref() };

    let module_graph = compilation.get_module_graph();
    let block = self.block(&module_graph);
    let blocks = block.get_blocks();
    blocks
      .iter()
      .cloned()
      .map(|block_id| JsDependenciesBlock::new(block_id, self.compilation.as_ptr()))
      .collect::<Vec<_>>()
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
  block: NonNull<dyn DependenciesBlock>,
}

impl JsDependenciesBlockWrapper {
  pub fn new(block: &dyn DependenciesBlock, compilation_id: CompilationId) -> Self {
    let block_id = *block.id

    #[allow(clippy::unwrap_used)]
    Self {
      compilation_id,
      dependency_id,
      dependency: NonNull::new(dependency as *const dyn Dependency as *mut dyn Dependency).unwrap(),
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

      match refs.entry(val.dependency_id) {
        std::collections::hash_map::Entry::Occupied(occupied_entry) => {
          let r = occupied_entry.get();
          ToNapiValue::to_napi_value(env, r)
        }
        std::collections::hash_map::Entry::Vacant(vacant_entry) => {
          let instance: ClassInstance<JsDependency> = JsDependency {
            compilation_id: val.compilation_id,
            dependency_id: val.dependency_id,
            dependency: val.dependency,
          }
          .into_instance(Env::from_raw(env))?;
          let r = vacant_entry.insert(OneShotRef::new(env, instance)?);
          ToNapiValue::to_napi_value(env, r)
        }
      }
    })
  }
}
