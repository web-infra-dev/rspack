use std::cell::RefCell;

use napi::bindgen_prelude::{Buffer, ToNapiValue};
use rspack_collections::Identifier;
use rspack_napi::napi::bindgen_prelude::Result;
use rustc_hash::FxHashMap as HashMap;

use crate::value_ref::Ref;

thread_local! {
  static IDENTIFIER_REFS: RefCell<HashMap<Identifier, Ref>> = Default::default();
}

#[derive(Debug)]
pub struct JsIdentifier(Identifier);

impl From<Identifier> for JsIdentifier {
  fn from(value: Identifier) -> Self {
    Self(value)
  }
}

impl ToNapiValue for JsIdentifier {
  unsafe fn to_napi_value(env: napi::sys::napi_env, val: Self) -> Result<napi::sys::napi_value> {
    IDENTIFIER_REFS.with(|refs| {
      let id = val.0;
      let mut refs = refs.borrow_mut();
      match refs.entry(id) {
        std::collections::hash_map::Entry::Occupied(entry) => {
          let r = entry.get();
          ToNapiValue::to_napi_value(env, r)
        }
        std::collections::hash_map::Entry::Vacant(entry) => {
          let vec: Vec<u8> = id.as_str().into();
          let buffer: Buffer = vec.into();
          let napi_value = ToNapiValue::to_napi_value(env, buffer)?;
          let r = Ref::new(env, napi_value, 1)?;
          let r = entry.insert(r);
          ToNapiValue::to_napi_value(env, r)
        }
      }
    })
  }
}
