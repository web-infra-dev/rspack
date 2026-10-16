use std::{any::TypeId, cell::RefCell};

use napi::{
  Env, JsValue, Property, Unknown,
  bindgen_prelude::{FnArgs, Function, JsObjectValue, Object, ObjectRef},
};
use rustc_hash::FxHashMap;

thread_local! {
  static SHARED_PROPERTIES: RefCell<FxHashMap<(napi::sys::napi_env, TypeId), ObjectRef>> = RefCell::default();
}

/// Install own properties using the same JavaScript accessor functions in each env.
/// Reusing `Property` alone would still make `napi_define_properties` allocate new
/// functions and FunctionTemplateInfo objects for every instance.
///
/// `create` must only define shared behavior, never capture an instance or owner.
pub(crate) fn define_shared_properties<T: 'static>(
  env: &Env,
  object: Object<'_>,
  create: impl FnOnce() -> napi::Result<Vec<Property>>,
) -> napi::Result<()> {
  let key = (env.raw(), TypeId::of::<T>());
  let cached = SHARED_PROPERTIES.with_borrow(|cache| {
    cache
      .get(&key)
      .map(|reference| reference.get_value(env))
      .transpose()
  })?;
  let shared = match cached {
    Some(shared) => shared,
    None => {
      let object_constructor = env
        .get_global()?
        .get_named_property::<Unknown>("Object")?
        .coerce_to_object()?;
      let get_descriptors: Function<Object, Object> =
        object_constructor.get_named_property("getOwnPropertyDescriptors")?;
      let define_properties: Function<FnArgs<(Object, Object)>, Object> =
        object_constructor.get_named_property("defineProperties")?;
      let mut template = Object::new(env)?;
      template.define_properties(&create()?)?;
      let descriptors = get_descriptors.call(template)?;
      let mut shared = Object::new(env)?;
      // napi-rs attaches getter-closure finalizers to the object, not the getter.
      // Keep the template alive as long as any shared accessor can be called.
      shared.set_named_property("template", template)?;
      shared.set_named_property("descriptors", descriptors)?;
      shared.set_named_property("defineProperties", define_properties)?;
      let reference = shared.create_ref()?;
      if let Err(error) = env.add_env_cleanup_hook(key, |key| {
        let reference = SHARED_PROPERTIES.with_borrow_mut(|cache| cache.remove(&key));
        if let Some(reference) = reference {
          // The hook runs on the owning JS thread while this env is still valid.
          let _ = reference.unref(&Env::from(key.0));
        }
      }) {
        reference.unref(env)?;
        return Err(error);
      }
      SHARED_PROPERTIES.with_borrow_mut(|cache| cache.insert(key, reference));
      shared
    }
  };
  // Release the cache borrow before entering JavaScript.
  let define_properties: Function<FnArgs<(Object, Object)>, Object> =
    shared.get_named_property("defineProperties")?;
  let descriptors: Object = shared.get_named_property("descriptors")?;
  define_properties.call((object, descriptors).into())?;
  Ok(())
}
