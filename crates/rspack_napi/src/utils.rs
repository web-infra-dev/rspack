use napi::{
  bindgen_prelude::{Array, FromNapiValue, JsObjectValue, Object, Unknown},
  Env, JsValue,
};

pub fn downcast_into<T: FromNapiValue + 'static>(o: Unknown) -> napi::Result<T> {
  <T as FromNapiValue>::from_unknown(o)
}

pub fn object_assign(target: &mut Object, source: &Object) -> napi::Result<()> {
  let names = source.get_all_property_names(
    napi::KeyCollectionMode::OwnOnly,
    napi::KeyFilter::AllProperties,
    napi::KeyConversion::KeepNumbers,
  )?;
  let names = Array::from_unknown(names.to_unknown())?;

  for index in 0..names.len() {
    if let Some(name) = names.get::<Unknown>(index)? {
      let value = source.get_property::<Unknown, Unknown>(name)?;
      target.set_property::<Unknown, Unknown>(name, value)?;
    }
  }

  Ok(())
}

pub fn object_clone<'a>(env: &Env, object: &'a Object<'a>) -> napi::Result<Object<'a>> {
  let mut new_object = Object::new(env)?;

  let names = object.get_all_property_names(
    napi::KeyCollectionMode::OwnOnly,
    napi::KeyFilter::AllProperties,
    napi::KeyConversion::KeepNumbers,
  )?;
  let names = Array::from_unknown(names.to_unknown())?;

  for index in 0..names.len() {
    if let Some(name) = names.get::<Unknown>(index)? {
      let value = object.get_property::<Unknown, Unknown>(name)?;
      new_object.set_property::<Unknown, Unknown>(name, value)?;
    }
  }

  Ok(new_object)
}
