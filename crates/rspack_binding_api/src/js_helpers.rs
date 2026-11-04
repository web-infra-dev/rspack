use napi::{Env, JsString, JsValue, bindgen_prelude::*};
use rspack_napi::WeakRef;

use crate::symbols;

#[napi(object, object_from_js = false)]
pub struct JsNormalModuleData<'env> {
  pub resource: JsString<'env>,
  pub request: JsString<'env>,
  pub user_request: JsString<'env>,
  pub raw_request: JsString<'env>,
  #[napi(ts_type = "Readonly<JsResourceData>")]
  pub resource_resolve_data: crate::resource_data::ReadonlyResourceDataWrapper,
  pub loaders: Vec<crate::plugins::JsLoaderItem>,
}

#[napi(object, object_from_js = false)]
pub struct JsModuleData<'env> {
  pub identifier: JsString<'env>,
  pub module_type: JsString<'env>,
  pub normal: Option<JsNormalModuleData<'env>>,
  pub user_request: Option<JsString<'env>>,
}

#[napi(object)]
pub struct JsHelpers<'env> {
  #[napi(ts_type = "(kind: number, data: JsModuleData) => Module")]
  pub create_module: Function<'env>,
  #[napi(ts_type = "<T>(source: ReadonlyArray<T>, target: T[], commands: Uint32Array) => void")]
  pub apply_indexed_array_updates: Function<'env>,
  #[napi(ts_type = "<T>(array: T[], removedIndices: Uint32Array) => void")]
  pub swap_remove_array_elements: Function<'env>,
}

/// Weak handles to JS-owned helper functions. Callers must pin the JS owner
/// and release native borrows before invoking these synchronous helpers.
pub(crate) struct JsHelperRefs {
  create_module: WeakRef,
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
      create_module: WeakRef::new(env.raw(), &mut helpers.create_module.coerce_to_object()?)?,
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
      .with_name(env, env.symbol_for(symbols::JS_HELPERS_SYMBOL)?)?
      .with_napi_value(env, helpers)?
      .with_property_attributes(PropertyAttributes::Default)])?;
    Ok(references)
  }

  pub(crate) fn create_module<'env>(
    &self,
    env: &'env Env,
    kind: u32,
    data: JsModuleData<'env>,
  ) -> napi::Result<Object<'env>> {
    // The caller pins the Compiler, which owns this JS helper, until the call returns.
    let function = unsafe {
      Function::<FnArgs<(u32, JsModuleData<'env>)>, Object<'env>>::from_napi_value(
        env.raw(),
        ToNapiValue::to_napi_value(env.raw(), &self.create_module)?,
      )?
    };
    function.call((kind, data).into())
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
    .with_name(env, env.symbol_for(name)?)?
    .with_value(&value)
    .with_property_attributes(PropertyAttributes::Default)])?;
  Ok(reference)
}

/// One synchronous update using call-scoped source indices. Callers release
/// native borrows and pin the JS helper owner before applying the update.
#[derive(Default)]
pub(crate) struct IndexedArrayUpdate<'env> {
  pub(crate) source: Option<Array<'env>>,
  pub(crate) target: Option<Array<'env>>,
  // [mode, final length, ...payload].
  // Fill (0) uses source indices; Patch (1) uses target/source index pairs.
  pub(crate) commands: Vec<u32>,
}

impl IndexedArrayUpdate<'_> {
  pub(crate) fn apply(self, env: &Env, helpers: &JsHelperRefs) -> napi::Result<()> {
    if self.commands.is_empty() {
      return Ok(());
    }
    // SAFETY: construction validates this function; the caller pins its JS
    // owner until this synchronous call finishes.
    let function = unsafe {
      Function::<FnArgs<(Array, Array, Uint32ArraySlice)>, ()>::from_napi_value(
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
        self
          .target
          .expect("nonempty indexed array update has a target array"),
        commands,
      )
        .into(),
    )
  }
}
