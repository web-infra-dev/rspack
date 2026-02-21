use std::{fmt::Debug, path::Path, sync::Arc};

use napi::{
  Either, Env,
  bindgen_prelude::{Either4, Function, FunctionCallContext, Object, Promise, ToNapiValue},
};
use napi_derive::napi;
use rspack_core::{
  Context, DependencyCategory, ExternalItem, ExternalItemFnCtx, ExternalItemFnResult,
  ExternalItemValue, ResolveOptionsWithDependencyType, ResolverFactory,
};
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use rspack_regex::RspackRegex;
use rustc_hash::FxHashMap as HashMap;

use crate::{
  error::ErrorCode,
  normal_module_factory::ContextInfo,
  options::raw_resolve::{
    RawResolveOptionsWithDependencyType, normalize_raw_resolve_options_with_dependency_type,
  },
  resolver::ResolveRequest,
  utils::callbackify,
};

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
  pub place_in_initial: bool,
}

type RawExternalItem = Either4<
  String,
  RspackRegex,
  HashMap<String, RawExternalItemValue>,
  ThreadsafeFunction<RawExternalItemFnCtx, Promise<RawExternalItemFnResult>>,
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

#[derive(Debug)]
#[napi(object, object_from_js = false)]
pub struct RawExternalItemFnCtxData<'a> {
  pub request: &'a str,
  pub context: &'a str,
  pub dependency_type: &'a str,
  pub context_info: &'a ContextInfo,
}

#[derive(Debug)]
struct RawExternalItemFnCtxInner {
  request: String,
  context: Context,
  dependency_type: DependencyCategory,
  context_info: ContextInfo,
  resolve_options_with_dependency_type: Arc<ResolveOptionsWithDependencyType>,
  resolver_factory: Arc<ResolverFactory>,
}

#[derive(Debug)]
#[napi]
pub struct RawExternalItemFnCtx {
  i: Option<RawExternalItemFnCtxInner>,
}

impl Drop for RawExternalItemFnCtx {
  fn drop(&mut self) {
    let inner = self.i.take();
    rayon::spawn(move || drop(inner));
  }
}

#[napi]
impl RawExternalItemFnCtx {
  #[napi]
  pub fn data(&self) -> RawExternalItemFnCtxData<'_> {
    #[allow(clippy::unwrap_used)]
    let inner = self.i.as_ref().unwrap();
    RawExternalItemFnCtxData {
      request: inner.request.as_str(),
      context: inner.context.as_str(),
      dependency_type: inner.dependency_type.as_str(),
      context_info: &inner.context_info,
    }
  }

  #[napi(
    ts_return_type = "(context: string, path: string, callback: (error?: Error, text?: string) => void) => void"
  )]
  pub fn get_resolve<'a>(
    &self,
    env: &'a Env,
    options: Option<RawResolveOptionsWithDependencyType>,
  ) -> napi::Result<Function<'a, (String, String, Function<'static>), ()>> {
    #[allow(clippy::unwrap_used)]
    let inner = self.i.as_ref().unwrap();
    let first = inner.resolve_options_with_dependency_type.clone();
    let second = Arc::new(
      normalize_raw_resolve_options_with_dependency_type(options, first.resolve_to_context)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?,
    );
    let resolver_factory = inner.resolver_factory.clone();

    let f: Function<(String, String, Function<'static>), ()> =
      env.create_function_from_closure("resolve", move |ctx: FunctionCallContext| {
        let context = ctx.get::<String>(0)?;
        let request = ctx.get::<String>(1)?;
        let callback = ctx.get::<Function<'static>>(2)?;

        let first = first.clone();
        let second = second.clone();
        let resolver_factory = resolver_factory.clone();

        callbackify(
          callback,
          async move {
            let merged_resolve_options = match second.resolve_options.as_ref() {
              Some(second_resolve_options) => match first.resolve_options.as_ref() {
                Some(first_resolve_options) => Some(Box::new(
                  first_resolve_options
                    .clone()
                    .merge(*second_resolve_options.clone()),
                )),
                None => Some(second_resolve_options.clone()),
              },
              None => first.as_ref().resolve_options.clone(),
            };

            let merged_options = ResolveOptionsWithDependencyType {
              resolve_options: merged_resolve_options,
              resolve_to_context: first.resolve_to_context,
              dependency_category: first.dependency_category,
            };
            let resolver = resolver_factory.get(merged_options);

            match resolver.resolve(Path::new(&context), &request).await {
              Ok(rspack_core::ResolveResult::Resource(resource)) => {
                let resolve_request = ResolveRequest::from(resource);
                Ok(match serde_json::to_string(&resolve_request) {
                  Ok(json) => Either::<String, ()>::A(json),
                  Err(_) => Either::B(()),
                })
              }
              Ok(rspack_core::ResolveResult::Ignored) => Ok(Either::B(())),
              Err(err) => Err(napi::Error::new(
                ErrorCode::Napi(napi::Status::GenericFailure),
                format!("{err:?}"),
              )),
            }
          },
          None::<fn()>,
        )
        .map_err(|e| napi::Error::from_reason(e.reason.clone()))
      })?;

    Ok(f)
  }
}

impl From<ExternalItemFnCtx> for RawExternalItemFnCtx {
  fn from(value: ExternalItemFnCtx) -> Self {
    Self {
      i: Some(RawExternalItemFnCtxInner {
        request: value.request,
        dependency_type: value.dependency_type,
        context: value.context,
        context_info: ContextInfo {
          issuer: value.context_info.issuer,
          issuer_layer: value.context_info.issuer_layer,
        },
        resolve_options_with_dependency_type: Arc::new(value.resolve_options_with_dependency_type),
        resolver_factory: value.resolver_factory,
      }),
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
        Box::pin(async move { v.call_with_promise(ctx.into()).await.map(|r| r.into()) })
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
