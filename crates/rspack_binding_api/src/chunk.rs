use std::{cell::RefCell, ptr::NonNull};

use napi::{
  Either, Env, JsString,
  bindgen_prelude::{Object, ToNapiValue},
};
use napi_derive::napi;
use rspack_collections::UkeyMap;
use rspack_core::{Compilation, CompilationId};
use rspack_napi::OneShotRef;

use crate::{chunk_group::ChunkGroupWrapper, compilation::entries::EntryOptionsDTO};

#[napi]
pub struct Chunk {
  pub(crate) chunk_ukey: rspack_core::ChunkUkey,
  compilation: NonNull<Compilation>,
}

impl Chunk {
  fn as_ref(&self) -> napi::Result<(&'static Compilation, &'static rspack_core::Chunk)> {
    let compilation = unsafe { self.compilation.as_ref() };
    if let Some(chunk) = compilation.chunk_by_ukey.get(&self.chunk_ukey) {
      Ok((compilation, chunk))
    } else {
      Err(napi::Error::from_reason(format!(
        "Unable to access chunk with id = {:?} now. The module have been removed on the Rust side.",
        self.chunk_ukey
      )))
    }
  }
}

#[napi]
impl Chunk {
  #[napi(getter)]
  pub fn name(&self) -> napi::Result<Either<&str, ()>> {
    let (_, chunk) = self.as_ref()?;
    Ok(match chunk.name() {
      Some(name) => Either::A(name),
      None => Either::B(()),
    })
  }

  #[napi(getter)]
  pub fn id(&self) -> napi::Result<Either<&str, ()>> {
    let (compilation, chunk) = self.as_ref()?;
    Ok(match chunk.id() {
      Some(id) => Either::A(id.as_str()),
      None => Either::B(()),
    })
  }

  #[napi(getter)]
  pub fn ids(&self) -> napi::Result<Vec<&str>> {
    let (compilation, chunk) = self.as_ref()?;
    Ok(chunk.id().map(|id| vec![id.as_str()]).unwrap_or_default())
  }

  #[napi(getter)]
  pub fn id_name_hints<'a>(&self, env: &'a Env) -> napi::Result<Vec<JsString<'a>>> {
    let (_, chunk) = self.as_ref()?;
    chunk
      .id_name_hints()
      .iter()
      .map(|s| env.create_string(s))
      .collect::<napi::Result<Vec<_>>>()
  }

  #[napi(getter)]
  pub fn filename_template(&self) -> napi::Result<Either<&str, ()>> {
    let (_, chunk) = self.as_ref()?;
    Ok(match chunk.filename_template().and_then(|f| f.template()) {
      Some(tpl) => Either::A(tpl),
      None => Either::B(()),
    })
  }

  #[napi(getter)]
  pub fn css_filename_template(&self) -> napi::Result<Either<&str, ()>> {
    let (_, chunk) = self.as_ref()?;
    Ok(
      match chunk.css_filename_template().and_then(|f| f.template()) {
        Some(tpl) => Either::A(tpl),
        None => Either::B(()),
      },
    )
  }

  #[napi(getter, js_name = "_files")]
  pub fn files(&self) -> napi::Result<Vec<&String>> {
    let (_, chunk) = self.as_ref()?;
    let mut files = Vec::from_iter(chunk.files());
    files.sort_unstable();
    Ok(files)
  }

  #[napi(getter, js_name = "_runtime")]
  pub fn runtime(&self) -> napi::Result<Vec<&str>> {
    let (_, chunk) = self.as_ref()?;
    Ok(chunk.runtime().iter().map(|r| r.as_ref()).collect())
  }

  #[napi(getter)]
  pub fn hash(&self) -> napi::Result<Either<&str, ()>> {
    let (compilation, chunk) = self.as_ref()?;
    Ok(
      match chunk
        .hash(&compilation.chunk_hashes_artifact)
        .map(|d| d.encoded())
      {
        Some(hash) => Either::A(hash),
        None => Either::B(()),
      },
    )
  }

  #[napi(getter, ts_return_type = "Record<string, string>")]
  pub fn content_hash(&self, env: &Env) -> napi::Result<Object<'_>> {
    let (compilation, chunk) = self.as_ref()?;

    let mut object = Object::new(env)?;
    if let Some(content_hash) = chunk.content_hash(&compilation.chunk_hashes_artifact) {
      for (key, value) in content_hash.iter() {
        object.set(key.to_string(), value.encoded())?;
      }
    }
    Ok(object)
  }

  #[napi(getter)]
  pub fn rendered_hash(&self) -> napi::Result<Either<&str, ()>> {
    let (compilation, chunk) = self.as_ref()?;
    Ok(
      match chunk.rendered_hash(
        &compilation.chunk_hashes_artifact,
        compilation.options.output.hash_digest_length,
      ) {
        Some(hash) => Either::A(hash),
        None => Either::B(()),
      },
    )
  }

  #[napi(getter)]
  pub fn chunk_reason(&self) -> napi::Result<Either<&str, ()>> {
    let (_, chunk) = self.as_ref()?;
    Ok(match chunk.chunk_reason() {
      Some(reason) => Either::A(reason),
      None => Either::B(()),
    })
  }

  #[napi(getter, js_name = "_auxiliaryFiles")]
  pub fn auxiliary_files(&self) -> napi::Result<Vec<&String>> {
    let (_, chunk) = self.as_ref()?;
    Ok(chunk.auxiliary_files().iter().collect::<Vec<_>>())
  }
}

