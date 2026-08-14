use std::sync::Arc;

use napi::bindgen_prelude::{Either3, Uint32Array};
use napi_derive::napi;
use rspack_core::ChunkUkey;
use rspack_plugin_split_chunks::{ChunkNameGetter, ChunkNameGetterFnCtx};
use rustc_hash::FxHashMap;

use crate::{
  chunk::ChunkWrapper, compiler_scoped_tsfn::CompilerScopedTsFnHandle as ThreadsafeFunction,
  module::ModuleObject,
};

pub(super) type RawChunkOptionName =
  Either3<String, bool, ThreadsafeFunction<JsChunkOptionNameBatch, Vec<Option<String>>>>;

#[inline]
pub(super) fn default_chunk_option_name() -> ChunkNameGetter {
  ChunkNameGetter::Disabled
}

const CHUNK_DEDUP_HASH_THRESHOLD: usize = 16;

#[napi(object, object_from_js = false)]
pub struct JsChunkOptionNameBatch {
  #[napi(ts_type = "Module[]")]
  pub modules: Vec<ModuleObject>,
  #[napi(ts_type = "Chunk[]")]
  pub chunks: Vec<ChunkWrapper>,
  // The first `modules.len() + 1` values are offsets into the remaining chunk indices.
  pub chunk_data: Uint32Array,
  pub cache_group_key: String,
}

impl<'a> From<Vec<ChunkNameGetterFnCtx<'a>>> for JsChunkOptionNameBatch {
  fn from(contexts: Vec<ChunkNameGetterFnCtx<'a>>) -> Self {
    let mut chunk_ukeys = Vec::<ChunkUkey>::new();
    let mut chunk_indices_by_ukey = None::<FxHashMap<ChunkUkey, u32>>;
    let mut modules = Vec::new();
    let mut chunks = Vec::new();
    let mut chunk_offsets = Vec::with_capacity(contexts.len() + 1);
    let mut chunk_indices = Vec::new();
    let cache_group_key = contexts[0].cache_group_key.to_string();

    chunk_offsets.push(0);
    for context in contexts {
      debug_assert_eq!(context.cache_group_key, cache_group_key);

      modules.push(ModuleObject::with_ref(
        context.module,
        context.compilation.compiler_id(),
      ));

      for chunk in context.chunks {
        let chunk_index = if let Some(chunk_indices_by_ukey) = &mut chunk_indices_by_ukey {
          *chunk_indices_by_ukey.entry(*chunk).or_insert_with(|| {
            let index = chunks.len() as u32;
            chunks.push(ChunkWrapper::new(*chunk, context.compilation));
            index
          })
        } else if let Some(index) = chunk_ukeys.iter().position(|item| item == chunk) {
          index as u32
        } else {
          let index = chunks.len() as u32;
          chunk_ukeys.push(*chunk);
          chunks.push(ChunkWrapper::new(*chunk, context.compilation));
          if chunk_ukeys.len() == CHUNK_DEDUP_HASH_THRESHOLD {
            chunk_indices_by_ukey = Some(
              chunk_ukeys
                .iter()
                .enumerate()
                .map(|(index, chunk)| (*chunk, index as u32))
                .collect(),
            );
          }
          index
        };
        chunk_indices.push(chunk_index);
      }
      chunk_offsets.push(chunk_indices.len() as u32);
    }

    chunk_offsets.extend(chunk_indices);

    Self {
      modules,
      chunks,
      chunk_data: chunk_offsets.into(),
      cache_group_key,
    }
  }
}

pub(super) fn normalize_raw_chunk_name(raw: RawChunkOptionName) -> ChunkNameGetter {
  match raw {
    Either3::A(str) => ChunkNameGetter::String(str),
    Either3::B(_) => ChunkNameGetter::Disabled, // FIXME: when set bool is true?
    Either3::C(v) => ChunkNameGetter::Fn(Arc::new(move |contexts: Vec<ChunkNameGetterFnCtx>| {
      let batch = contexts.into();
      let v = v.clone();
      Box::pin(async move { v.call_with_sync(batch).await })
    })),
  }
}
