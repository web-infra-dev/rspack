use std::cell::RefCell;

use napi::{
  Either, Env, JsString,
  bindgen_prelude::{Array, This, ToNapiValue},
};
use napi_derive::napi;
use rspack_core::{Compilation, CompilationId, OriginRecord};
use rspack_napi::{MemoizedJsArray, OneShotRef};
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
  chunks_jsobject: MemoizedJsArray<Vec<rspack_core::ChunkUkey>>,
  children_jsobject: MemoizedJsArray<Vec<rspack_core::ChunkGroupUkey>>,
  parents_jsobject: MemoizedJsArray<Vec<rspack_core::ChunkGroupUkey>>,
  origins_jsobject: MemoizedJsArray<Vec<ChunkGroupOriginSnapshot>>,
  files_jsobject: MemoizedJsArray<Vec<String>>,
}

struct ChunkGroupOriginSnapshot {
  origin: OriginRecord,
  module_exists: bool,
}

fn with_chunk_group<R>(
  compilation_id: CompilationId,
  chunk_group_ukey: rspack_core::ChunkGroupUkey,
  f: impl FnOnce(&Compilation, &rspack_core::ChunkGroup) -> napi::Result<R>,
) -> napi::Result<R> {
  with_compilation(compilation_id, |compilation| {
    if let Some(value) = compilation
      .build_chunk_graph_artifact
      .chunk_group_by_ukey
      .get(&chunk_group_ukey)
    {
      f(compilation, value)
    } else {
      Err(napi::Error::from_reason(format!(
        "Unable to access chunk_group with id = {:?} now. The chunk group has been removed on the Rust side.",
        chunk_group_ukey
      )))
    }
  })
}

impl ChunkGroup {
  fn with_ref<R>(
    &self,
    f: impl FnOnce(&Compilation, &rspack_core::ChunkGroup) -> napi::Result<R>,
  ) -> napi::Result<R> {
    with_chunk_group(self.compilation_id, self.chunk_group_ukey, f)
  }
}

