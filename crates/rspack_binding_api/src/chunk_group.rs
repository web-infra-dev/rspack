use std::cell::RefCell;

use napi::{Either, Env, JsString, bindgen_prelude::ToNapiValue};
use napi_derive::napi;
use rspack_core::{Compilation, CompilationId};
use rspack_napi::OneShotRef;
use rustc_hash::FxHashMap;

use crate::{
  chunk::ChunkWrapper,
  location::RealDependencyLocation,
  module::{ModuleObject, ModuleObjectRef},
  with_compilation,
};

#[napi]
pub struct ChunkGroup {
  chunk_group_ukey: rspack_core::ChunkGroupUkey,
  compilation_id: CompilationId,
}

impl ChunkGroup {
  fn with_ref<R>(
    &self,
    f: impl FnOnce(&Compilation, &rspack_core::ChunkGroup) -> napi::Result<R>,
  ) -> napi::Result<R> {
    with_compilation(self.compilation_id, |compilation| {
      if let Some(chunk_group) = compilation
        .build_chunk_graph_artifact
        .chunk_group_by_ukey
        .get(&self.chunk_group_ukey)
      {
        f(compilation, chunk_group)
      } else {
        Err(napi::Error::from_reason(format!(
          "Unable to access chunk_group with id = {:?} now. The chunk group has been removed on the Rust side.",
          self.chunk_group_ukey
        )))
      }
    })
  }
}

#[napi]
impl ChunkGroup {
  #[napi(getter, ts_return_type = "Chunk[]")]
  pub fn chunks(&self) -> napi::Result<Vec<ChunkWrapper>> {
    self.with_ref(|compilation, chunk_group| {
      Ok(
        chunk_group
          .chunks
          .iter()
          .map(|ukey| ChunkWrapper::new(*ukey, compilation))
          .collect::<Vec<_>>(),
      )
    })
  }

  #[napi(getter)]
  pub fn index(&self) -> napi::Result<Either<u32, ()>> {
    self.with_ref(|_, chunk_group| {
      Ok(match chunk_group.index {
        Some(index) => Either::A(index),
        None => Either::B(()),
      })
    })
  }

  #[napi(getter)]
  pub fn name(&self) -> napi::Result<Either<String, ()>> {
    self.with_ref(|_, chunk_group| {
      Ok(match chunk_group.name() {
        Some(name) => Either::A(name.to_string()),
        None => Either::B(()),
      })
    })
  }

  #[napi(getter)]
  pub fn origins<'a>(&self, env: &'a Env) -> napi::Result<Vec<JsChunkGroupOrigin<'a>>> {
    self.with_ref(|compilation, chunk_group| {
      let origins = chunk_group.origins();
      let mut js_origins = Vec::with_capacity(origins.len());

      for origin in origins {
        let loc = if let Some(loc) = &origin.loc {
          Some(match loc {
            rspack_core::DependencyLocation::Real(real) => Either::B(real.into()),
            rspack_core::DependencyLocation::Synthetic(synthetic) => {
              Either::A(env.create_string(&synthetic.name)?)
            }
          })
        } else {
          None
        };

        js_origins.push(JsChunkGroupOrigin {
          module: origin.module.and_then(|module_id| {
            compilation
              .module_by_identifier(&module_id)
              .map(|module| ModuleObject::with_ref(module.as_ref(), compilation.compiler_id()))
          }),
          request: match &origin.request {
            Some(request) => Some(env.create_string(request)?),
            None => None,
          },
          loc,
        })
      }

      Ok(js_origins)
    })
  }

  #[napi(getter, ts_return_type = "ChunkGroup[]")]
  pub fn children_iterable(&self) -> napi::Result<Vec<ChunkGroupWrapper>> {
    self.with_ref(|compilation, chunk_group| {
      Ok(
        chunk_group
          .children_iterable()
          .map(|ukey| ChunkGroupWrapper::new(*ukey, compilation))
          .collect::<Vec<_>>(),
      )
    })
  }

  #[napi]
  pub fn is_initial(&self) -> napi::Result<bool> {
    self.with_ref(|_, chunk_group| Ok(chunk_group.is_initial()))
  }

  #[napi(ts_return_type = "ChunkGroup[]")]
  pub fn get_parents(&self) -> napi::Result<Vec<ChunkGroupWrapper>> {
    self.with_ref(|compilation, chunk_group| {
      Ok(
        chunk_group
          .parents
          .iter()
          .map(|ukey| ChunkGroupWrapper::new(*ukey, compilation))
          .collect(),
      )
    })
  }

  #[napi(ts_return_type = "Chunk")]
  pub fn get_runtime_chunk(&self) -> napi::Result<ChunkWrapper> {
    self.with_ref(|compilation, chunk_group| {
      let chunk_ukey =
        chunk_group.get_runtime_chunk(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey);
      Ok(ChunkWrapper::new(chunk_ukey, compilation))
    })
  }

  #[napi(ts_return_type = "Chunk")]
  pub fn get_entrypoint_chunk(&self) -> napi::Result<ChunkWrapper> {
    self.with_ref(|compilation, chunk_group| {
      let chunk_ukey = chunk_group.get_entrypoint_chunk();
      Ok(ChunkWrapper::new(chunk_ukey, compilation))
    })
  }

  #[napi]
  pub fn get_files(&self) -> napi::Result<Vec<String>> {
    self.with_ref(|compilation, chunk_group| {
      Ok(
        chunk_group
          .chunks
          .iter()
          .filter_map(|chunk_ukey| {
            compilation
              .build_chunk_graph_artifact
              .chunk_by_ukey
              .get(chunk_ukey)
              .map(|chunk| chunk.files().iter())
          })
          .flatten()
          .cloned()
          .collect::<Vec<_>>(),
      )
    })
  }

  #[napi(ts_args_type = "module: Module")]
  pub fn get_module_pre_order_index(&self, module: ModuleObjectRef) -> napi::Result<Option<u32>> {
    self.with_ref(|_, chunk_group| Ok(chunk_group.module_pre_order_index(&module.identifier)))
  }

  #[napi(ts_args_type = "module: Module")]
  pub fn get_module_post_order_index(&self, module: ModuleObjectRef) -> napi::Result<Option<u32>> {
    self.with_ref(|_, chunk_group| Ok(chunk_group.module_post_order_index(&module.identifier)))
  }
}

