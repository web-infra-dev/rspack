use std::fmt::Debug;

use futures::future::BoxFuture;
use rspack_error::Result;
use rspack_regex::RspackRegex;
use rustc_hash::FxHashMap as HashMap;

pub type Externals = Vec<ExternalItem>;

#[derive(Debug)]
pub enum ExternalItemValue {
  String(String),
  Array(Vec<String>),
  Bool(bool),
  Object(HashMap<String, Vec<String>>),
}

pub type ExternalItemObject = HashMap<String, ExternalItemValue>;

pub struct ContextInfo {
  pub issuer: String,
}

pub struct ExternalItemFnCtx {
  pub request: String,
  pub context: String,
  pub dependency_type: String,
  pub context_info: ContextInfo,
}

pub struct ExternalItemFnResult {
  pub external_type: Option<ExternalType>,
  pub result: Option<ExternalItemValue>,
}

type ExternalItemFn =
  Box<dyn Fn(ExternalItemFnCtx) -> BoxFuture<'static, Result<ExternalItemFnResult>> + Sync + Send>;

pub enum ExternalItem {
  Object(ExternalItemObject),
  String(String),
  RegExp(RspackRegex),
  Fn(ExternalItemFn),
}

impl std::fmt::Debug for ExternalItem {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Object(v) => f.debug_tuple("Object").field(v).finish(),
      Self::String(v) => f.debug_tuple("String").field(v).finish(),
      Self::RegExp(v) => f.debug_tuple("RegExp").field(v).finish(),
      Self::Fn(_) => f.debug_tuple("Fn").field(&"...").finish(),
    }
  }
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

#[derive(Debug, Default, Clone)]
pub struct ExternalsPresets {
  pub web: bool,
  pub web_async: bool,
  pub node: bool,
  pub electron: bool,
  pub electron_main: bool,
  pub electron_preload: bool,
  pub electron_renderer: bool,
  pub nwjs: bool,
}
