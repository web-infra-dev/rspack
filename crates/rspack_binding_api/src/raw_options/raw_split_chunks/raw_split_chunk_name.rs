use std::sync::Arc;

use napi::bindgen_prelude::{Either3, Uint32Array};
use napi_derive::napi;
use rspack_core::ChunkUkey;
use rspack_plugin_split_chunks::{ChunkNameGetter, ChunkNameGetterFnCtx, SplitChunksNameBatchFn};
use rustc_hash::FxHashMap;

use crate::{
  chunk::ChunkWrapper, compiler_scoped_tsfn::CompilerScopedTsFnHandle as ThreadsafeFunction,
  module::ModuleObject,
};

pub(super) type RawChunkOptionName =
  Either3<String, bool, ThreadsafeFunction<JsChunkOptionNameCtx, Option<String>>>;
pub(super) type RawChunkOptionNameBatch =
  ThreadsafeFunction<JsChunkOptionNameBatch, Vec<Option<String>>>;

#[inline]
pub(super) fn default_chunk_option_name() -> ChunkNameGetter {
  ChunkNameGetter::Disabled
}

const CHUNK_DEDUP_HASH_THRESHOLD: usize = 16;

#[napi(object, object_from_js = false)]
pub struct JsChunkOptionNameCtx {
  #[napi(ts_type = "Module")]
  pub module: ModuleObject,
  #[napi(ts_type = "Chunk[]")]
  pub chunks: Vec<ChunkWrapper>,
  pub cache_group_key: String,
}

impl<'a> From<ChunkNameGetterFnCtx<'a>> for JsChunkOptionNameCtx {
  fn from(context: ChunkNameGetterFnCtx<'a>) -> Self {
    Self {
      module: ModuleObject::with_ref(context.module, context.compilation.compiler_id()),
      chunks: context
        .chunks
        .iter()
        .map(|chunk| ChunkWrapper::new(*chunk, context.compilation))
        .collect(),
      cache_group_key: context.cache_group_key.to_string(),
    }
  }
}

#[napi(object, object_from_js = false)]
pub struct JsChunkOptionNameBatch {
  #[napi(ts_type = "Module[]")]
  pub modules: Vec<ModuleObject>,
  #[napi(ts_type = "Chunk[]")]
  pub chunks: Vec<ChunkWrapper>,
  // Example input: two calls to the original `name` function:
  //
  //   name(module M0, [chunk A, chunk B], key)
  //   name(module M1, [chunk B],          key)
  //
  // The batch sends each module and unique chunk object only once:
  //
  //   modules = [M0, M1]
  //   chunks  = [A, B]       // chunk index 0 means A; chunk index 1 means B
  //
  // `chunk_data` joins two logical arrays into one allocation:
  //
  //   offsets       = [0, 2, 3]
  //   chunk_indices = [0, 1, 1]
  //
  //                 chunk_data
  //   ┌───────────────────────┬─────────────────────────────┐
  //   │ offsets               │ chunk_indices               │
  //   │ [0, 2, 3]             │ [0, 1, 1]                   │
  //   └───────────────────────┴─────────────────────────────┘
  //
  // `offsets` stores a range in `chunk_indices` for every module:
  //
  //   module M0 -> chunk_indices[offsets[0]..offsets[1]] = chunk_indices[0..2]
  //   module M1 -> chunk_indices[offsets[1]..offsets[2]] = chunk_indices[2..3]
  //
  // The JS side decodes module M0 as follows:
  //
  //   range         = chunk_data[0]..chunk_data[1] = 0..2
  //   chunk indices = chunk_data[(3 + 0)..(3 + 2)] = [0, 1]
  //   chunk objects = [chunks[0], chunks[1]]        = [A, B]
  //
  // It decodes module M1 in the same way:
  //
  //   range         = chunk_data[1]..chunk_data[2] = 2..3
  //   chunk indices = chunk_data[(3 + 2)..(3 + 3)] = [1]
  //   chunk objects = [chunks[1]]                   = [B]
  pub chunk_data: Uint32Array,
  pub cache_group_key: String,
}

impl<'a> From<Vec<ChunkNameGetterFnCtx<'a>>> for JsChunkOptionNameBatch {
  fn from(contexts: Vec<ChunkNameGetterFnCtx<'a>>) -> Self {
    let context_count = contexts.len();
    let chunk_reference_count: usize = contexts.iter().map(|context| context.chunks.len()).sum();
    let chunk_index_start = context_count + 1;
    // `modules` and `chunk_data` have exact capacities. `chunks` is deduplicated, so the total
    // reference count could greatly overallocate it; cap its initial capacity at 16.
    let mut chunk_ukeys =
      Vec::<ChunkUkey>::with_capacity(chunk_reference_count.min(CHUNK_DEDUP_HASH_THRESHOLD));
    let mut chunk_indices_by_ukey = None::<FxHashMap<ChunkUkey, u32>>;
    let mut modules = Vec::with_capacity(context_count);
    let mut chunks = Vec::with_capacity(chunk_reference_count.min(CHUNK_DEDUP_HASH_THRESHOLD));
    let mut chunk_data = Vec::with_capacity(chunk_index_start + chunk_reference_count);
    chunk_data.resize(chunk_index_start, 0);
    let cache_group_key = contexts[0].cache_group_key.to_string();

    for (context_index, context) in contexts.into_iter().enumerate() {
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
        chunk_data.push(chunk_index);
      }
      chunk_data[context_index + 1] = (chunk_data.len() - chunk_index_start) as u32;
    }

    Self {
      modules,
      chunks,
      chunk_data: chunk_data.into(),
      cache_group_key,
    }
  }
}

pub(super) fn normalize_raw_chunk_name(raw: RawChunkOptionName) -> ChunkNameGetter {
  match raw {
    Either3::A(str) => ChunkNameGetter::String(str),
    Either3::B(_) => ChunkNameGetter::Disabled, // FIXME: when set bool is true?
    Either3::C(v) => ChunkNameGetter::Fn(Arc::new(move |context: ChunkNameGetterFnCtx| {
      let context = context.into();
      let v = v.clone();
      Box::pin(async move { v.call_with_sync(context).await })
    })),
  }
}

pub(super) fn normalize_raw_chunk_name_batch(
  raw: RawChunkOptionNameBatch,
) -> SplitChunksNameBatchFn {
  Arc::new(move |contexts: Vec<ChunkNameGetterFnCtx>| {
    let batch = contexts.into();
    let raw = raw.clone();
    Box::pin(async move { raw.call_with_sync(batch).await })
  })
}
