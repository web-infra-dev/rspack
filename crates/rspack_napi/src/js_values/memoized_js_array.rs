use napi::{
  Env,
  bindgen_prelude::{
    Array, FromNapiValue, JsObjectValue, Object, Property, PropertyAttributes, ToNapiValue,
  },
};

use super::weak_ref::WeakRef;

/// Reuses one JavaScript array, updating its contents when the cached key expires.
///
/// Each instance belongs to one N-API environment and must only be accessed on
/// that environment's JavaScript thread. Callers validate any native owner before
/// consulting this cache. The JS owner holds the array through a non-enumerable symbol property;
/// Rust only holds a weak handle, so V8 can collect cycles between cached objects.
/// Consumers must not mutate or freeze the shared array;
/// retained views observe updates after the next getter refreshes its contents.
#[derive(Default)]
pub struct MemoizedJsArray<K> {
  // A missing key means initialization or an interrupted update must be retried.
  value: Option<(Option<K>, WeakRef)>,
}

impl<K> MemoizedJsArray<K> {
  pub fn get_or_update<'a>(
    &mut self,
    env: &'a Env,
    owner: &mut Object<'a>,
    symbol_name: &str,
    is_valid: impl FnOnce(&K) -> bool,
    update: impl FnOnce(&mut Array<'a>) -> napi::Result<(K, u32)>,
  ) -> napi::Result<Array<'a>> {
    let cache = &mut self.value;
    if cache.is_none() {
      let array = env.create_array(0)?;
      let mut object = array.coerce_to_object()?;
      let reference = WeakRef::new(env.raw(), &mut object)?;
      owner.define_properties(&[Property::new()
        .with_name(env, env.symbol_for(symbol_name)?)?
        .with_value(&object)
        .with_property_attributes(PropertyAttributes::Default)])?;
      *cache = Some((None, reference));
    }
    let (key, reference) = cache.as_mut().expect("array was initialized above");
    // Reconstruct the Rust handle, not the JavaScript array it references.
    let raw = unsafe { ToNapiValue::to_napi_value(env.raw(), &*reference)? };
    let mut array = unsafe { Array::from_napi_value(env.raw(), raw)? };
    if key.as_ref().is_some_and(is_valid) {
      return Ok(array);
    }

    // Keep the array identity even if conversion fails, but never reuse a key
    // for partially updated contents. A subsequent getter retries the update.
    *key = None;
    let (new_key, len) = update(&mut array)?;
    if array.len() != len {
      // Overwriting elements does not remove a previous, longer array's tail.
      array.set_named_property("length", len)?;
      // napi-rs caches length in its Rust handle; refresh it after truncation.
      array = unsafe { Array::from_napi_value(env.raw(), raw)? };
    }
    *key = Some(new_key);
    Ok(array)
  }
}
