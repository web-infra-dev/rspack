mod js_loader;

use std::fmt::Formatter;
use std::{collections::HashMap, fmt::Debug, sync::Arc};

use derivative::Derivative;
use napi::bindgen_prelude::Either3;
use napi::Either;
use napi_derive::napi;
use rspack_core::{
  AssetGeneratorDataUrl, AssetGeneratorDataUrlFnArgs, AssetGeneratorDataUrlOptions,
  AssetGeneratorOptions, AssetInlineGeneratorOptions, AssetParserDataUrl,
  AssetParserDataUrlOptions, AssetParserOptions, AssetResourceGeneratorOptions, BoxLoader,
  CssAutoGeneratorOptions, CssAutoParserOptions, CssGeneratorOptions, CssModuleGeneratorOptions,
  CssModuleParserOptions, CssParserOptions, DescriptionData, DynamicImportMode, FuncUseCtx,
  GeneratorOptions, GeneratorOptionsByModuleType, JavascriptParserOptions, JavascriptParserOrder,
  JavascriptParserUrl, ModuleNoParseRule, ModuleNoParseRules, ModuleNoParseTestFn, ModuleOptions,
  ModuleRule, ModuleRuleEnforce, ModuleRuleUse, ModuleRuleUseLoader, ModuleType, ParserOptions,
  ParserOptionsByModuleType,
};
use rspack_error::error;
use rspack_loader_react_refresh::REACT_REFRESH_LOADER_IDENTIFIER;
use rspack_loader_swc::SWC_LOADER_IDENTIFIER;
use rspack_napi::regexp::{JsRegExp, JsRegExpExt};
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use serde::Deserialize;
use tokio::runtime::Handle;

pub use self::js_loader::JsLoaderAdapter;
pub use self::js_loader::*;
use crate::RawResolveOptions;

pub fn get_builtin_loader(builtin: &str, options: Option<&str>) -> BoxLoader {
  if builtin.starts_with(SWC_LOADER_IDENTIFIER) {
    return Arc::new(
      rspack_loader_swc::SwcLoader::new(
        serde_json::from_str(options.unwrap_or("{}")).unwrap_or_else(|e| {
          panic!("Could not parse builtin:swc-loader options:{options:?},error: {e:?}")
        }),
      )
      .with_identifier(builtin.into()),
    );
  }
  if builtin.starts_with(REACT_REFRESH_LOADER_IDENTIFIER) {
    return Arc::new(
      rspack_loader_react_refresh::ReactRefreshLoader::default().with_identifier(builtin.into()),
    );
  }

  unreachable!("Unexpected builtin loader: {builtin}")
}

/// `loader` is for both JS and Rust loaders.
/// `options` is
///   - a `None` on rust side and handled by js side `getOptions` when
/// using with `loader`.
///   - a `Some(string)` on rust side, deserialized by `serde_json::from_str`
/// and passed to rust side loader in [get_builtin_loader] when using with
/// `builtin_loader`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawModuleRuleUse {
  pub loader: String,
  pub options: Option<String>,
}

impl Debug for RawModuleRuleUse {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("RawModuleRuleUse")
      .field("loader", &self.loader)
      .field("options", &self.options)
      .finish()
  }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object, object_to_js = false)]
pub struct RawModuleRuleUses {
  #[napi(ts_type = r#""array" | "function""#)]
  pub r#type: String,
  pub array_use: Option<Vec<RawModuleRuleUse>>,
  #[serde(skip_deserializing)]
  #[napi(ts_type = "(arg: RawFuncUseCtx) => RawModuleRuleUse[]")]
  pub func_use: Option<ThreadsafeFunction<RawFuncUseCtx, Vec<RawModuleRuleUse>>>,
}

impl Debug for RawModuleRuleUses {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("RawModuleRuleUses")
      .field("r#type", &self.r#type)
      .field("array_use", &self.array_use)
      .field("func_use", &"...")
      .finish()
  }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawRegexMatcher {
  pub source: String,
  pub flags: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object, object_to_js = false)]
