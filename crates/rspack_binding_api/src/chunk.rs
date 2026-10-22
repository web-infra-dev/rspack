use std::cell::RefCell;

use napi::{
  Either, Env, JsString,
  bindgen_prelude::{
    Array, Either3, FromNapiValue, Object, This, ToNapiValue, TypeName, ValidateNapiValue,
  },
  sys,
};
use napi_derive::napi;
use rspack_core::{
  ChunkGroupUkey, Compilation, CompilationId, RuntimeSpec, chunk_graph_chunk::ChunkId,
};
use rspack_napi::{MemoizedJsArray, OneShotRef};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
  chunk_group::ChunkGroupWrapper, compilation::entries::EntryOptionsDTO, with_compilation,
};

#[derive(Default)]
struct GroupsCacheKey {
  membership: FxHashSet<ChunkGroupUkey>,
  // Outer None means absent; inner None means a group without an index.
  indices: Vec<(ChunkGroupUkey, Option<Option<u32>>)>,
}

#[napi]
pub struct Chunk {
  pub(crate) chunk_ukey: rspack_core::ChunkUkey,
  compilation_id: CompilationId,
  files_jsobject: MemoizedJsArray<FxHashSet<String>>,
  runtime_jsobject: MemoizedJsArray<RuntimeSpec>,
  auxiliary_files_jsobject: MemoizedJsArray<FxHashSet<String>>,
  groups_jsobject: MemoizedJsArray<GroupsCacheKey>,
}

fn with_chunk<R>(
  compilation_id: CompilationId,
  chunk_ukey: rspack_core::ChunkUkey,
  f: impl FnOnce(&Compilation, &rspack_core::Chunk) -> napi::Result<R>,
) -> napi::Result<R> {
  with_compilation(compilation_id, |compilation| {
    if let Some(value) = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .get(&chunk_ukey)
    {
      f(compilation, value)
    } else {
      Err(napi::Error::from_reason(format!(
        "Unable to access chunk with id = {:?} now. The chunk have been removed on the Rust side.",
        chunk_ukey
      )))
    }
  })
}

impl Chunk {
  fn with_ref<R>(
    &self,
    f: impl FnOnce(&Compilation, &rspack_core::Chunk) -> napi::Result<R>,
  ) -> napi::Result<R> {
    with_chunk(self.compilation_id, self.chunk_ukey, f)
  }
}

