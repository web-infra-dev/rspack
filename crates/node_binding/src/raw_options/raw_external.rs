use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use napi::bindgen_prelude::Either4;
use napi_derive::napi;
use rspack_core::{ExternalItem, ExternalItemFnResult, ExternalItemValue};
use rspack_core::{ExternalItemFnCtx, ResolveOptionsWithDependencyType, ResolverFactory};
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use rspack_regex::RspackRegex;

use crate::JsResolver;

#[napi(object)]
pub struct RawHttpExternalsRspackPluginOptions {
  pub css: bool,
  pub web_async: bool,
}

#[napi(object, object_to_js = false)]
pub struct RawExternalsPluginOptions {
  pub r#type: String,
  #[napi(
    ts_type = "(string | RegExp | Record<string, string | boolean | string[] | Record<string, string[]>> | ((...args: any[]) => any))[]"
  )]
  pub externals: Vec<RawExternalItem>,
}

type RawExternalItem = Either4<
  String,
  RspackRegex,
  HashMap<String, RawExternalItemValue>,
  ThreadsafeFunction<RawExternalItemFnCtx, RawExternalItemFnResult>,
>;
type RawExternalItemValue = Either4<String, bool, Vec<String>, HashMap<String, Vec<String>>>;
pub(crate) struct RawExternalItemWrapper(pub(crate) RawExternalItem);
struct RawExternalItemValueWrapper(RawExternalItemValue);

impl From<RawExternalItemValueWrapper> for ExternalItemValue {
  fn from(value: RawExternalItemValueWrapper) -> Self {
    match value.0 {
      Either4::A(v) => Self::String(v),
      Either4::B(v) => Self::Bool(v),
      Either4::C(v) => Self::Array(v),
      Either4::D(v) => Self::Object(v.into_iter().collect()),
    }
  }
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct RawExternalItemFnResult {
  pub external_type: Option<String>,
  // sadly, napi.rs does not support type alias at the moment. Need to add Either here
  #[napi(ts_type = "string | boolean | string[] | Record<string, string[]>")]
  pub result: Option<RawExternalItemValue>,
}

impl From<RawExternalItemFnResult> for ExternalItemFnResult {
  fn from(value: RawExternalItemFnResult) -> Self {
    Self {
      external_type: value.external_type,
      result: value.result.map(|v| RawExternalItemValueWrapper(v).into()),
    }
  }
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct ContextInfo {
  pub issuer: String,
  pub issuer_layer: Option<String>,
}

#[derive(Debug)]
#[napi]
pub struct RawExternalItemFnCtx {
  request: String,
  context: String,
  dependency_type: String,
  context_info: ContextInfo,
  resolve_options_with_dependency_type: ResolveOptionsWithDependencyType,
  resolver_factory: Arc<ResolverFactory>,
}

#[derive(Debug)]
#[napi(object)]
pub struct RawExternalItemFnCtxData {
  pub request: String,
  pub context: String,
  pub dependency_type: String,
  pub context_info: ContextInfo,
}

#[napi]
impl RawExternalItemFnCtx {
  #[napi]
  pub fn data(&self) -> RawExternalItemFnCtxData {
    RawExternalItemFnCtxData {
      request: self.request.clone(),
      context: self.context.clone(),
      dependency_type: self.dependency_type.clone(),
      context_info: self.context_info.clone(),
    }
  }

  #[napi]
  pub fn get_resolver(&self) -> JsResolver {
    JsResolver::new(
      self.resolver_factory.clone(),
      self.resolve_options_with_dependency_type.clone(),
    )
  }
}

impl From<ExternalItemFnCtx> for RawExternalItemFnCtx {
  fn from(value: ExternalItemFnCtx) -> Self {
    Self {
      request: value.request,
      dependency_type: value.dependency_type,
      context: value.context,
      context_info: ContextInfo {
        issuer: value.context_info.issuer,
        issuer_layer: value.context_info.issuer_layer,
      },
      resolve_options_with_dependency_type: value.resolve_options_with_dependency_type,
      resolver_factory: value.resolver_factory,
    }
  }
}

impl TryFrom<RawExternalItemWrapper> for ExternalItem {
  type Error = rspack_error::Error;

  #[allow(clippy::unwrap_in_result)]
  fn try_from(value: RawExternalItemWrapper) -> rspack_error::Result<Self> {
    match value.0 {
      Either4::A(v) => Ok(Self::String(v)),
      Either4::B(v) => Ok(Self::RegExp(v)),
      Either4::C(v) => Ok(Self::Object(
        v.into_iter()
          .map(|(k, v)| (k, RawExternalItemValueWrapper(v).into()))
          .collect(),
      )),
      Either4::D(v) => Ok(Self::Fn(Box::new(move |ctx: ExternalItemFnCtx| {
        let v = v.clone();
        Box::pin(async move { v.call(ctx.into()).await.map(|r| r.into()) })
      }))),
    }
  }
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct RawExternalsPresets {
  pub node: bool,
  pub web: bool,
  pub electron: bool,
  pub electron_main: bool,
  pub electron_preload: bool,
  pub electron_renderer: bool,
}
