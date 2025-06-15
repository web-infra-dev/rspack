use std::collections::HashMap;

use napi::bindgen_prelude::BigInt;
use napi_derive::napi;

#[napi(object)]
#[derive(Debug)]
pub struct RawTraceEvent {
  // event name
  pub name: String,
  // separate track name
  pub track_name: Option<String>,
  // separate group sliced name
  pub process_name: Option<String>,
  // extra debug arguments
  pub args: Option<HashMap<String, String>>,
  // track_uuid
  pub uuid: u32,
  // timestamp in microseconds
  pub ts: BigInt,
  // chrome trace event ph
  pub ph: String,
  // category
  pub categories: Option<Vec<String>>,
}