#[napi]
impl Chunk {
  #[napi(getter)]
  pub fn name<'a>(&self, env: &'a Env) -> napi::Result<Either<JsString<'a>, ()>> {
    self.with_ref(|_, chunk| {
      Ok(match chunk.name() {
        Some(name) => Either::A(env.create_string(name)?),
        None => Either::B(()),
      })
    })
  }

  #[napi(getter, ts_return_type = "string | number | undefined")]
  pub fn id<'a>(&self, env: &'a Env) -> napi::Result<Either3<JsString<'a>, u32, ()>> {
    self.with_ref(|_, chunk| {
      Ok(match chunk.id() {
        Some(id) => match id.as_number() {
          Some(id) => Either3::B(id),
          None => Either3::A(env.create_string(id.as_str())?),
        },
        None => Either3::C(()),
      })
    })
  }

  #[napi(getter, ts_return_type = "Array<string | number>")]
  pub fn ids<'a>(&self, env: &'a Env) -> napi::Result<Array<'a>> {
    self.with_ref(|_, chunk| {
      let Some(id) = chunk.id() else {
        return env.create_array(0);
      };
      let mut array = env.create_array(1)?;
      match id.as_number() {
        Some(id) => array.set(0, id)?,
        None => array.set(0, id.as_str())?,
      };
      Ok(array)
    })
  }

  #[napi(getter, ts_return_type = "Array<string>")]
  pub fn id_name_hints<'a>(&self, env: &'a Env) -> napi::Result<Array<'a>> {
    self.with_ref(|_, chunk| {
      let id_name_hints = chunk.id_name_hints();
      let mut array = env.create_array(id_name_hints.len() as u32)?;
      for (i, hint) in id_name_hints.iter().enumerate() {
        array.set(i as u32, hint.as_str())?;
      }
      Ok(array)
    })
  }

  #[napi(getter)]
  pub fn filename_template<'a>(&self, env: &'a Env) -> napi::Result<Either<JsString<'a>, ()>> {
    self.with_ref(|_, chunk| {
      Ok(match chunk.filename_template().and_then(|f| f.template()) {
        Some(tpl) => Either::A(env.create_string(tpl)?),
        None => Either::B(()),
      })
    })
  }

  #[napi(getter)]
  pub fn css_filename_template<'a>(&self, env: &'a Env) -> napi::Result<Either<JsString<'a>, ()>> {
    self.with_ref(|_, chunk| {
      Ok(
        match chunk.css_filename_template().and_then(|f| f.template()) {
          Some(tpl) => Either::A(env.create_string(tpl)?),
          None => Either::B(()),
        },
      )
    })
  }

  #[napi(getter, js_name = "_files", ts_return_type = "Array<string>")]
  pub fn files<'a>(&mut self, env: &'a Env, mut this: This<'a>) -> napi::Result<Array<'a>> {
    with_chunk(self.compilation_id, self.chunk_ukey, |_, chunk| {
      // Resolve the compilation and chunk before consulting the cache, so a
      // retained wrapper cannot bypass native-target revocation.
      let files = chunk.files();
      self.files_jsobject.get_or_update(
        env,
        &mut this.object,
        "rspack.chunk.files",
        |cached| cached == files,
        |array| {
          let mut sorted_files = files.iter().collect::<Vec<_>>();
          sorted_files.sort_unstable();
          for (i, file) in sorted_files.iter().enumerate() {
            array.set(i as u32, file.as_str())?;
          }
          Ok((files.clone(), files.len() as u32))
        },
      )
    })
  }

  #[napi(getter, js_name = "_runtime", ts_return_type = "Array<string>")]
  pub fn runtime<'a>(&mut self, env: &'a Env, mut this: This<'a>) -> napi::Result<Array<'a>> {
    with_chunk(self.compilation_id, self.chunk_ukey, |_, chunk| {
      let runtime = chunk.runtime();
      self.runtime_jsobject.get_or_update(
        env,
        &mut this.object,
        "rspack.chunk.runtime",
        // Compare members rather than RuntimeSpec's derived string key.
        |cached| cached.len() == runtime.len() && cached.iter().all(|item| runtime.contains(item)),
        |array| {
          for (i, runtime) in runtime.iter().enumerate() {
            array.set(i as u32, runtime.as_str())?;
          }
          Ok((runtime.clone(), runtime.len() as u32))
        },
      )
    })
  }

  #[napi(getter)]
  pub fn hash<'a>(&self, env: &'a Env) -> napi::Result<Either<JsString<'a>, ()>> {
    self.with_ref(|compilation, chunk| {
      Ok(
        match chunk
          .hash(&compilation.chunk_hashes_artifact)
          .map(|d| d.encoded())
        {
          Some(hash) => Either::A(env.create_string(hash)?),
          None => Either::B(()),
        },
      )
    })
  }

  #[napi(getter, ts_return_type = "Record<string, string>")]
  pub fn content_hash(&self, env: &Env) -> napi::Result<Object<'_>> {
    self.with_ref(|compilation, chunk| {
      let mut object = Object::new(env)?;
      if let Some(content_hash) = chunk.content_hash(&compilation.chunk_hashes_artifact) {
        for (key, value) in content_hash.iter() {
          object.set(key.to_string(), value.encoded())?;
        }
      }
      Ok(object)
    })
  }

  #[napi(getter)]
  pub fn rendered_hash<'a>(&self, env: &'a Env) -> napi::Result<Either<JsString<'a>, ()>> {
    self.with_ref(|compilation, chunk| {
      Ok(
        match chunk.rendered_hash(
          &compilation.chunk_hashes_artifact,
          compilation.options.output.hash_digest_length,
        ) {
          Some(hash) => Either::A(env.create_string(hash)?),
          None => Either::B(()),
        },
      )
    })
  }

  #[napi(getter)]
  pub fn chunk_reason<'a>(&self, env: &'a Env) -> napi::Result<Either<JsString<'a>, ()>> {
    self.with_ref(|_, chunk| {
      Ok(match chunk.chunk_reason() {
        Some(reason) => Either::A(env.create_string(reason)?),
        None => Either::B(()),
      })
    })
  }

  #[napi(getter, js_name = "_auxiliaryFiles", ts_return_type = "Array<string>")]
  pub fn auxiliary_files<'a>(
    &mut self,
    env: &'a Env,
    mut this: This<'a>,
  ) -> napi::Result<Array<'a>> {
    with_chunk(self.compilation_id, self.chunk_ukey, |_, chunk| {
      let files = chunk.auxiliary_files();
      self.auxiliary_files_jsobject.get_or_update(
        env,
        &mut this.object,
        "rspack.chunk.auxiliaryFiles",
        |cached| cached == files,
        |array| {
          for (i, file) in files.iter().enumerate() {
            array.set(i as u32, file.as_str())?;
          }
          Ok((files.clone(), files.len() as u32))
        },
      )
    })
  }
}