pub struct RawRuleSetCondition {
  #[napi(ts_type = r#""string" | "regexp" | "logical" | "array" | "function""#)]
  pub r#type: String,
  pub string_matcher: Option<String>,
  pub regexp_matcher: Option<RawRegexMatcher>,
  pub logical_matcher: Option<Vec<RawRuleSetLogicalConditions>>,
  pub array_matcher: Option<Vec<RawRuleSetCondition>>,
  #[serde(skip_deserializing)]
  #[napi(ts_type = r#"(value: string) => boolean"#)]
  pub func_matcher: Option<ThreadsafeFunction<String, bool>>,
}

impl Debug for RawRuleSetCondition {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("RawRuleSetCondition")
      .field("r#type", &self.r#type)
      .field("string_matcher", &self.string_matcher)
      .field("regexp_matcher", &self.regexp_matcher)
      .field("logical_matcher", &self.logical_matcher)
      .field("array_matcher", &self.array_matcher)
      .field("func_matcher", &"...")
      .finish()
  }
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object, object_to_js = false)]
pub struct RawRuleSetLogicalConditions {
  pub and: Option<Vec<RawRuleSetCondition>>,
  pub or: Option<Vec<RawRuleSetCondition>>,
  pub not: Option<RawRuleSetCondition>,
}

impl TryFrom<RawRuleSetLogicalConditions> for rspack_core::RuleSetLogicalConditions {
  type Error = rspack_error::Error;

  fn try_from(value: RawRuleSetLogicalConditions) -> rspack_error::Result<Self> {
    Ok(Self {
      and: value
        .and
        .map(|i| {
          i.into_iter()
            .map(TryFrom::try_from)
            .collect::<rspack_error::Result<Vec<_>>>()
        })
        .transpose()?,
      or: value
        .or
        .map(|i| {
          i.into_iter()
            .map(TryFrom::try_from)
            .collect::<rspack_error::Result<Vec<_>>>()
        })
        .transpose()?,
      not: value.not.map(TryFrom::try_from).transpose()?,
    })
  }
}

impl TryFrom<RawRuleSetCondition> for rspack_core::RuleSetCondition {
  type Error = rspack_error::Error;

  fn try_from(x: RawRuleSetCondition) -> rspack_error::Result<Self> {
    let result = match x.r#type.as_str() {
      "string" => Self::String(x.string_matcher.ok_or_else(|| {
        error!("should have a string_matcher when RawRuleSetCondition.type is \"string\"")
      })?),
      "regexp" => {
        let reg_matcher = x.regexp_matcher.as_ref().ok_or_else(|| {
          error!(
            "should have a regexp_matcher when RawRuleSetCondition.type is \"regexp\""
          )
        })?;
        let reg = rspack_regex::RspackRegex::with_flags(&reg_matcher.source, &reg_matcher.flags)?;
        tracing::debug!(regex_matcher = ?x.regexp_matcher, algo_type = ?reg.algo);
        Self::Regexp(reg)
      },
      "logical" => {
        let mut logical_matcher = x.logical_matcher.ok_or_else(|| {
          error!(
            "should have a logical_matcher when RawRuleSetCondition.type is \"logical\""
          )
        })?;
        let logical_matcher = logical_matcher.get_mut(0).ok_or_else(|| {
          error!(
            "TODO: use Box after https://github.com/napi-rs/napi-rs/issues/1500 landed"
          )
        })?;
        let logical_matcher = std::mem::take(logical_matcher);
        Self::Logical(Box::new(rspack_core::RuleSetLogicalConditions::try_from(
          logical_matcher,
        )?))
      }
      "array" => Self::Array(
        x.array_matcher
          .ok_or_else(|| {
            error!(
              "should have a array_matcher when RawRuleSetCondition.type is \"array\""
            )
          })?
          .into_iter()
          .map(|i| i.try_into())
          .collect::<rspack_error::Result<Vec<_>>>()?,
      ),
      "function" => {
        let func_matcher = x.func_matcher.ok_or_else(|| {
          error!(
            "should have a func_matcher when RawRuleSetCondition.type is \"function\""
          )
        })?;
        Self::Func(Box::new(move|data: &str| {
          let data = data.to_string();
          let func_matcher = func_matcher.clone();
          Box::pin(async move { func_matcher.call(data).await })
        }))
      }
      _ => panic!(
        "Failed to resolve the condition type {}. Expected type is `string`, `regexp`, `array`, `logical` or `function`.",
        x.r#type
      ),
    };