#[napi]
impl Chunk {
  #[napi]
  pub fn is_only_initial(&self) -> napi::Result<bool> {
    let (compilation, chunk) = self.as_ref()?;
    Ok(chunk.is_only_initial(&compilation.chunk_group_by_ukey))
  }

  #[napi]
  pub fn can_be_initial(&self) -> napi::Result<bool> {
    let (compilation, chunk) = self.as_ref()?;
    Ok(chunk.can_be_initial(&compilation.chunk_group_by_ukey))
  }

  #[napi]
  pub fn has_runtime(&self) -> napi::Result<bool> {
    let (compilation, chunk) = self.as_ref()?;
    Ok(chunk.has_runtime(&compilation.chunk_group_by_ukey))
  }

  #[napi(ts_return_type = "Chunk[]")]
  pub fn get_all_async_chunks(&self) -> napi::Result<Vec<ChunkWrapper>> {
    let (compilation, chunk) = self.as_ref()?;
    Ok(
      chunk
        .get_all_async_chunks(&compilation.chunk_group_by_ukey)
        .into_iter()
        .map(|chunk_ukey| ChunkWrapper::new(chunk_ukey, compilation))
        .collect::<Vec<_>>(),
    )
  }

  #[napi(ts_return_type = "Chunk[]")]
  pub fn get_all_initial_chunks(&self) -> napi::Result<Vec<ChunkWrapper>> {
    let (compilation, chunk) = self.as_ref()?;
    Ok(
      chunk
        .get_all_initial_chunks(&compilation.chunk_group_by_ukey)
        .into_iter()
        .map(|chunk_ukey| ChunkWrapper::new(chunk_ukey, compilation))
        .collect::<Vec<_>>(),
    )
  }

  #[napi(ts_return_type = "Chunk[]")]
  pub fn get_all_referenced_chunks(&self) -> napi::Result<Vec<ChunkWrapper>> {
    let (compilation, chunk) = self.as_ref()?;
    Ok(
      chunk
        .get_all_referenced_chunks(&compilation.chunk_group_by_ukey)
        .into_iter()
        .map(|chunk_ukey| ChunkWrapper::new(chunk_ukey, compilation))
        .collect::<Vec<_>>(),
    )
  }

  #[napi(getter, js_name = "_groupsIterable", ts_return_type = "ChunkGroup[]")]
  pub fn groups_iterable(&self) -> napi::Result<Vec<ChunkGroupWrapper>> {
    let (compilation, chunk) = self.as_ref()?;
    let mut groups = chunk
      .groups()
      .iter()
      .filter_map(|group| compilation.chunk_group_by_ukey.get(group))
      .collect::<Vec<_>>();
    groups.sort_unstable_by(|a, b| a.index.cmp(&b.index));
    Ok(
      groups
        .iter()
        .map(|group| ChunkGroupWrapper::new(group.ukey, compilation))
        .collect::<Vec<_>>(),
    )
  }

  #[napi(ts_return_type = "EntryOptionsDTO | undefined")]
  pub fn get_entry_options(&self) -> napi::Result<Option<EntryOptionsDTO>> {
    let (compilation, chunk) = self.as_ref()?;

    let entry_options = chunk.get_entry_options(&compilation.chunk_group_by_ukey);

    Ok(entry_options.map(|options| EntryOptionsDTO::new(options.clone())))
  }
}

thread_local! {
  static CHUNK_INSTANCE_REFS: RefCell<UkeyMap<CompilationId, UkeyMap<rspack_core::ChunkUkey, OneShotRef>>> = Default::default();
}

pub struct ChunkWrapper {
  pub chunk_ukey: rspack_core::ChunkUkey,
  pub compilation_id: CompilationId,
  pub compilation: NonNull<Compilation>,
}

unsafe impl Send for ChunkWrapper {}

impl ChunkWrapper {
  pub fn new(chunk_ukey: rspack_core::ChunkUkey, compilation: &Compilation) -> Self {
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    #[allow(clippy::unwrap_used)]
    Self {
      chunk_ukey,
      compilation_id: compilation.id(),
      compilation: NonNull::new(compilation as *const Compilation as *mut Compilation).unwrap(),
    }
  }

  pub fn cleanup_last_compilation(compilation_id: CompilationId) {
    CHUNK_INSTANCE_REFS.with(|refs| {
      let mut refs_by_compilation_id = refs.borrow_mut();
      refs_by_compilation_id.remove(&compilation_id)
    });
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
            let refs = UkeyMap::default();
            entry.insert(refs)
          }
        };

        match refs.entry(val.chunk_ukey) {
          std::collections::hash_map::Entry::Occupied(entry) => {
            let r = entry.get();
            ToNapiValue::to_napi_value(env, r)
          }
          std::collections::hash_map::Entry::Vacant(entry) => {
            let js_module = Chunk {
              chunk_ukey: val.chunk_ukey,
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
pub struct JsChunkAssetArgs {
  #[napi(ts_type = "Chunk")]
  pub chunk: ChunkWrapper,
  pub filename: String,
}
