use napi::{
  Env, JsValue, Property, PropertyAttributes,
  bindgen_prelude::{Array, FromNapiValue, JsObjectValue, This, Unknown},
};
use rspack_core::{CompilationId, ModuleIdentifier};

use super::with_module_graph;
use crate::module::{ModuleObject, ModuleObjectRef};

const VALUES_SYMBOL: &str = "rspack.BuiltModules.values";

#[napi]
pub struct BuiltModules {
  compilation_id: CompilationId,
  snapshot: Option<Vec<ModuleIdentifier>>,
}

impl BuiltModules {
  pub fn new(compilation_id: CompilationId) -> Self {
    Self {
      compilation_id,
      snapshot: None,
    }
  }
}

#[napi]
impl BuiltModules {
  #[napi]
  pub fn size(&self) -> napi::Result<u32> {
    with_module_graph(self.compilation_id, |compilation| {
      Ok(
        compilation
          .build_module_graph_artifact
          .built_modules()
          .filter(|identifier| compilation.module_by_identifier(identifier).is_some())
          .count() as u32,
      )
    })
  }

  #[napi(ts_args_type = "value: Module")]
  pub fn has(&self, env: &Env, value: Unknown<'_>) -> napi::Result<bool> {
    // SAFETY: Convert the live value with the matching N-API environment,
    // before borrowing the compilation's module graph.
    let module = unsafe { ModuleObjectRef::from_napi_value(env.raw(), value.raw()) };
    with_module_graph(self.compilation_id, |compilation| {
      let Ok(module) = module else {
        return Ok(false);
      };
      Ok(
        module.compiler_id == compilation.compiler_id()
          && compilation
            .module_by_identifier(&module.identifier)
            .is_some()
          && compilation
            .build_module_graph_artifact
            .built_modules()
            .any(|identifier| identifier == &module.identifier),
      )
    })
  }

  #[napi(ts_return_type = "Module[]")]
  pub fn values<'env>(
    &mut self,
    env: &'env Env,
    mut this: This<'env>,
  ) -> napi::Result<Array<'env>> {
    with_module_graph(self.compilation_id, |compilation| {
      let identifiers = || {
        compilation
          .build_module_graph_artifact
          .built_modules()
          .filter(|identifier| compilation.module_by_identifier(identifier).is_some())
      };
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
        array.set(
          index as u32,
          compilation
            .module_by_identifier(identifier)
            .map(|module| ModuleObject::with_ref(module.as_ref(), compilation.compiler_id())),
        )?;
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
}
