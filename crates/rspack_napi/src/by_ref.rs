use std::{mem::ManuallyDrop, ops::Deref};

use derivative::Derivative;
use napi::{
  bindgen_prelude::{FromNapiValue, ToNapiValue, TypeName, ValidateNapiValue},
  Env, JsFunction, JsNumber, Ref,
};

/// Wrapper for napi types that provides by-reference equality and hash
#[derive(Derivative, Debug, Clone)]
#[derivative(Eq, PartialEq, Hash)]
pub struct ByRef<T> {
  #[derivative(PartialEq = "ignore")]
  #[derivative(Hash = "ignore")]
  value: T,

  // ref_id is the same iff the two js values are the same reference.
  // PartialEq and Hash is implemented based on its value.
  ref_id: i64,
}

fn get_ref_id(env: &mut napi::Env, value: impl ToNapiValue) -> napi::Result<i64> {
  // GET_REF_ID_FUNC holds a per env function that returns ids for identifiying references.
  // Internally it uses `WeakMap` to record ids.
  // It lives as long as the env so we wrap it with `ManuallyDrop`.
  thread_local! {
      static GET_REF_ID_FUNC: std::cell::RefCell<Option<ManuallyDrop<Ref<()>>>> = Default::default();
  }

  GET_REF_ID_FUNC.with_borrow_mut(|get_ref_id_func| {
    let func = if let Some(func_ref) = get_ref_id_func {
      env.get_reference_value::<JsFunction>(func_ref.deref())?
    } else {
      let js_func: JsFunction = env.run_script::<_, JsFunction>(
        r#"(() => {
            const refIdMap = new WeakMap();
            let lastId = 0;
            return (obj) => {
                let id = refIdMap.get(obj);
                if (id === undefined) {
                    id = lastId++;
                    refIdMap.set(obj, id);
                }
                return id;
            }
      })()"#,
      )?;
      *get_ref_id_func = Some(ManuallyDrop::new(env.create_reference(&js_func)?));
      env.add_env_cleanup_hook((), |()| {
        // Reset GET_REF_ID_FUNC when the env is exiting, just in case another env will be started in the same thread.
        // Ref<()> doesn't need to be unref'ed as the function it holds will be released by the exiting env.
        GET_REF_ID_FUNC.with_borrow_mut(|get_ref_id_func| *get_ref_id_func = None)
      })?;
      js_func
    };

    func.call1::<_, JsNumber>(value)?.get_int64()
  })
}

impl<T: FromNapiValue> FromNapiValue for ByRef<T> {
  unsafe fn from_napi_value(
    env: napi::sys::napi_env,
    napi_val: napi::sys::napi_value,
  ) -> napi::Result<Self> {
    let value = unsafe { T::from_napi_value(env, napi_val) }?;

    let mut env = unsafe { Env::from_raw(env) };
    let ref_id = get_ref_id(&mut env, napi_val)?;
    Ok(Self { value, ref_id })
  }
}

impl<T: ToNapiValue> ToNapiValue for ByRef<T> {
  unsafe fn to_napi_value(
    env: napi::sys::napi_env,
    val: Self,
  ) -> napi::Result<napi::sys::napi_value> {
    unsafe { T::to_napi_value(env, val.value) }
  }
}

impl<T: ValidateNapiValue> ValidateNapiValue for ByRef<T> {
  unsafe fn validate(
    env: napi::sys::napi_env,
    napi_val: napi::sys::napi_value,
  ) -> napi::Result<napi::sys::napi_value> {
    unsafe { T::validate(env, napi_val) }
  }
}

impl<T: TypeName> TypeName for ByRef<T> {
  fn type_name() -> &'static str {
    T::type_name()
  }

  fn value_type() -> napi::ValueType {
    T::value_type()
  }
}
impl<T> Deref for ByRef<T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    &self.value
  }
}
