use napi::{Env, JsValue, bindgen_prelude::*};
use rspack_napi::WeakRef;

#[napi(object)]
pub struct JsHelpers<'env> {
  #[napi(ts_type = "<T>(source: ReadonlyArray<T>, targets: T[][], commands: Uint32Array) => void")]
  pub apply_indexed_array_updates: Function<'env>,
  #[napi(ts_type = "<T>(array: T[], removedIndices: Uint32Array) => void")]
  pub swap_remove_array_elements: Function<'env>,
}

/// Weak handles to JS-owned helper functions. Callers must pin the JS owner
/// and release native borrows before invoking these synchronous helpers.
pub(crate) struct JsHelperRefs {
  apply_indexed_array_updates: WeakRef,
  swap_remove_array_elements: WeakRef,
}

impl JsHelperRefs {
  pub(crate) fn new(
    env: &Env,
    owner: &mut Object<'_>,
    helpers: JsHelpers<'_>,
  ) -> napi::Result<Self> {
    let references = Self {
      apply_indexed_array_updates: WeakRef::new(
        env.raw(),
        &mut helpers.apply_indexed_array_updates.coerce_to_object()?,
      )?,
      swap_remove_array_elements: WeakRef::new(
        env.raw(),
        &mut helpers.swap_remove_array_elements.coerce_to_object()?,
      )?,
    };
    owner.define_properties(&[Property::new()
      .with_name(env, env.create_symbol(Some("rspack.jsHelpers"))?)?
      .with_napi_value(env, helpers)?
      .with_property_attributes(PropertyAttributes::Default)])?;
    Ok(references)
  }

  pub(crate) fn swap_remove_array_elements(
    &self,
    env: &Env,
    array: Array<'_>,
    removed_indices: &Uint32ArraySlice<'_>,
  ) -> napi::Result<()> {
    // SAFETY: construction validates this function; the caller pins its JS
    // owner until this synchronous call finishes.
    let function = unsafe {
      Function::<FnArgs<(Array, &Uint32ArraySlice)>, ()>::from_napi_value(
        env.raw(),
        ToNapiValue::to_napi_value(env.raw(), &self.swap_remove_array_elements)?,
      )?
    };
    function.call((array, removed_indices).into())
  }
}

/// Let the JS owner hold the value, so GC can see its entire ownership graph.
pub(crate) fn js_owned_ref(
  env: &Env,
  owner: &mut Object<'_>,
  name: &str,
  mut value: Object<'_>,
) -> napi::Result<WeakRef> {
  let reference = WeakRef::new(env.raw(), &mut value)?;
  owner.define_properties(&[Property::new()
    .with_name(env, env.create_symbol(Some(name))?)?
    .with_value(&value)
    .with_property_attributes(PropertyAttributes::Default)])?;
  Ok(reference)
}

/// One synchronous update using call-scoped source indices. Callers release
/// native borrows and pin the JS helper owner before applying the batch.
#[derive(Default)]
pub(crate) struct IndexedArrayUpdateBatch<'env> {
  pub(crate) source: Option<Array<'env>>,
  pub(crate) targets: Vec<Array<'env>>,
  // One [mode, final length, payload length, ...payload] segment per target.
  // Fill (0) uses source indices; Patch (1) uses target/source index pairs.
  pub(crate) commands: Vec<u32>,
}

impl IndexedArrayUpdateBatch<'_> {
  pub(crate) fn apply(self, env: &Env, helpers: &JsHelperRefs) -> napi::Result<()> {
    if self.commands.is_empty() {
      return Ok(());
    }
    // SAFETY: construction validates this function; the caller pins its JS
    // owner until this synchronous call finishes.
    let function = unsafe {
      Function::<FnArgs<(Array, Vec<Array>, Uint32ArraySlice)>, ()>::from_napi_value(
        env.raw(),
        ToNapiValue::to_napi_value(env.raw(), &helpers.apply_indexed_array_updates)?,
      )?
    };
    let commands = Uint32ArraySlice::from_data(env, self.commands)?;
    function.call(
      (
        self
          .source
          .expect("nonempty indexed array update has a source array"),
        self.targets,
        commands,
      )
        .into(),
    )
  }
}
