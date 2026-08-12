use std::sync::Arc;

use napi::bindgen_prelude::{Either3, Uint32Array};
use napi_derive::napi;
use rspack_core::{ChunkUkey, ModuleIdentifier};
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

#[napi(object, object_from_js = false)]
pub struct JsChunkOptionNameBatch {
  #[napi(ts_type = "Module[]")]
  pub modules: Vec<ModuleObject>,
  #[napi(ts_type = "Chunk[]")]
  pub chunks: Vec<ChunkWrapper>,
  pub module_indices: Uint32Array,
  pub chunk_offsets: Uint32Array,
  pub chunk_indices: Uint32Array,
  pub cache_group_key: String,
}

impl<'a> From<Vec<ChunkNameGetterFnCtx<'a>>> for JsChunkOptionNameBatch {
  fn from(contexts: Vec<ChunkNameGetterFnCtx<'a>>) -> Self {
    let mut module_map = FxHashMap::<ModuleIdentifier, u32>::default();
    let mut chunk_map = FxHashMap::<ChunkUkey, u32>::default();
    let mut modules = Vec::new();
    let mut chunks = Vec::new();
    let mut module_indices = Vec::with_capacity(contexts.len());
    let mut chunk_offsets = Vec::with_capacity(contexts.len() + 1);
    let mut chunk_indices = Vec::new();
    let cache_group_key = contexts
      .first()
      .map(|context| context.cache_group_key.to_string())
      .unwrap_or_default();

    chunk_offsets.push(0);
    for context in contexts {
      debug_assert_eq!(context.cache_group_key, cache_group_key);

      let module_identifier = context.module.identifier();
      let module_index = if let Some(index) = module_map.get(&module_identifier) {
        *index
      } else {
        let index = u32::try_from(modules.len()).expect("too many modules in name batch");
        module_map.insert(module_identifier, index);
        modules.push(ModuleObject::with_ref(
          context.module,
          context.compilation.compiler_id(),
        ));
        index
      };
      module_indices.push(module_index);

      for chunk in context.chunks {
        let chunk_index = if let Some(index) = chunk_map.get(chunk) {
          *index
        } else {
          let index = u32::try_from(chunks.len()).expect("too many chunks in name batch");
          chunk_map.insert(*chunk, index);
          chunks.push(ChunkWrapper::new(*chunk, context.compilation));
          index
        };
        chunk_indices.push(chunk_index);
      }
      chunk_offsets
        .push(u32::try_from(chunk_indices.len()).expect("too many chunk references in name batch"));
    }

    Self {
      modules,
      chunks,
      module_indices: module_indices.into(),
      chunk_offsets: chunk_offsets.into(),
      chunk_indices: chunk_indices.into(),
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
