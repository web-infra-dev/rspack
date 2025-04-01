use napi::{
  bindgen_prelude::{Array, FromNapiValue, Object, Unknown},
  Env,
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
  let names = Array::from_unknown(names.into_unknown())?;

  for index in 0..names.len() {
    if let Some(name) = names.get::<Unknown>(index)? {
      let value = source.get_property::<&Unknown, Unknown>(&name)?;
      target.set_property::<Unknown, Unknown>(name, value)?;
    }
  }

  Ok(())
}

pub fn object_clone(env: &Env, object: &Object) -> napi::Result<Object> {
  let mut new_object = env.create_object()?;

  let names = object.get_all_property_names(
    napi::KeyCollectionMode::OwnOnly,
    napi::KeyFilter::AllProperties,
    napi::KeyConversion::KeepNumbers,
  )?;
  let names = Array::from_unknown(names.into_unknown())?;

  println!("object_clone names.len() {}", names.len());
  for index in 0..names.len() {
    if let Some(name) = names.get::<Unknown>(index)? {
      let value = object.get_property::<&Unknown, Unknown>(&name)?;
      new_object.set_property::<Unknown, Unknown>(name, value)?;
    }
  }

  Ok(new_object)
}