#[napi]
impl ChunkGroup {
  #[napi(getter, ts_return_type = "ReadonlyArray<Chunk>")]
  pub fn chunks<'a>(&mut self, env: &'a Env, mut this: This<'a>) -> napi::Result<Array<'a>> {
    with_chunk_group(
      self.compilation_id,
      self.chunk_group_ukey,
      |compilation, chunk_group| {
        // Validate the owner before consulting any cached JS graph objects.
        let chunks = &chunk_group.chunks;
        self.chunks_jsobject.get_or_update(
          env,
          &mut this.object,
          "rspack.chunkGroup.chunks",
          |cached| cached == chunks,
          |array| {
            for (i, ukey) in chunks.iter().enumerate() {
              array.set(i as u32, ChunkWrapper::new(*ukey, compilation))?;
            }
            Ok((chunks.clone(), chunks.len() as u32))
          },
        )
      },
    )
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

  #[napi(getter, ts_return_type = "ReadonlyArray<Readonly<JsChunkGroupOrigin>>")]
  pub fn origins<'a>(&mut self, env: &'a Env, mut this: This<'a>) -> napi::Result<Array<'a>> {
    with_chunk_group(
      self.compilation_id,
      self.chunk_group_ukey,
      |compilation, chunk_group| {
        let origins = chunk_group.origins();
        self.origins_jsobject.get_or_update(
          env,
          &mut this.object,
          "rspack.chunkGroup.origins",
          |cached| {
            cached.len() == origins.len()
              && cached.iter().zip(origins).all(|(cached, origin)| {
                cached.origin.module == origin.module
                  && cached.origin.request == origin.request
                  && cached.origin.loc == origin.loc
                  // Module availability can change independently of the origin record.
                  && cached.module_exists
                    == origin
                      .module
                      .is_some_and(|id| compilation.module_by_identifier(&id).is_some())
              })
          },
          |array| {
            let mut snapshot = Vec::with_capacity(origins.len());
            for (i, origin) in origins.iter().enumerate() {
              let loc = match &origin.loc {
                Some(rspack_core::DependencyLocation::Real(real)) => Some(Either::B(real.into())),
                Some(rspack_core::DependencyLocation::Synthetic(synthetic)) => {
                  Some(Either::A(env.create_string(&synthetic.name)?))
                }
                None => None,
              };
              let module = origin
                .module
                .and_then(|id| compilation.module_by_identifier(&id));
              array.set(
                i as u32,
                JsChunkGroupOrigin {
                  module: module.map(|module| {
                    ModuleObject::with_ref(module.as_ref(), compilation.compiler_id())
                  }),
                  request: origin
                    .request
                    .as_ref()
                    .map(|request| env.create_string(request))
                    .transpose()?,
                  loc,
                },
              )?;
              snapshot.push(ChunkGroupOriginSnapshot {
                origin: origin.clone(),
                module_exists: module.is_some(),
              });
            }
            Ok((snapshot, origins.len() as u32))
          },
        )
      },
    )
  }

  #[napi(getter, ts_return_type = "ReadonlyArray<ChunkGroup>")]
  pub fn children_iterable<'a>(
    &mut self,
    env: &'a Env,
    mut this: This<'a>,
  ) -> napi::Result<Array<'a>> {
    with_chunk_group(
      self.compilation_id,
      self.chunk_group_ukey,
      |compilation, chunk_group| {
        self.children_jsobject.get_or_update(
          env,
          &mut this.object,
          "rspack.chunkGroup.childrenIterable",
          |cached| cached.iter().eq(chunk_group.children_iterable()),
          |array| {
            let children = chunk_group.children_iterable().copied().collect::<Vec<_>>();
            for (i, ukey) in children.iter().enumerate() {
              array.set(i as u32, ChunkGroupWrapper::new(*ukey, compilation))?;
            }
            let len = children.len() as u32;
            Ok((children, len))
          },
        )
      },
    )
  }

  #[napi]
  pub fn is_initial(&self) -> napi::Result<bool> {
    self.with_ref(|_, chunk_group| Ok(chunk_group.is_initial()))
  }

  #[napi(ts_return_type = "ReadonlyArray<ChunkGroup>")]
  pub fn get_parents<'a>(&mut self, env: &'a Env, mut this: This<'a>) -> napi::Result<Array<'a>> {
    with_chunk_group(
      self.compilation_id,
      self.chunk_group_ukey,
      |compilation, chunk_group| {
        self.parents_jsobject.get_or_update(
          env,
          &mut this.object,
          "rspack.chunkGroup.parents",
          // Preserve iteration order as well as membership in the snapshot.
          |cached| cached.iter().eq(chunk_group.parents.iter()),
          |array| {
            let parents = chunk_group.parents.iter().copied().collect::<Vec<_>>();
            for (i, ukey) in parents.iter().enumerate() {
              array.set(i as u32, ChunkGroupWrapper::new(*ukey, compilation))?;
            }
            let len = parents.len() as u32;
            Ok((parents, len))
          },
        )
      },
    )
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

  #[napi(ts_return_type = "ReadonlyArray<string>")]
  pub fn get_files<'a>(&mut self, env: &'a Env, mut this: This<'a>) -> napi::Result<Array<'a>> {
    with_chunk_group(
      self.compilation_id,
      self.chunk_group_ukey,
      |compilation, chunk_group| {
        let files = || {
          chunk_group
            .chunks
            .iter()
            .filter_map(|ukey| {
              compilation
                .build_chunk_graph_artifact
                .chunk_by_ukey
                .get(ukey)
            })
            .flat_map(|chunk| chunk.files().iter())
        };
        self.files_jsobject.get_or_update(
          env,
          &mut this.object,
          "rspack.chunkGroup.files",
          |cached| cached.iter().eq(files()),
          |array| {
            let mut snapshot = Vec::new();
            for (i, file) in files().enumerate() {
              array.set(i as u32, file.as_str())?;
              snapshot.push(file.clone());
            }
            let len = snapshot.len() as u32;
            Ok((snapshot, len))
          },
        )
      },
    )
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
              chunks_jsobject: Default::default(),
              children_jsobject: Default::default(),
              parents_jsobject: Default::default(),
              origins_jsobject: Default::default(),
              files_jsobject: Default::default(),
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