    Ok(result)
  }
}

#[derive(Derivative, Deserialize)]
#[derivative(Debug, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object, object_to_js = false)]
pub struct RawModuleRule {
  /// A conditional match matching an absolute path + query + fragment.
  /// Note:
  ///   This is a custom matching rule not initially designed by webpack.
  ///   Only for single-threaded environment interoperation purpose.
  pub rspack_resource: Option<RawRuleSetCondition>,
  /// A condition matcher matching an absolute path.
  pub test: Option<RawRuleSetCondition>,
  pub include: Option<RawRuleSetCondition>,
  pub exclude: Option<RawRuleSetCondition>,
  /// A condition matcher matching an absolute path.
  pub resource: Option<RawRuleSetCondition>,
  /// A condition matcher against the resource query.
  pub resource_query: Option<RawRuleSetCondition>,
  pub resource_fragment: Option<RawRuleSetCondition>,
  pub description_data: Option<HashMap<String, RawRuleSetCondition>>,
  pub side_effects: Option<bool>,
  pub r#use: Option<RawModuleRuleUses>,
  pub r#type: Option<String>,
  pub parser: Option<RawParserOptions>,
  pub generator: Option<RawGeneratorOptions>,
  pub resolve: Option<RawResolveOptions>,
  pub issuer: Option<RawRuleSetCondition>,
  pub dependency: Option<RawRuleSetCondition>,
  pub scheme: Option<RawRuleSetCondition>,
  pub mimetype: Option<RawRuleSetCondition>,
  pub one_of: Option<Vec<RawModuleRule>>,
  pub rules: Option<Vec<RawModuleRule>>,
  /// Specifies the category of the loader. No value means normal loader.
  #[napi(ts_type = "'pre' | 'post'")]
  pub enforce: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawParserOptions {
  #[napi(ts_type = r#""asset" | "css" | "css/auto" | "css/module" | "javascript""#)]
  pub r#type: String,
  pub asset: Option<RawAssetParserOptions>,
  pub css: Option<RawCssParserOptions>,
  pub css_auto: Option<RawCssAutoParserOptions>,
  pub css_module: Option<RawCssModuleParserOptions>,
  pub javascript: Option<RawJavascriptParserOptions>,
}

impl From<RawParserOptions> for ParserOptions {
  fn from(value: RawParserOptions) -> Self {
    match value.r#type.as_str() {
      "asset" => Self::Asset(
        value
          .asset
          .expect("should have an \"asset\" when RawParserOptions.type is \"asset\"")
          .into(),
      ),
      "javascript" => Self::Javascript(
        value
          .javascript
          .expect("should have an \"javascript\" when RawParserOptions.type is \"javascript\"")
          .into(),
      ),
      "css" => Self::Css(
        value
          .css
          .expect("should have an \"css\" when RawParserOptions.type is \"css\"")
          .into(),
      ),
      "css/auto" => Self::CssAuto(
        value
          .css_auto
          .expect("should have an \"css_auto\" when RawParserOptions.type is \"css/auto\"")
          .into(),
      ),
      "css/module" => Self::CssModule(
        value
          .css_module
          .expect("should have an \"css_module\" when RawParserOptions.type is \"css/module\"")
          .into(),
      ),
      _ => panic!(
        "Failed to resolve the RawParserOptions.type {}.",
        value.r#type
      ),
    }
  }
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawJavascriptParserOptions {
  pub dynamic_import_mode: String,
  pub dynamic_import_preload: String,
  pub dynamic_import_prefetch: String,
  pub url: String,
}

impl From<RawJavascriptParserOptions> for JavascriptParserOptions {
  fn from(value: RawJavascriptParserOptions) -> Self {
    Self {
      dynamic_import_mode: DynamicImportMode::from(value.dynamic_import_mode.as_str()),
      dynamic_import_preload: JavascriptParserOrder::from(value.dynamic_import_preload.as_str()),
      dynamic_import_prefetch: JavascriptParserOrder::from(value.dynamic_import_prefetch.as_str()),
      url: JavascriptParserUrl::from(value.url.as_str()),
    }
  }
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawAssetParserOptions {
  pub data_url_condition: Option<RawAssetParserDataUrl>,
}

impl From<RawAssetParserOptions> for AssetParserOptions {
  fn from(value: RawAssetParserOptions) -> Self {
    Self {
      data_url_condition: value.data_url_condition.map(|i| i.into()),
    }
  }
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawAssetParserDataUrl {
  #[napi(ts_type = r#""options""#)]
  pub r#type: String,
  pub options: Option<RawAssetParserDataUrlOptions>,
  // TODO: pub function
}

impl From<RawAssetParserDataUrl> for AssetParserDataUrl {
  fn from(value: RawAssetParserDataUrl) -> Self {
    match value.r#type.as_str() {
      "options" => Self::Options(
        value
          .options
          .expect("should have an \"options\" when RawAssetParserDataUrl.type is \"options\"")
          .into(),
      ),
      _ => panic!(
        "Failed to resolve the RawAssetParserDataUrl.type {}. Expected type is `options`.",
        value.r#type
      ),
    }
  }
}

#[derive(Debug, Clone, Deserialize, Default)]
#[napi(object)]
#[serde(rename_all = "camelCase")]
pub struct RawAssetParserDataUrlOptions {
  pub max_size: Option<u32>,
}

impl From<RawAssetParserDataUrlOptions> for AssetParserDataUrlOptions {
  fn from(value: RawAssetParserDataUrlOptions) -> Self {
    Self {
      max_size: value.max_size,
    }
  }
}

#[derive(Debug, Deserialize, Default)]
#[napi(object)]
#[serde(rename_all = "camelCase")]
pub struct RawCssParserOptions {
  pub named_exports: Option<bool>,
}

impl From<RawCssParserOptions> for CssParserOptions {
  fn from(value: RawCssParserOptions) -> Self {
    Self {
      named_exports: value.named_exports,
    }
  }
}

#[derive(Debug, Deserialize, Default)]
#[napi(object)]
#[serde(rename_all = "camelCase")]
pub struct RawCssAutoParserOptions {
  pub named_exports: Option<bool>,
}

impl From<RawCssAutoParserOptions> for CssAutoParserOptions {
  fn from(value: RawCssAutoParserOptions) -> Self {
    Self {
      named_exports: value.named_exports,
    }
  }
}

#[derive(Debug, Deserialize, Default)]
#[napi(object)]
#[serde(rename_all = "camelCase")]
pub struct RawCssModuleParserOptions {
  pub named_exports: Option<bool>,
}

impl From<RawCssModuleParserOptions> for CssModuleParserOptions {
  fn from(value: RawCssModuleParserOptions) -> Self {
    Self {
      named_exports: value.named_exports,
    }
  }
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object, object_to_js = false)]
pub struct RawGeneratorOptions {
  #[napi(
    ts_type = r#""asset" | "asset/inline" | "asset/resource" | "css" | "css/auto" | "css/module""#
  )]
  pub r#type: String,
  pub asset: Option<RawAssetGeneratorOptions>,
  pub asset_inline: Option<RawAssetInlineGeneratorOptions>,
  pub asset_resource: Option<RawAssetResourceGeneratorOptions>,
  pub css: Option<RawCssGeneratorOptions>,
  pub css_auto: Option<RawCssAutoGeneratorOptions>,
  pub css_module: Option<RawCssModuleGeneratorOptions>,
}

