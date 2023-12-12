use std::sync::Arc;

use derivative::Derivative;
use napi::{Either, Env, JsFunction};
use napi_derive::napi;
use rspack_binding_values::JsChunk;
use rspack_error::Result;
use rspack_napi_shared::{
  get_napi_env,
  threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode},
  JsRegExp, JsRegExpExt, NapiResultExt,
};
use rspack_plugin_banner::{
  BannerContent, BannerContentFnCtx, BannerPluginOptions, BannerRule, BannerRules,
};

#[napi(object)]
pub struct RawBannerContentFnCtx {
  pub hash: String,
  pub chunk: JsChunk,
  pub filename: String,
}

impl<'a> From<BannerContentFnCtx<'a>> for RawBannerContentFnCtx {
  fn from(value: BannerContentFnCtx) -> Self {
    Self {
      hash: value.hash.to_string(),
      chunk: JsChunk::from(value.chunk),
      filename: value.filename.to_string(),
    }
  }
}

type RawBannerContent = Either<String, JsFunction>;
struct RawBannerContentWrapper(RawBannerContent);

impl TryFrom<RawBannerContentWrapper> for BannerContent {
  type Error = rspack_error::Error;
  fn try_from(value: RawBannerContentWrapper) -> Result<Self> {
    match value.0 {
      Either::A(s) => Ok(Self::String(s)),
      Either::B(f) => {
        let func: napi::Result<ThreadsafeFunction<RawBannerContentFnCtx, String>> = try {
          let env = get_napi_env();
          rspack_binding_macros::js_fn_into_threadsafe_fn!(f, &Env::from(env))
        };
        let func = Arc::new(func.expect("convert to threadsafe function failed"));
        Ok(BannerContent::Fn(Box::new(
          move |ctx: BannerContentFnCtx| {
            let func = func.clone();
            Box::pin(async move {
              func
                .call(ctx.into(), ThreadsafeFunctionCallMode::NonBlocking)
                .into_rspack_result()?
                .await
                .unwrap_or_else(|err| panic!("Failed to call rule.use function: {err}"))
            })
          },
        )))
      }
    }
  }
}

type RawBannerRule = Either<String, JsRegExp>;
type RawBannerRules = Either<RawBannerRule, Vec<RawBannerRule>>;
struct RawBannerRuleWrapper(RawBannerRule);
struct RawBannerRulesWrapper(RawBannerRules);

#[derive(Derivative)]
#[derivative(Debug)]
#[napi(object)]
pub struct RawBannerPluginOptions {
  #[derivative(Debug = "ignore")]
  #[napi(ts_type = "string | ((...args: any[]) => any)")]
  pub banner: RawBannerContent,
  pub entry_only: Option<bool>,
  pub footer: Option<bool>,
  pub raw: Option<bool>,
  #[napi(ts_type = "string | RegExp | (string | RegExp)[]")]
  pub test: Option<RawBannerRules>,
  #[napi(ts_type = "string | RegExp | (string | RegExp)[]")]
  pub include: Option<RawBannerRules>,
  #[napi(ts_type = "string | RegExp | (string | RegExp)[]")]
  pub exclude: Option<RawBannerRules>,
}

impl From<RawBannerRuleWrapper> for BannerRule {
  fn from(x: RawBannerRuleWrapper) -> Self {
    match x.0 {
      Either::A(s) => BannerRule::String(s),
      Either::B(r) => BannerRule::Regexp(r.to_rspack_regex()),
    }
  }
}

impl From<RawBannerRulesWrapper> for BannerRules {
  fn from(x: RawBannerRulesWrapper) -> Self {
    match x.0 {
      Either::A(v) => BannerRules::Single(RawBannerRuleWrapper(v).into()),
      Either::B(v) => v
        .into_iter()
        .map(|v| RawBannerRuleWrapper(v).into())
        .collect(),
    }
  }
}

impl TryFrom<RawBannerPluginOptions> for BannerPluginOptions {
  type Error = rspack_error::Error;
  fn try_from(value: RawBannerPluginOptions) -> Result<Self> {
    Ok(BannerPluginOptions {
      banner: RawBannerContentWrapper(value.banner).try_into()?,
      entry_only: value.entry_only,
      footer: value.footer,
      raw: value.raw,
      test: value.test.map(|v| RawBannerRulesWrapper(v).into()),
      include: value.include.map(|v| RawBannerRulesWrapper(v).into()),
      exclude: value.exclude.map(|v| RawBannerRulesWrapper(v).into()),
    })
  }
}
