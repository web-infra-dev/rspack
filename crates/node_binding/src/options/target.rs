use std::{str::FromStr, string::ParseError};

use rspack_core::{Target, TargetOptions};
use serde::Deserialize;

#[cfg(not(feature = "test"))]
use napi_derive::napi;

#[derive(Deserialize, Debug)]
pub struct RawTarget {
  source: String,
}

impl RawTarget {}
