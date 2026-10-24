use napi::{
  Either, Env, Property, PropertyAttributes,
  bindgen_prelude::{Array, JsObjectValue, This, Unknown},
};
use rspack_core::{ChunkUkey, Compilation, CompilationId, CompilerId};

use crate::{
  COMPILER_REFERENCES,
  chunk::{Chunk, ChunkWrapper},
};

const VALUES_SYMBOL: &str = "rspack.Chunks.values";

#[napi]
pub struct Chunks {
  // Keep the historical behavior of Chunks: a set retained after done follows
  // the compiler's current compilation across watch rebuilds. CompilationId
  // would instead revoke that set on the next rebuild. Resolve CompilerId through
  // the weak compiler registry on every access, without keeping the owner alive.
  compiler_id: CompilerId,
  snapshot: Option<(CompilationId, Vec<ChunkUkey>)>,
}

impl Chunks {
  pub fn new(compiler_id: CompilerId) -> Self {
    Self {
      compiler_id,
      snapshot: None,
    }
  }
}

#[napi]
impl Chunks {
  #[napi]
  pub fn size(&self) -> napi::Result<u32> {
    with_current_compilation(self.compiler_id, |compilation| {
      Ok(compilation.build_chunk_graph_artifact.chunk_by_ukey.len() as u32)
    })
  }

  #[napi(ts_return_type = "Chunk[]")]
  pub fn values<'env>(
    &mut self,
    env: &'env Env,
    mut this: This<'env>,
  ) -> napi::Result<Array<'env>> {
    with_current_compilation(self.compiler_id, |compilation| {
      let chunks = &compilation.build_chunk_graph_artifact.chunk_by_ukey;
      let identifiers = || chunks.keys();
      let symbol = env.symbol_for(VALUES_SYMBOL)?;
      // Validate the live owner before cache lookup. Compare identifiers in
      // iteration order, without allocating or converting JS values on a hit.
      // Incremental rebuilds may reuse ChunkUkeys, but Chunk wrappers are scoped
      // to a CompilationId. Refresh them when the compilation changes even if
      // the identifiers match, while retaining the same JS array.
      if self
        .snapshot
        .as_ref()
        .is_some_and(|(compilation_id, snapshot)| {
          *compilation_id == compilation.id() && identifiers().eq(snapshot.iter())
        })
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
      self.snapshot = Some((compilation.id(), snapshot));
      Ok(array)
    })
  }

  #[napi(ts_args_type = "value: Chunk")]
  pub fn has(&self, value: Either<&Chunk, Unknown<'_>>) -> napi::Result<bool> {
    with_current_compilation(self.compiler_id, |compilation| {
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

fn with_current_compilation<R>(
  compiler_id: CompilerId,
  f: impl FnOnce(&Compilation) -> napi::Result<R>,
) -> napi::Result<R> {
  let reference =
    COMPILER_REFERENCES.with(|references| references.borrow().get(&compiler_id).cloned());
  let compiler = reference.as_ref().and_then(|reference| reference.get());
  let Some(compiler) = compiler else {
    return Err(napi::Error::from_reason(format!(
      "Unable to access compilation for compiler with id = {compiler_id:?} now. The Compiler has been garbage collected by JavaScript."
    )));
  };
  f(&compiler.compiler.compilation)
}