impl From<RawGeneratorOptions> for GeneratorOptions {
  fn from(value: RawGeneratorOptions) -> Self {
    match value.r#type.as_str() {
      "asset" => Self::Asset(
        value
          .asset
          .expect("should have an \"asset\" when RawGeneratorOptions.type is \"asset\"")
          .into(),
      ),
      "asset/inline" => Self::AssetInline(
        value
          .asset_inline
          .expect(
            "should have an \"asset_inline\" when RawGeneratorOptions.type is \"asset/inline\"",
          )
          .into(),
      ),
      "asset/resource" => Self::AssetResource(
        value
          .asset_resource
          .expect(
            "should have an \"asset_resource\" when RawGeneratorOptions.type is \"asset/resource\"",
          )
          .into(),
      ),
      "css" => Self::Css(
        value
          .css
          .expect("should have an \"css\" when RawGeneratorOptions.type is \"css\"")
          .into(),
      ),
      "css/auto" => Self::CssAuto(
        value
          .css_auto
          .expect("should have an \"css_auto\" when RawGeneratorOptions.type is \"css/auto\"")
          .into(),
      ),
      "css/module" => Self::CssModule(
        value
          .css_module
          .expect("should have an \"css_module\" when RawGeneratorOptions.type is \"css/module\"")
          .into(),
      ),
      _ => panic!(
        r#"Failed to resolve the RawGeneratorOptions.type {}."#,
        value.r#type
      ),
    }
  }
}

