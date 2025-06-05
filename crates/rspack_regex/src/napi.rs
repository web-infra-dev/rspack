use napi::{
  bindgen_prelude::{
    FromNapiValue, Function, JsObjectValue, ToNapiValue, TypeName, ValidateNapiValue,
  },
  Env, JsValue, Unknown, ValueType,
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
    let unknown = Unknown::from_raw_unchecked(raw_env, napi_val);

    if unknown.get_type()? == ValueType::Object {
      let object = unknown.coerce_to_object()?;
      let source = object.get_named_property::<String>("source").map_err(|_| {
        napi::Error::from_reason(
          "Failed to extract the 'source' property. Ensure the value is a valid RegExp object.",
        )
      })?;
      let flags = object.get_named_property::<String>("flags").map_err(|_| {
        napi::Error::from_reason(
          "Failed to extract the 'flags'. Ensure the value is a valid RegExp object.",
        )
      })?;

      Self::with_flags(&source, &flags)
        .map_err(|err| napi::Error::new(napi::Status::InvalidArg, err.to_string()))
    } else {
      Err(napi::Error::new(
        napi::Status::ObjectExpected,
        "Expected a RegExp object as input value.",
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
