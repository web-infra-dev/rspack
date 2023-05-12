use std::fmt::Debug;

use futures::future::BoxFuture;
use rspack_error::Result;
use rspack_regex::RspackRegex;
use rustc_hash::FxHashMap as HashMap;

pub type Externals = Vec<ExternalItem>;

#[derive(Debug)]
pub enum ExternalItemValue {
  String(String),
  Bool(bool),
  Array(Vec<String>), // TODO: string[] | Record<string, string|string[]>
}

pub type ExternalItemObject = HashMap<String, ExternalItemValue>;

pub struct ExternalItemFnCtx {
  pub request: String,
  pub context: String,
  pub dependency_type: String,
}

pub struct ExternalItemFnResult {
  pub external_type: Option<ExternalType>,
  pub result: Option<ExternalItemValue>,
}

pub type ExternalItemFn =
  Box<dyn Fn(ExternalItemFnCtx) -> BoxFuture<'static, Result<ExternalItemFnResult>> + Sync + Send>;

pub enum ExternalItem {
  Object(ExternalItemObject),
  String(String),
  RegExp(RspackRegex),
  Fn(ExternalItemFn),
}

impl From<ExternalItemObject> for ExternalItem {
  fn from(value: ExternalItemObject) -> Self {
    Self::Object(value)
  }
}

impl From<String> for ExternalItem {
  fn from(value: String) -> Self {
    Self::String(value)
  }
}

impl From<RspackRegex> for ExternalItem {
  fn from(value: RspackRegex) -> Self {
    Self::RegExp(value)
  }
}

pub type ExternalType = String;