thread_local! {
  static CHUNK_GROUP_INSTANCE_REFS: RefCell<FxHashMap<CompilationId, FxHashMap<rspack_core::ChunkGroupUkey, OneShotRef>>> = Default::default();
}

pub struct ChunkGroupWrapper {
  chunk_group_ukey: rspack_core::ChunkGroupUkey,
  compilation_id: CompilationId,
}

impl ChunkGroupWrapper {
  pub fn new(chunk_group_ukey: rspack_core::ChunkGroupUkey, compilation: &Compilation) -> Self {
    Self {
      chunk_group_ukey,
      compilation_id: compilation.id(),
    }
  }

  pub fn cleanup_last_compilation(compilation_id: CompilationId) {
    CHUNK_GROUP_INSTANCE_REFS.with(|refs| {
      let mut refs_by_compilation_id = refs.borrow_mut();
      refs_by_compilation_id.remove(&compilation_id)
    });
  }
}

impl ToNapiValue for ChunkGroupWrapper {
  unsafe fn to_napi_value(
    env: napi::sys::napi_env,
    val: Self,
  ) -> napi::Result<napi::sys::napi_value> {
    unsafe {
      CHUNK_GROUP_INSTANCE_REFS.with(|refs| {
        let mut refs_by_compilation_id = refs.borrow_mut();
        let entry = refs_by_compilation_id.entry(val.compilation_id);
        let refs = match entry {
          std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
          std::collections::hash_map::Entry::Vacant(entry) => {
            let refs = FxHashMap::default();
            entry.insert(refs)
          }
        };

        match refs.entry(val.chunk_group_ukey) {
          std::collections::hash_map::Entry::Occupied(entry) => {
            let r = entry.get();
            ToNapiValue::to_napi_value(env, r)
          }
          std::collections::hash_map::Entry::Vacant(entry) => {
            let js_module = ChunkGroup {
              chunk_group_ukey: val.chunk_group_ukey,
              compilation_id: val.compilation_id,
            };
            let r = entry.insert(OneShotRef::new(env, js_module)?);
            ToNapiValue::to_napi_value(env, r)
          }
        }
      })
    }
  }
}

#[napi(object, object_from_js = false)]
pub struct JsChunkGroupOrigin<'a> {
  #[napi(ts_type = "Module | undefined")]
  pub module: Option<ModuleObject>,
  pub request: Option<JsString<'a>>,
  pub loc: Option<Either<JsString<'a>, RealDependencyLocation>>,
}
