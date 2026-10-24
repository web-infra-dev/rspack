use napi::{
  Either, Env, Property, PropertyAttributes,
  bindgen_prelude::{Array, JsObjectValue, This, Unknown},
};
use napi_derive::napi;
use rspack_core::{ChunkUkey, CompilationId};
use rustc_hash::FxHashSet;

use super::with_chunk;

const VALUES_SYMBOL: &str = "rspack.ChunkFiles.values";

#[napi]
pub struct ChunkFiles {
  compilation_id: CompilationId,
  chunk_ukey: ChunkUkey,
  snapshot: Option<FxHashSet<String>>,
}

impl ChunkFiles {
  pub fn new(compilation_id: CompilationId, chunk_ukey: ChunkUkey) -> Self {
    Self {
      compilation_id,
      chunk_ukey,
      snapshot: None,
    }
  }
}

#[napi]
impl ChunkFiles {
  #[napi]
  pub fn size(&self) -> napi::Result<u32> {
    with_chunk(self.compilation_id, self.chunk_ukey, |_, chunk| {
      Ok(chunk.files().len() as u32)
    })
  }

  #[napi(ts_args_type = "value: string")]
  pub fn has(&self, value: Either<String, Unknown<'_>>) -> napi::Result<bool> {
    with_chunk(self.compilation_id, self.chunk_ukey, |_, chunk| {
      let Either::A(value) = value else {
        return Ok(false);
      };
      Ok(chunk.files().contains(&value))
    })
  }

  #[napi(ts_return_type = "string[]")]
  pub fn values<'env>(
    &mut self,
    env: &'env Env,
    mut this: This<'env>,
  ) -> napi::Result<Array<'env>> {
    with_chunk(self.compilation_id, self.chunk_ukey, |_, chunk| {
      let values = chunk.files();
      let symbol = env.symbol_for(VALUES_SYMBOL)?;
      // Resolve the live chunk before returning a cached array.
      if self
        .snapshot
        .as_ref()
        .is_some_and(|snapshot| snapshot == values)
      {
        return this.get_property(symbol);
      }

      let mut array = env.create_array(values.len() as u32)?;
      let mut sorted: Vec<_> = values.iter().collect();
      sorted.sort_unstable();
      for (index, value) in sorted.into_iter().enumerate() {
        array.set(index as u32, value.as_str())?;
      }
      // Keep the array in the V8 object graph, without a native strong reference.
      this.define_properties(&[Property::new()
        .with_name(env, symbol)?
        .with_value(&array)
        .with_property_attributes(PropertyAttributes::Writable)])?;
      self.snapshot = Some(values.clone());
      Ok(array)
    })
  }
}