#[derive(Derivative, Deserialize, Default)]
#[derivative(Debug)]
#[serde(rename_all = "camelCase")]
#[napi(object, object_to_js = false)]
pub struct RawAssetGeneratorOptions {
  pub filename: Option<String>,
  pub public_path: Option<String>,
  #[derivative(Debug = "ignore")]
  #[serde(skip_deserializing)]
  #[napi(
    ts_type = "RawAssetGeneratorDataUrlOptions | ((arg: RawAssetGeneratorDataUrlFnArgs) => string)"
  )]
  pub data_url: Option<RawAssetGeneratorDataUrl>,
}

impl From<RawAssetGeneratorOptions> for AssetGeneratorOptions {
  fn from(value: RawAssetGeneratorOptions) -> Self {
    Self {
      filename: value.filename.map(|i| i.into()),
      public_path: value.public_path.map(|i| i.into()),
      data_url: value
        .data_url
        .map(|i| RawAssetGeneratorDataUrlWrapper(i).into()),
    }
  }
}

#[derive(Derivative, Deserialize, Default)]
#[derivative(Debug)]
#[serde(rename_all = "camelCase")]
#[napi(object, object_to_js = false)]
pub struct RawAssetInlineGeneratorOptions {
  #[derivative(Debug = "ignore")]
  #[serde(skip_deserializing)]
  #[napi(
    ts_type = "RawAssetGeneratorDataUrlOptions | ((arg: RawAssetGeneratorDataUrlFnArgs) => string)"
  )]
  pub data_url: Option<RawAssetGeneratorDataUrl>,
}

