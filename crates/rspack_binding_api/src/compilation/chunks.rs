use napi::{
  Either, Env, Property, PropertyAttributes,
  bindgen_prelude::{Array, JsObjectValue, This, Unknown},
};
use rspack_core::{ChunkUkey, CompilationId};

use crate::{
  chunk::{Chunk, ChunkWrapper},
  with_compilation,
};

const VALUES_SYMBOL: &str = "rspack.Chunks.values";

#[napi]
pub struct Chunks {
  compilation_id: CompilationId,
  snapshot: Option<Vec<ChunkUkey>>,
}

impl Chunks {
  pub fn new(compilation_id: CompilationId) -> Self {
    Self {
      compilation_id,
      snapshot: None,
    }
  }
}

#[napi]
impl Chunks {
  #[napi]
  pub fn size(&self) -> napi::Result<u32> {
    with_compilation(self.compilation_id, |compilation| {
      Ok(compilation.build_chunk_graph_artifact.chunk_by_ukey.len() as u32)
    })
  }

  #[napi(ts_return_type = "Chunk[]")]
  pub fn values<'env>(
    &mut self,
    env: &'env Env,
    mut this: This<'env>,
  ) -> napi::Result<Array<'env>> {
    with_compilation(self.compilation_id, |compilation| {
      let chunks = &compilation.build_chunk_graph_artifact.chunk_by_ukey;
      let identifiers = || chunks.keys();
      let symbol = env.symbol_for(VALUES_SYMBOL)?;
      // Validate the live owner before cache lookup. Compare identifiers in
      // iteration order, without allocating or converting JS values on a hit.
      if self
        .snapshot
        .as_ref()
        .is_some_and(|snapshot| identifiers().eq(snapshot.iter()))
      {
        return this.get_property(symbol);
      }

      let snapshot: Vec<_> = identifiers().copied().collect();
      let mut array: Array = if this.has_own_property_js(symbol)? {
        this.get_property(symbol)?
      } else {
        let array = env.create_array(snapshot.len() as u32)?;
        // V8 owns the array through the collection object; Rust retains only IDs.
        this.define_properties(&[Property::new()
          .with_name(env, symbol)?
          .with_value(&array)
          .with_property_attributes(PropertyAttributes::Writable)])?;
        array
      };
      // Reuse the array even when membership changes. Invalidate the snapshot
      // before writing so a failed conversion cannot validate a partial update.
      self.snapshot = None;
      for (index, identifier) in snapshot.iter().enumerate() {
        array.set(index as u32, ChunkWrapper::new(*identifier, compilation))?;
      }
      // Truncate removed entries as well as growing the array. Existing JS
      // iterators keep this same array and observe its updated contents.
      array.set_named_property("length", snapshot.len() as u32)?;
      // Array caches its length in Rust; read it again after changing JS length.
      let array = this.get_property(symbol)?;
      self.snapshot = Some(snapshot);
      Ok(array)
    })
  }

  #[napi(ts_args_type = "value: Chunk")]
  pub fn has(&self, value: Either<&Chunk, Unknown<'_>>) -> napi::Result<bool> {
    with_compilation(self.compilation_id, |compilation| {
      let Either::A(chunk) = value else {
        return Ok(false);
      };
      Ok(
        compilation
          .build_chunk_graph_artifact
          .chunk_by_ukey
          .contains(&chunk.chunk_ukey),
      )
    })
  }
}
