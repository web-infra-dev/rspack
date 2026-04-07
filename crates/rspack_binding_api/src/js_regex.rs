use napi::{
  Env, JsValue, Unknown, ValueType,
  bindgen_prelude::{FromNapiValue, JsObjectValue, ToNapiValue, TypeName, ValidateNapiValue},
};
use napi_derive::napi;
use rspack_regex::RspackRegex;

#[derive(Debug, Clone)]
pub struct JsRegExp {
  pub pattern: String,
  pub flags: String,
}

impl ValidateNapiValue for JsRegExp {}

impl TypeName for JsRegExp {
  fn type_name() -> &'static str {
    "RegExp"
  }

  fn value_type() -> napi::ValueType {
    napi::ValueType::Object
  }
}

impl FromNapiValue for JsRegExp {
  unsafe fn from_napi_value(
    raw_env: napi::sys::napi_env,
    napi_val: napi::sys::napi_value,
  ) -> napi::Result<Self> {
    let unknown = unsafe { Unknown::from_raw_unchecked(raw_env, napi_val) };

    let object = unknown.coerce_to_object()?;
    let pattern = object
      .get_named_property::<String>("pattern")
      .or_else(|_| object.get_named_property::<String>("source"))
      .map_err(|_| {
        napi::Error::from_reason(
          "Failed to extract the 'pattern' or 'source' property. Ensure the value is a valid RegExp object.",
        )
      })?;
    let flags = object.get_named_property::<String>("flags").map_err(|_| {
      napi::Error::from_reason(
        "Failed to extract the 'flags' property. Ensure the value is a valid RegExp object.",
      )
    })?;

    Ok(Self { pattern, flags })
  }
}

impl ToNapiValue for JsRegExp {
  unsafe fn to_napi_value(
    env: napi::sys::napi_env,
    val: Self,
  ) -> napi::Result<napi::sys::napi_value> {
    let env = Env::from(env);

    let global = env.get_global()?;
    let regex = global.get_named_property::<napi::bindgen_prelude::Function<'_, _>>("RegExp")?;

    let flags = env.create_string(&val.flags)?;
    let pattern = env.create_string(&val.pattern)?;

    Ok(regex.new_instance((pattern, flags))?.raw())
  }
}

impl TryFrom<JsRegExp> for RspackRegex {
  type Error = rspack_error::Error;

  fn try_from(value: JsRegExp) -> Result<Self, Self::Error> {
    RspackRegex::from_js_regex(value.pattern, value.flags)
  }
}