impl From<RawAssetInlineGeneratorOptions> for AssetInlineGeneratorOptions {
  fn from(value: RawAssetInlineGeneratorOptions) -> Self {
    Self {
      data_url: value
        .data_url
        .map(|i| RawAssetGeneratorDataUrlWrapper(i).into()),
    }
  }
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawAssetResourceGeneratorOptions {
  pub emit: Option<bool>,
  pub filename: Option<String>,
  pub public_path: Option<String>,
}

impl From<RawAssetResourceGeneratorOptions> for AssetResourceGeneratorOptions {
  fn from(value: RawAssetResourceGeneratorOptions) -> Self {
    Self {
      emit: value.emit,
      filename: value.filename.map(|i| i.into()),
      public_path: value.public_path.map(|i| i.into()),
    }
  }
}

type RawAssetGeneratorDataUrl = Either<
  RawAssetGeneratorDataUrlOptions,
  ThreadsafeFunction<RawAssetGeneratorDataUrlFnArgs, String>,
>;
struct RawAssetGeneratorDataUrlWrapper(RawAssetGeneratorDataUrl);

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawAssetGeneratorDataUrlFnArgs {
  pub filename: String,
  pub content: String,
}

impl From<AssetGeneratorDataUrlFnArgs> for RawAssetGeneratorDataUrlFnArgs {
  fn from(value: AssetGeneratorDataUrlFnArgs) -> Self {
    Self {
      filename: value.filename,
      content: value.content,
    }
  }
}

impl From<RawAssetGeneratorDataUrlWrapper> for AssetGeneratorDataUrl {
  fn from(value: RawAssetGeneratorDataUrlWrapper) -> Self {
    let handle = Handle::current();
    match value.0 {
      Either::A(a) => Self::Options(a.into()),
      Either::B(b) => Self::Func(Arc::new(move |ctx| handle.block_on(b.call(ctx.into())))),
    }
  }
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawAssetGeneratorDataUrlOptions {
  #[napi(ts_type = r#""base64" | "false" | undefined"#)]
  pub encoding: Option<String>,
  pub mimetype: Option<String>,
}

impl From<RawAssetGeneratorDataUrlOptions> for AssetGeneratorDataUrlOptions {
  fn from(value: RawAssetGeneratorDataUrlOptions) -> Self {
    Self {
      encoding: value.encoding.map(|i| i.into()),
      mimetype: value.mimetype,
    }
  }
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawCssGeneratorOptions {
  #[napi(ts_type = r#""as-is" | "camel-case" | "camel-case-only" | "dashes" | "dashes-only""#)]
  pub exports_convention: Option<String>,
  pub exports_only: Option<bool>,
}

impl From<RawCssGeneratorOptions> for CssGeneratorOptions {
  fn from(value: RawCssGeneratorOptions) -> Self {
    Self {
      exports_convention: value.exports_convention.map(|n| n.into()),
      exports_only: value.exports_only,
    }
  }
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawCssAutoGeneratorOptions {
  #[napi(ts_type = r#""as-is" | "camel-case" | "camel-case-only" | "dashes" | "dashes-only""#)]
  pub exports_convention: Option<String>,
  pub exports_only: Option<bool>,
  pub local_ident_name: Option<String>,
}

impl From<RawCssAutoGeneratorOptions> for CssAutoGeneratorOptions {
  fn from(value: RawCssAutoGeneratorOptions) -> Self {
    Self {
      exports_convention: value.exports_convention.map(|n| n.into()),
      exports_only: value.exports_only,
      local_ident_name: value.local_ident_name.map(|n| n.into()),
    }
  }
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawCssModuleGeneratorOptions {
  #[napi(ts_type = r#""as-is" | "camel-case" | "camel-case-only" | "dashes" | "dashes-only""#)]
  pub exports_convention: Option<String>,
  pub exports_only: Option<bool>,
  pub local_ident_name: Option<String>,
}

impl From<RawCssModuleGeneratorOptions> for CssModuleGeneratorOptions {
  fn from(value: RawCssModuleGeneratorOptions) -> Self {
    Self {
      exports_convention: value.exports_convention.map(|n| n.into()),
      exports_only: value.exports_only,
      local_ident_name: value.local_ident_name.map(|n| n.into()),
    }
  }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object, object_to_js = false)]
pub struct RawModuleOptions {
  pub rules: Vec<RawModuleRule>,
  pub parser: Option<HashMap<String, RawParserOptions>>,
  pub generator: Option<HashMap<String, RawGeneratorOptions>>,
  #[napi(
    ts_type = "string | RegExp | ((request: string) => boolean) | (string | RegExp | ((request: string) => boolean))[]"
  )]
  #[serde(skip_deserializing)]
  pub no_parse: Option<RawModuleNoParseRules>,
}

impl Debug for RawModuleOptions {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("RawModuleOptions")
      .field("rules", &self.rules)
      .field("parser", &self.parser)
      .field("generator", &self.generator)
      .field("no_parse", &"...")
      .finish()
  }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawFuncUseCtx {
  pub resource: Option<String>,
  pub real_resource: Option<String>,
  pub resource_query: Option<String>,
  pub issuer: Option<String>,
}

impl From<FuncUseCtx> for RawFuncUseCtx {
  fn from(value: FuncUseCtx) -> Self {
    Self {
      resource: value.resource,
      real_resource: value.real_resource,
      resource_query: value.resource_query,
      issuer: value.issuer.map(|s| s.to_string()),
    }
  }
}

impl TryFrom<RawModuleRule> for ModuleRule {
  type Error = rspack_error::Error;

