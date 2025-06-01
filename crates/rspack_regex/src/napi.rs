use napi::{
  bindgen_prelude::{
    FromNapiValue, Function, JsObjectValue, Object, ToNapiValue, TypeName, Undefined,
    ValidateNapiValue,
  },
  Env, JsValue, Unknown,
};

use crate::RspackRegex;

impl ValidateNapiValue for RspackRegex {}

impl TypeName for RspackRegex {
  fn type_name() -> &'static str {
    "RegExp"
  }

  fn value_type() -> napi::ValueType {
    napi::ValueType::Object
  }
}

impl FromNapiValue for RspackRegex {
  unsafe fn from_napi_value(
    raw_env: napi::sys::napi_env,
    napi_val: napi::sys::napi_value,
  ) -> napi::Result<Self> {
    let js_object = Object::from_raw(raw_env, napi_val);

    let env = Env::from(raw_env);
    let global = env.get_global()?;
    let object_prototype_to_string = global
      .get_named_property_unchecked::<Object>("Object")?
      .get_named_property_unchecked::<Object>("prototype")?
      .get_named_property_unchecked::<Function>("toString")?;

    let raw_undefined = Undefined::to_napi_value(raw_env, ())?;
    let undefined = Unknown::from_napi_value(raw_env, raw_undefined)?;
    let js_value = object_prototype_to_string.apply(js_object, undefined)?;
    let js_string = js_value.coerce_to_string()?;
    let js_utf8_string = js_string.into_utf8()?;
    let object_type = js_utf8_string.as_str()?;

    if object_type == "[object RegExp]" {
      let source = js_object.get_named_property::<String>("source")?;
      let flags = js_object.get_named_property::<String>("flags")?;

      Self::with_flags(&source, &flags)
        .map_err(|err| napi::Error::new(napi::Status::InvalidArg, err.to_string()))
    } else {
      Err(napi::Error::new(
        napi::Status::ObjectExpected,
        format!("Expect value to be '[object RegExp]', but received {object_type}"),
      ))
    }
  }
}

impl ToNapiValue for RspackRegex {
  unsafe fn to_napi_value(
    env: napi::sys::napi_env,
    val: Self,
  ) -> napi::Result<napi::sys::napi_value> {
    let env = Env::from(env);

    let global = env.get_global()?;
    let regex = global.get_named_property::<Function<'_, _>>("RegExp")?;

    let flags = env.create_string(&val.flags)?;
    let source = env.create_string(&val.source)?;

    Ok(regex.new_instance((source, flags))?.raw())
  }
}
