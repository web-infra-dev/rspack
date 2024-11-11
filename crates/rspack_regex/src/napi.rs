use napi::{
  bindgen_prelude::{FromNapiValue, Function, ToNapiValue, TypeName, ValidateNapiValue},
  Env, JsObject, NapiRaw, NapiValue,
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
    env: napi::sys::napi_env,
    napi_val: napi::sys::napi_value,
  ) -> napi::Result<Self> {
    let js_object = unsafe { JsObject::from_raw_unchecked(env, napi_val) };

    let env = Env::from(env);
    let global = env.get_global()?;
    let object_prototype_to_string = global
      .get_named_property_unchecked::<JsObject>("Object")?
      .get_named_property::<JsObject>("prototype")?
      .get_named_property::<Function>("toString")?;

    let js_string = object_prototype_to_string
      .apply(&js_object, env.get_undefined()?.into_unknown())?
      // .call_without_args(Some(&js_object))?
      .coerce_to_string()?
      .into_utf8()?;
    let js_object_type = js_string.as_str()?;

    if js_object_type == "[object RegExp]" {
      let source = js_object.get_named_property::<String>("source")?;
      let flags = js_object.get_named_property::<String>("flags")?;

      Self::with_flags(&source, &flags)
        .map_err(|err| napi::Error::new(napi::Status::InvalidArg, err.to_string()))
    } else {
      Err(napi::Error::new(
        napi::Status::ObjectExpected,
        format!(
          "Expect value to be '[object RegExp]', but received {}",
          js_object_type
        ),
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