  fn try_from(value: RawModuleRule) -> rspack_error::Result<Self> {
    // Even this part is using the plural version of loader, it's recommended to use singular version from js side to reduce overhead (This behavior maybe changed later for advanced usage).
    let uses = value.r#use.map(|raw| match raw.r#type.as_str() {
      "array" => {
        let uses = raw
          .array_use
          .map(|uses| {
            uses
              .into_iter()
              .map(|rule_use| ModuleRuleUseLoader {
                loader: rule_use.loader,
                options: rule_use.options,
              })
              .collect::<Vec<_>>()
          })
          .unwrap_or_default();
        Ok::<ModuleRuleUse, rspack_error::Error>(ModuleRuleUse::Array(uses))
      }
      "function" => {
        let func_use = raw.func_use.ok_or_else(|| {
          error!("should have a func_matcher when RawRuleSetCondition.type is \"function\"")
        })?;
        Ok::<ModuleRuleUse, rspack_error::Error>(ModuleRuleUse::Func(Box::new(
          move |ctx: FuncUseCtx| {
            let func_use = func_use.clone();
            Box::pin(async move {
              func_use.call(ctx.into()).await.map(|uses| {
                uses
                  .into_iter()
                  .map(|rule_use| ModuleRuleUseLoader {
                    loader: rule_use.loader,
                    options: rule_use.options,
                  })
                  .collect::<Vec<_>>()
              })
            })
          },
        )))
      }
      _ => Ok::<ModuleRuleUse, rspack_error::Error>(ModuleRuleUse::Array(vec![])),
    });

    let module_type = value.r#type.map(|t| (&*t).into());

    let one_of = value
      .one_of
      .map(|one_of| {
        one_of
          .into_iter()
          .map(|raw| raw.try_into())
          .collect::<rspack_error::Result<Vec<_>>>()
      })
      .transpose()?;

    let rules = value
      .rules
      .map(|rule| {
        rule
          .into_iter()
          .map(|raw| raw.try_into())
          .collect::<rspack_error::Result<Vec<_>>>()
      })
      .transpose()?;

    let description_data = value
      .description_data
      .map(|data| {
        data
          .into_iter()
          .map(|(k, v)| Ok((k, v.try_into()?)))
          .collect::<rspack_error::Result<DescriptionData>>()
      })
      .transpose()?;

    let enforce = value
      .enforce
      .map(|enforce| match &*enforce {
        "pre" => Ok(ModuleRuleEnforce::Pre),
        "post" => Ok(ModuleRuleEnforce::Post),
        _ => Err(error!(
          "Unsupported Rule.enforce type, supported: 'pre' | 'post' | undefined"
        )),
      })
      .transpose()?
      .unwrap_or_default();

