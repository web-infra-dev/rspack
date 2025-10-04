use std::{fmt::Debug, sync::Arc};

use futures::future::BoxFuture;
use rspack_error::Result;
use rspack_regex::RspackRegex;
use rustc_hash::FxHashMap as HashMap;

use crate::{Context, DependencyCategory, ResolveOptionsWithDependencyType, ResolverFactory};

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
  pub issuer_layer: Option<String>,
}

pub struct ExternalItemFnCtx {
  pub request: String,
  pub context: Context,
  pub dependency_type: DependencyCategory,
  pub context_info: ContextInfo,
  pub resolve_options_with_dependency_type: ResolveOptionsWithDependencyType,
  pub resolver_factory: Arc<ResolverFactory>,
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
