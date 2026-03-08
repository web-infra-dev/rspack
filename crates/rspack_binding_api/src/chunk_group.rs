use std::{cell::RefCell, ptr::NonNull};

use napi::{Either, Env, JsString, bindgen_prelude::ToNapiValue};
use napi_derive::napi;
use rspack_collections::UkeyMap;
use rspack_core::{Compilation, CompilationId};
use rspack_napi::OneShotRef;

use crate::{
  chunk::ChunkWrapper,
  location::RealDependencyLocation,
  module::{ModuleObject, ModuleObjectRef},
};

#[napi]
pub struct ChunkGroup {
  pub(crate) chunk_group_ukey: rspack_core::ChunkGroupUkey,
  compilation: NonNull<Compilation>,
}

impl ChunkGroup {
  fn as_ref(&self) -> napi::Result<(&'static Compilation, &'static rspack_core::ChunkGroup)> {
    let compilation = unsafe { self.compilation.as_ref() };
    if let Some(chunk_group) = compilation
      .build_chunk_graph_artifact
      .chunk_group_by_ukey
      .get(&self.chunk_group_ukey)
    {
      Ok((compilation, chunk_group))
    } else {
      Err(napi::Error::from_reason(format!(
        "Unable to access chunk_group with id = {:?} now. The module have been removed on the Rust side.",
        self.chunk_group_ukey
      )))
    }
  }
}

#[napi]
impl ChunkGroup {
  #[napi(getter, ts_return_type = "Chunk[]")]
  pub fn chunks(&self) -> napi::Result<Vec<ChunkWrapper>> {
    let (compilation, chunk_graph) = self.as_ref()?;
    Ok(
      chunk_graph
        .chunks
        .iter()
        .map(|ukey| ChunkWrapper::new(*ukey, compilation))
        .collect::<Vec<_>>(),
    )
  }

  #[napi(getter)]
  pub fn index(&self) -> napi::Result<Either<u32, ()>> {
    let (_, chunk_graph) = self.as_ref()?;
    Ok(match chunk_graph.index {
      Some(index) => Either::A(index),
      None => Either::B(()),
    })
  }

  #[napi(getter)]
  pub fn name(&self) -> napi::Result<Either<&str, ()>> {
    let (_, chunk_graph) = self.as_ref()?;
    Ok(match chunk_graph.name() {
      Some(name) => Either::A(name),
      None => Either::B(()),
    })
  }

  #[napi(getter)]
  pub fn origins<'a>(&self, env: &'a Env) -> napi::Result<Vec<JsChunkGroupOrigin<'a>>> {
    let (compilation, chunk_graph) = self.as_ref()?;
    let origins = chunk_graph.origins();
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
  }

  #[napi(getter, ts_return_type = "ChunkGroup[]")]
  pub fn children_iterable(&self) -> napi::Result<Vec<ChunkGroupWrapper>> {
    let (compilation, chunk_graph) = self.as_ref()?;
    Ok(
      chunk_graph
        .children_iterable()
        .map(|ukey| ChunkGroupWrapper::new(*ukey, compilation))
        .collect::<Vec<_>>(),
    )
  }

  #[napi]
  pub fn is_initial(&self) -> napi::Result<bool> {
    let (_, chunk_group) = self.as_ref()?;
    Ok(chunk_group.is_initial())
  }

  #[napi(ts_return_type = "ChunkGroup[]")]
  pub fn get_parents(&self) -> napi::Result<Vec<ChunkGroupWrapper>> {
    let (compilation, chunk_group) = self.as_ref()?;
    Ok(
      chunk_group
        .parents
        .iter()
        .map(|ukey| ChunkGroupWrapper::new(*ukey, compilation))
        .collect(),
    )
  }

  #[napi(ts_return_type = "Chunk")]
  pub fn get_runtime_chunk(&self) -> napi::Result<ChunkWrapper> {
    let (compilation, chunk_group) = self.as_ref()?;
    let chunk_ukey =
      chunk_group.get_runtime_chunk(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey);
    Ok(ChunkWrapper::new(chunk_ukey, compilation))
  }

  #[napi(ts_return_type = "Chunk")]
  pub fn get_entrypoint_chunk(&self) -> napi::Result<ChunkWrapper> {
    let (compilation, chunk_group) = self.as_ref()?;
    let chunk_ukey = chunk_group.get_entrypoint_chunk();
    Ok(ChunkWrapper::new(chunk_ukey, compilation))
  }

  #[napi]
  pub fn get_files(&self) -> napi::Result<Vec<&String>> {
    let (compilation, chunk_group) = self.as_ref()?;
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
        .collect::<Vec<_>>(),
    )
  }

  #[napi(ts_args_type = "module: Module")]
  pub fn get_module_pre_order_index(&self, module: ModuleObjectRef) -> napi::Result<Option<u32>> {
    let (_, chunk_group) = self.as_ref()?;
    Ok(
      chunk_group
        .module_pre_order_index(&module.identifier)
        .map(|v| v as u32),
    )
  }

  #[napi(ts_args_type = "module: Module")]
  pub fn get_module_post_order_index(&self, module: ModuleObjectRef) -> napi::Result<Option<u32>> {
    let (_, chunk_group) = self.as_ref()?;

    Ok(
      chunk_group
        .module_post_order_index(&module.identifier)
        .map(|v| v as u32),
    )
  }
}

thread_local! {
  static CHUNK_GROUP_INSTANCE_REFS: RefCell<UkeyMap<CompilationId, UkeyMap<rspack_core::ChunkGroupUkey, OneShotRef>>> = Default::default();
}

pub struct ChunkGroupWrapper {
  chunk_group_ukey: rspack_core::ChunkGroupUkey,
  compilation_id: CompilationId,
  compilation: NonNull<Compilation>,
}

impl ChunkGroupWrapper {
  pub fn new(chunk_group_ukey: rspack_core::ChunkGroupUkey, compilation: &Compilation) -> Self {
    #[allow(clippy::unwrap_used)]
    Self {
      chunk_group_ukey,
      compilation_id: compilation.id(),
      compilation: NonNull::new(compilation as *const Compilation as *mut Compilation).unwrap(),
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
            let refs = UkeyMap::default();
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
              compilation: val.compilation,
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
