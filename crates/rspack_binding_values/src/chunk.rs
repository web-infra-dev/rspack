use std::{cell::RefCell, collections::HashMap, ptr::NonNull};

use napi::{bindgen_prelude::ToNapiValue, Either, Env, JsString};
use napi_derive::napi;
use rspack_core::{Chunk, ChunkUkey, Compilation, CompilationId};
use rspack_napi::OneShotRef;

use crate::compilation::entries::EntryOptionsDTO;
use crate::JsChunkGroupWrapper;

#[napi]
pub struct JsChunk {
  pub(crate) chunk_ukey: ChunkUkey,
  compilation: NonNull<Compilation>,
}

impl JsChunk {
  fn as_ref(&self) -> napi::Result<(&'static Compilation, &'static Chunk)> {
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
impl JsChunk {
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
    Ok(match chunk.id(&compilation.chunk_ids_artifact) {
      Some(id) => Either::A(id.as_str()),
      None => Either::B(()),
    })
  }

  #[napi(getter)]
  pub fn ids(&self) -> napi::Result<Vec<&str>> {
    let (compilation, chunk) = self.as_ref()?;
    Ok(
      chunk
        .id(&compilation.chunk_ids_artifact)
        .map(|id| vec![id.as_str()])
        .unwrap_or_default(),
    )
  }

  #[napi(getter)]
  pub fn id_name_hints(&self, env: Env) -> napi::Result<Vec<JsString>> {
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

  #[napi(getter)]
  pub fn files(&self) -> napi::Result<Vec<&String>> {
    let (_, chunk) = self.as_ref()?;
    let mut files = Vec::from_iter(chunk.files());
    files.sort_unstable();
    Ok(files)
  }

  #[napi(getter)]
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

  #[napi(getter)]
  pub fn content_hash(&self) -> napi::Result<HashMap<String, &str>> {
    let (compilation, chunk) = self.as_ref()?;
    Ok(
      chunk
        .content_hash(&compilation.chunk_hashes_artifact)
        .map(|content_hash| {
          content_hash
            .iter()
            .map(|(key, v)| (key.to_string(), v.encoded()))
            .collect::<HashMap<String, &str>>()
        })
        .unwrap_or_default(),
    )
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

  #[napi(getter)]
  pub fn auxiliary_files(&self) -> napi::Result<Vec<&String>> {
    let (_, chunk) = self.as_ref()?;
    Ok(chunk.auxiliary_files().iter().collect::<Vec<_>>())
  }
}

#[napi]
impl JsChunk {
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

  #[napi(ts_return_type = "JsChunk[]")]
  pub fn get_all_async_chunks(&self) -> napi::Result<Vec<JsChunkWrapper>> {
    let (compilation, chunk) = self.as_ref()?;
    Ok(
      chunk
        .get_all_async_chunks(&compilation.chunk_group_by_ukey)
        .into_iter()
        .map(|chunk_ukey| JsChunkWrapper::new(chunk_ukey, compilation))
        .collect::<Vec<_>>(),
    )
  }

  #[napi(ts_return_type = "JsChunk[]")]
  pub fn get_all_initial_chunks(&self) -> napi::Result<Vec<JsChunkWrapper>> {
    let (compilation, chunk) = self.as_ref()?;
    Ok(
      chunk
        .get_all_initial_chunks(&compilation.chunk_group_by_ukey)
        .into_iter()
        .map(|chunk_ukey| JsChunkWrapper::new(chunk_ukey, compilation))
        .collect::<Vec<_>>(),
    )
  }

  #[napi(ts_return_type = "JsChunk[]")]
  pub fn get_all_referenced_chunks(&self) -> napi::Result<Vec<JsChunkWrapper>> {
    let (compilation, chunk) = self.as_ref()?;
    Ok(
      chunk
        .get_all_referenced_chunks(&compilation.chunk_group_by_ukey)
        .into_iter()
        .map(|chunk_ukey| JsChunkWrapper::new(chunk_ukey, compilation))
        .collect::<Vec<_>>(),
    )
  }

  #[napi(ts_return_type = "JsChunkGroup[]")]
  pub fn groups(&self) -> napi::Result<Vec<JsChunkGroupWrapper>> {
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
        .map(|group| JsChunkGroupWrapper::new(group.ukey, compilation))
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
  static CHUNK_INSTANCE_REFS: RefCell<HashMap<CompilationId, HashMap<ChunkUkey, OneShotRef>>> = Default::default();
}

pub struct JsChunkWrapper {
  pub chunk_ukey: ChunkUkey,
  pub compilation_id: CompilationId,
  pub compilation: NonNull<Compilation>,
}

unsafe impl Send for JsChunkWrapper {}

impl JsChunkWrapper {
  pub fn new(chunk_ukey: ChunkUkey, compilation: &Compilation) -> Self {
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

impl ToNapiValue for JsChunkWrapper {
  unsafe fn to_napi_value(
    env: napi::sys::napi_env,
    val: Self,
  ) -> napi::Result<napi::sys::napi_value> {
    CHUNK_INSTANCE_REFS.with(|refs| {
      let mut refs_by_compilation_id = refs.borrow_mut();
      let entry = refs_by_compilation_id.entry(val.compilation_id);
      let refs = match entry {
        std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
        std::collections::hash_map::Entry::Vacant(entry) => {
          let refs = HashMap::default();
          entry.insert(refs)
        }
      };

      match refs.entry(val.chunk_ukey) {
        std::collections::hash_map::Entry::Occupied(entry) => {
          let r = entry.get();
          ToNapiValue::to_napi_value(env, r)
        }
        std::collections::hash_map::Entry::Vacant(entry) => {
          let js_module = JsChunk {
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

#[napi(object, object_from_js = false)]
pub struct JsChunkAssetArgs {
  #[napi(ts_type = "JsChunk")]
  pub chunk: JsChunkWrapper,
  pub filename: String,
}