#[napi]
impl Chunk {
  #[napi]
  pub fn is_only_initial(&self) -> napi::Result<bool> {
    self.with_ref(|compilation, chunk| {
      Ok(chunk.is_only_initial(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey))
    })
  }

  #[napi]
  pub fn can_be_initial(&self) -> napi::Result<bool> {
    self.with_ref(|compilation, chunk| {
      Ok(chunk.can_be_initial(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey))
    })
  }

  #[napi]
  pub fn has_runtime(&self) -> napi::Result<bool> {
    self.with_ref(|compilation, chunk| {
      Ok(chunk.has_runtime(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey))
    })
  }

  #[napi(ts_return_type = "Chunk[]")]
  pub fn get_all_async_chunks(&self) -> napi::Result<Vec<ChunkWrapper>> {
    self.with_ref(|compilation, chunk| {
      Ok(
        chunk
          .get_all_async_chunks(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey)
          .into_iter()
          .map(|chunk_ukey| ChunkWrapper::new(chunk_ukey, compilation))
          .collect::<Vec<_>>(),
      )
    })
  }

  #[napi(ts_return_type = "Chunk[]")]
  pub fn get_all_initial_chunks(&self) -> napi::Result<Vec<ChunkWrapper>> {
    self.with_ref(|compilation, chunk| {
      Ok(
        chunk
          .get_all_initial_chunks(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey)
          .into_iter()
          .map(|chunk_ukey| ChunkWrapper::new(chunk_ukey, compilation))
          .collect::<Vec<_>>(),
      )
    })
  }

  #[napi(ts_return_type = "Chunk[]")]
  pub fn get_all_referenced_chunks(&self) -> napi::Result<Vec<ChunkWrapper>> {
    self.with_ref(|compilation, chunk| {
      Ok(
        chunk
          .get_all_referenced_chunks(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey)
          .into_iter()
          .map(|chunk_ukey| ChunkWrapper::new(chunk_ukey, compilation))
          .collect::<Vec<_>>(),
      )
    })
  }

  #[napi(getter, js_name = "_groupsIterable", ts_return_type = "ChunkGroup[]")]
  pub fn groups_iterable<'a>(
    &mut self,
    env: &'a Env,
    mut this: This<'a>,
  ) -> napi::Result<Array<'a>> {
    with_chunk(
      self.compilation_id,
      self.chunk_ukey,
      |compilation, chunk| {
        let membership = chunk.groups();
        let by_ukey = &compilation.build_chunk_graph_artifact.chunk_group_by_ukey;
        self.groups_jsobject.get_or_update(
          env,
          &mut this.object,
          "rspack.chunk.groupsIterable",
          |cached| {
            &cached.membership == membership
              && cached
                .indices
                .iter()
                .all(|(ukey, index)| by_ukey.get(ukey).map(|group| group.index) == *index)
          },
          |array| {
            // Membership alone does not track changes to group ordering or removal
            // from the graph. Retain their existence/index alongside the membership
            // snapshot.
            let indices = membership
              .iter()
              .map(|ukey| (*ukey, by_ukey.get(ukey).map(|group| group.index)))
              .collect();
            let mut groups = membership
              .iter()
              .filter_map(|ukey| by_ukey.get(ukey))
              .collect::<Vec<_>>();
            groups.sort_unstable_by_key(|group| group.index);
            for (i, group) in groups.iter().enumerate() {
              array.set(i as u32, ChunkGroupWrapper::new(group.ukey, compilation))?;
            }
            Ok((
              GroupsCacheKey {
                membership: membership.clone(),
                indices,
              },
              groups.len() as u32,
            ))
          },
        )
      },
    )
  }

  #[napi(ts_return_type = "EntryOptionsDTO | undefined")]
  pub fn get_entry_options(&self) -> napi::Result<Option<EntryOptionsDTO>> {
    self.with_ref(|compilation, chunk| {
      let entry_options =
        chunk.get_entry_options(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey);

      Ok(entry_options.map(|options| EntryOptionsDTO::new(options.clone())))
    })
  }
}