    Ok(ModuleRule {
      rspack_resource: value
        .rspack_resource
        .map(|raw| raw.try_into())
        .transpose()?,
      test: value.test.map(|raw| raw.try_into()).transpose()?,
      include: value.include.map(|raw| raw.try_into()).transpose()?,
      exclude: value.exclude.map(|raw| raw.try_into()).transpose()?,
      resource_query: value.resource_query.map(|raw| raw.try_into()).transpose()?,
      resource_fragment: value
        .resource_fragment
        .map(|raw| raw.try_into())
        .transpose()?,
      resource: value.resource.map(|raw| raw.try_into()).transpose()?,
      description_data,
      r#use: uses.transpose()?.unwrap_or_default(),
      r#type: module_type,
      parser: value.parser.map(|raw| raw.into()),
      generator: value.generator.map(|raw| raw.into()),
      resolve: value.resolve.map(|raw| raw.try_into()).transpose()?,
      side_effects: value.side_effects,
      issuer: value.issuer.map(|raw| raw.try_into()).transpose()?,
      dependency: value.dependency.map(|raw| raw.try_into()).transpose()?,
      scheme: value.scheme.map(|raw| raw.try_into()).transpose()?,
      mimetype: value.mimetype.map(|raw| raw.try_into()).transpose()?,
      one_of,
      rules,
      enforce,
    })
  }
}

impl TryFrom<RawModuleOptions> for ModuleOptions {
  type Error = rspack_error::Error;

  fn try_from(value: RawModuleOptions) -> rspack_error::Result<Self> {
    let rules = value
      .rules
      .into_iter()
      .map(|rule| rule.try_into())
      .collect::<rspack_error::Result<Vec<ModuleRule>>>()?;
    Ok(ModuleOptions {
      rules,
      parser: value
        .parser
        .map(|x| {
          x.into_iter()
            .map(|(k, v)| Ok((ModuleType::from(k.as_str()), v.into())))
            .collect::<std::result::Result<ParserOptionsByModuleType, rspack_error::Error>>()
        })
        .transpose()?,
      generator: value
        .generator
        .map(|x| {
          x.into_iter()
            .map(|(k, v)| Ok((ModuleType::from(k.as_str()), v.into())))
            .collect::<std::result::Result<GeneratorOptionsByModuleType, rspack_error::Error>>()
        })
        .transpose()?,
      no_parse: value
        .no_parse
        .map(|x| RawModuleNoParseRulesWrapper(x).into()),
    })
  }
}

type RawModuleNoParseRule = Either3<String, JsRegExp, ThreadsafeFunction<String, Option<bool>>>;
type RawModuleNoParseRules = Either<RawModuleNoParseRule, Vec<RawModuleNoParseRule>>;

struct RawModuleNoParseRuleWrapper(RawModuleNoParseRule);

struct RawModuleNoParseRulesWrapper(RawModuleNoParseRules);

fn js_func_to_no_parse_test_func(
  v: ThreadsafeFunction<String, Option<bool>>,
) -> ModuleNoParseTestFn {
  Box::new(move |s| {
    let v = v.clone();
    Box::pin(async move { v.call(s).await.map(|v| v.unwrap_or_default()) })
  })
}

impl From<RawModuleNoParseRuleWrapper> for ModuleNoParseRule {
  fn from(x: RawModuleNoParseRuleWrapper) -> Self {
    match x.0 {
      Either3::A(v) => Self::AbsPathPrefix(v),
      Either3::B(v) => Self::Regexp(v.to_rspack_regex()),
      Either3::C(v) => Self::TestFn(js_func_to_no_parse_test_func(v)),
    }
  }
}

impl From<RawModuleNoParseRulesWrapper> for ModuleNoParseRules {
  fn from(x: RawModuleNoParseRulesWrapper) -> Self {
    match x.0 {
      Either::A(v) => Self::Rule(RawModuleNoParseRuleWrapper(v).into()),
      Either::B(v) => Self::Rules(
        v.into_iter()
          .map(|r| RawModuleNoParseRuleWrapper(r).into())
          .collect(),
      ),
    }
  }
}
