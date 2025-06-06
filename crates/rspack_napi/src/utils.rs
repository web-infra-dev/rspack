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

pub fn unknown_to_json_value(value: Unknown) -> napi::Result<Option<serde_json::Value>> {
  if value.is_array()? {
    let js_array = Array::from_unknown(value)?;
    let mut array = Vec::with_capacity(js_array.len() as usize);

    for index in 0..js_array.len() {
      if let Some(item) = js_array.get::<Unknown>(index)? {
        if let Some(json_val) = unknown_to_json_value(item)? {
          array.push(json_val);
        } else {
          array.push(serde_json::Value::Null);
        }
      } else {
        array.push(serde_json::Value::Null);
      }
    }

    return Ok(Some(serde_json::Value::Array(array)));
  }

  match value.get_type()? {
    napi::ValueType::Null => Ok(Some(serde_json::Value::Null)),
    napi::ValueType::Boolean => {
      let b = value.coerce_to_bool()?;
      Ok(Some(serde_json::Value::Bool(b)))
    }
    napi::ValueType::Number => {
      let number = value.coerce_to_number()?.get_double()?;
      let f64_val = serde_json::Number::from_f64(number);
      match f64_val {
        Some(n) => Ok(Some(serde_json::Value::Number(n))),
        None => Ok(None),
      }
    }
    napi::ValueType::String => {
      let s = value.coerce_to_string()?.into_utf8()?.into_owned()?;
      Ok(Some(serde_json::Value::String(s)))
    }
    napi::ValueType::Object => {
      let object = value.coerce_to_object()?;
      let mut map = serde_json::Map::new();

      let names = Array::from_unknown(object.get_property_names()?.to_unknown())?;
      for index in 0..names.len() {
        if let Some(name) = names.get::<String>(index)? {
          let prop_val = object.get_named_property::<Unknown>(&name)?;
          if let Some(json_val) = unknown_to_json_value(prop_val)? {
            map.insert(name, json_val);
          }
        }
      }

      Ok(Some(serde_json::Value::Object(map)))
    }
    napi::ValueType::Undefined
    | napi::ValueType::Symbol
    | napi::ValueType::Function
    | napi::ValueType::External
    | napi::ValueType::BigInt
    | napi::ValueType::Unknown => Ok(None),
  }
}