thread_local! {
  static CHUNK_INSTANCE_REFS: RefCell<FxHashMap<CompilationId, FxHashMap<rspack_core::ChunkUkey, OneShotRef>>> = Default::default();
}

pub struct ChunkWrapper {
  pub chunk_ukey: rspack_core::ChunkUkey,
  pub compilation_id: CompilationId,
}

impl FromNapiValue for ChunkWrapper {
  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> napi::Result<Self> {
    let chunk: &Chunk = unsafe { FromNapiValue::from_napi_value(env, napi_val)? };
    Ok(Self {
      chunk_ukey: chunk.chunk_ukey,
      compilation_id: chunk.compilation_id,
    })
  }
}

impl ChunkWrapper {
  pub fn new(chunk_ukey: rspack_core::ChunkUkey, compilation: &Compilation) -> Self {
    Self {
      chunk_ukey,
      compilation_id: compilation.id(),
    }
  }

  pub fn new_by_id(chunk_ukey: rspack_core::ChunkUkey, compilation_id: CompilationId) -> Self {
    Self {
      chunk_ukey,
      compilation_id,
    }
  }

  pub fn cleanup_last_compilation(compilation_id: CompilationId) {
    CHUNK_INSTANCE_REFS.with(|refs| {
      let mut refs_by_compilation_id = refs.borrow_mut();
      refs_by_compilation_id.remove(&compilation_id)
    });
  }
}

impl TypeName for ChunkWrapper {
  fn type_name() -> &'static str {
    "Chunk"
  }

  fn value_type() -> napi::ValueType {
    napi::ValueType::Object
  }
}

impl ValidateNapiValue for ChunkWrapper {
  unsafe fn validate(
    env: sys::napi_env,
    napi_val: sys::napi_value,
  ) -> napi::Result<sys::napi_value> {
    unsafe { <&Chunk as ValidateNapiValue>::validate(env, napi_val) }
  }
}

impl ToNapiValue for ChunkWrapper {
  unsafe fn to_napi_value(
    env: napi::sys::napi_env,
    val: Self,
  ) -> napi::Result<napi::sys::napi_value> {
    unsafe {
      CHUNK_INSTANCE_REFS.with(|refs| {
        let mut refs_by_compilation_id = refs.borrow_mut();
        let entry = refs_by_compilation_id.entry(val.compilation_id);
        let refs = match entry {
          std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
          std::collections::hash_map::Entry::Vacant(entry) => {
            let refs = FxHashMap::default();
            entry.insert(refs)
          }
        };

        match refs.entry(val.chunk_ukey) {
          std::collections::hash_map::Entry::Occupied(entry) => {
            let r = entry.get();
            ToNapiValue::to_napi_value(env, r)
          }
          std::collections::hash_map::Entry::Vacant(entry) => {
            let js_chunk = Chunk {
              chunk_ukey: val.chunk_ukey,
              compilation_id: val.compilation_id,
              files_jsobject: Default::default(),
              runtime_jsobject: Default::default(),
              auxiliary_files_jsobject: Default::default(),
              groups_jsobject: Default::default(),
            };
            let r = entry.insert(OneShotRef::new(env, js_chunk)?);
            ToNapiValue::to_napi_value(env, r)
          }
        }
      })
    }
  }
}

#[napi(object, object_from_js = false)]
pub struct JsChunkAssetArgs {
  #[napi(ts_type = "Chunk")]
  pub chunk: ChunkWrapper,
  pub filename: String,
}
