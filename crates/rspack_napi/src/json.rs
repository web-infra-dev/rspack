use json::{number::Number, object::Object, short::Short, JsonValue};
use napi::{Env, JsUnknown};

pub trait JsonExt {
  fn to_js(&self, env: Env) -> napi::Result<JsUnknown>;
}

impl JsonExt for Object {
  fn to_js(&self, env: Env) -> napi::Result<JsUnknown> {
    let mut object = env.create_object()?;
    for (k, v) in self.iter() {
      object.set_named_property(k, (*v).to_js(env)?)?;
    }
    Ok(object.into_unknown())
  }
}

impl JsonExt for Short {
  fn to_js(&self, env: Env) -> napi::Result<JsUnknown> {
    env.create_string(self.as_str()).map(|v| v.into_unknown())
  }
}

impl JsonExt for String {
  fn to_js(&self, env: Env) -> napi::Result<JsUnknown> {
    env.create_string(self.as_str()).map(|v| v.into_unknown())
  }
}

impl JsonExt for Number {
  fn to_js(&self, env: Env) -> napi::Result<JsUnknown> {
    env.create_double((*self).into()).map(|v| v.into_unknown())
  }
}

impl JsonExt for JsonValue {
  fn to_js(&self, env: Env) -> napi::Result<JsUnknown> {
    Ok(match self {
      JsonValue::Null => env.get_null()?.into_unknown(),
      JsonValue::Short(s) => s.to_js(env)?,
      JsonValue::String(s) => env.create_string(s)?.into_unknown(),
      JsonValue::Number(n) => n.to_js(env)?,
      JsonValue::Boolean(b) => env.get_boolean(*b)?.into_unknown(),
      JsonValue::Array(vec) => {
        let mut array = env.create_array_with_length(vec.len())?;
        for (i, v) in vec.iter().enumerate() {
          array.set_element(i as u32, v.to_js(env)?)?;
        }
        array.into_unknown()
      }
      JsonValue::Object(o) => o.to_js(env)?,
    })
  }
}
