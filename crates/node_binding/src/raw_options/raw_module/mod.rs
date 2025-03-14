use std::fmt::Formatter;
use std::{collections::HashMap, sync::Arc};

use derive_more::Debug;
use napi::bindgen_prelude::{Buffer, Either3};
use napi::Either;
use napi_derive::napi;
use rspack_core::{
  AssetGeneratorDataUrl, AssetGeneratorDataUrlFnCtx, AssetGeneratorDataUrlOptions,
  AssetGeneratorOptions, AssetInlineGeneratorOptions, AssetParserDataUrl,
  AssetParserDataUrlOptions, AssetParserOptions, AssetResourceGeneratorOptions,
  CssAutoGeneratorOptions, CssAutoParserOptions, CssGeneratorOptions, CssModuleGeneratorOptions,
  CssModuleParserOptions, CssParserOptions, DescriptionData, DynamicImportFetchPriority,
  DynamicImportMode, ExportPresenceMode, FuncUseCtx, GeneratorOptions, GeneratorOptionsMap,
  JavascriptParserOptions, JavascriptParserOrder, JavascriptParserUrl, JsonGeneratorOptions,
  JsonParserOptions, ModuleNoParseRule, ModuleNoParseRules, ModuleNoParseTestFn, ModuleOptions,
  ModuleRule, ModuleRuleEffect, ModuleRuleEnforce, ModuleRuleUse, ModuleRuleUseLoader,
  OverrideStrict, ParseOption, ParserOptions, ParserOptionsMap,
};
use rspack_error::error;
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use rspack_regex::RspackRegex;

use crate::RawResolveOptions;
use crate::{JsFilename, ModuleObject};

/// `loader` is for both JS and Rust loaders.
/// `options` is
///   - a `None` on rust side and handled by js side `getOptions` when
/// using with `loader`.
///   - a `Some(string)` on rust side, deserialized by `serde_json::from_str`
/// and passed to rust side loader in [get_builtin_loader] when using with
/// `builtin_loader`.
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

#[rspack_napi_macros::tagged_union]
pub enum RawRuleSetCondition {
  string(String),
  #[napi(ts_type = "RegExp")]
  regexp(RspackRegex),
  logical(Vec<RawRuleSetLogicalConditions>),
  array(Vec<RawRuleSetCondition>),
  #[napi(ts_type = r#"(value: string) => boolean"#)]
  func(ThreadsafeFunction<serde_json::Value, bool>),
}

impl Debug for RawRuleSetCondition {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      RawRuleSetCondition::string(s) => write!(f, "RawRuleSetCondition::string({:?})", s),
      RawRuleSetCondition::regexp(r) => write!(f, "RawRuleSetCondition::regexp({:?})", r),
      RawRuleSetCondition::logical(l) => write!(f, "RawRuleSetCondition::logical({:?})", l),
      RawRuleSetCondition::array(a) => write!(f, "RawRuleSetCondition::array({:?})", a),
      RawRuleSetCondition::func(_) => write!(f, "RawRuleSetCondition::func(...)"),
    }
  }
}

#[derive(Debug, Default)]
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
    let result = match x {
      RawRuleSetCondition::string(s) => Self::String(s),
      RawRuleSetCondition::regexp(r) => {
        let reg = RspackRegex::with_flags(&r.source, &r.flags)?;
        Self::Regexp(reg)
      }
      RawRuleSetCondition::logical(mut l) => {
        let l = l.get_mut(0).ok_or_else(|| {
          error!("TODO: use Box after https://github.com/napi-rs/napi-rs/issues/1500 landed")
        })?;
        let l = std::mem::take(l);
        Self::Logical(Box::new(rspack_core::RuleSetLogicalConditions::try_from(
          l,
        )?))
      }
      RawRuleSetCondition::array(a) => Self::Array(
        a.into_iter()
          .map(|i| i.try_into())
          .collect::<rspack_error::Result<Vec<_>>>()?,
      ),
      RawRuleSetCondition::func(f) => Self::Func(Box::new(move |data| {
        let data = data.to_value();
        let f = f.clone();
        Box::pin(async move { f.call(data).await })
      })),
    };

    Ok(result)
  }
}

impl TryFrom<RawRuleSetCondition> for rspack_core::RuleSetConditionWithEmpty {
  type Error = rspack_error::Error;

  fn try_from(x: RawRuleSetCondition) -> rspack_error::Result<Self> {
    Ok(Self::new(x.try_into()?))
  }
}

type ThreadsafeUse = ThreadsafeFunction<RawFuncUseCtx, Vec<RawModuleRuleUse>>;

#[derive(Debug, Default)]
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
  pub with: Option<HashMap<String, RawRuleSetCondition>>,
  pub side_effects: Option<bool>,
  #[napi(ts_type = "RawModuleRuleUse[] | ((arg: RawFuncUseCtx) => RawModuleRuleUse[])")]
  pub r#use: Option<Either<Vec<RawModuleRuleUse>, ThreadsafeUse>>,
  pub r#type: Option<String>,
  pub layer: Option<String>,
  pub parser: Option<RawParserOptions>,
  pub generator: Option<RawGeneratorOptions>,
  pub resolve: Option<RawResolveOptions>,
  pub issuer: Option<RawRuleSetCondition>,
  pub issuer_layer: Option<RawRuleSetCondition>,
  pub dependency: Option<RawRuleSetCondition>,
  pub scheme: Option<RawRuleSetCondition>,
  pub mimetype: Option<RawRuleSetCondition>,
  pub one_of: Option<Vec<RawModuleRule>>,
  pub rules: Option<Vec<RawModuleRule>>,
  /// Specifies the category of the loader. No value means normal loader.
  #[napi(ts_type = "'pre' | 'post'")]
  pub enforce: Option<String>,
}

#[derive(Debug, Default)]
#[napi(object, object_to_js = false)]
pub struct RawParserOptions {
  #[napi(
    ts_type = r#""asset" | "css" | "css/auto" | "css/module" | "javascript" | "javascript/auto" | "javascript/dynamic" | "javascript/esm" | "json""#
  )]
  pub r#type: String,
  pub asset: Option<RawAssetParserOptions>,
  pub css: Option<RawCssParserOptions>,
  pub css_auto: Option<RawCssAutoParserOptions>,
  pub css_module: Option<RawCssModuleParserOptions>,
  pub javascript: Option<RawJavascriptParserOptions>,
  pub json: Option<RawJsonParserOptions>,
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
      "javascript/auto" => Self::JavascriptAuto(
        value
          .javascript
          .expect("should have an \"javascript\" when RawParserOptions.type is \"javascript/auto\"")
          .into(),
      ),
      "javascript/dynamic" => Self::JavascriptDynamic(
        value
          .javascript
          .expect(
            "should have an \"javascript\" when RawParserOptions.type is \"javascript/dynamic\"",
          )
          .into(),
      ),
      "javascript/esm" => Self::JavascriptEsm(
        value
          .javascript
          .expect("should have an \"javascript\" when RawParserOptions.type is \"javascript/esm\"")
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
      "json" => Self::Json(
        value
          .json
          .expect("should have an \"json\" when RawParserOptions.type is \"json\"")
          .into(),
      ),
      _ => panic!(
        "Failed to resolve the RawParserOptions.type {}.",
        value.r#type
      ),
    }
  }
}

#[derive(Debug, Default)]
#[napi(object)]
pub struct RawJavascriptParserOptions {
  pub dynamic_import_mode: Option<String>,
  pub dynamic_import_preload: Option<String>,
  pub dynamic_import_prefetch: Option<String>,
  pub dynamic_import_fetch_priority: Option<String>,
  pub url: Option<String>,
  pub expr_context_critical: Option<bool>,
  pub wrapped_context_critical: Option<bool>,
  #[napi(ts_type = "RegExp")]
  pub wrapped_context_reg_exp: Option<RspackRegex>,
  pub exports_presence: Option<String>,
  pub import_exports_presence: Option<String>,
  pub reexport_exports_presence: Option<String>,
  pub strict_export_presence: Option<bool>,
  pub worker: Option<Vec<String>>,
  pub override_strict: Option<String>,
  pub import_meta: Option<bool>,
  /// This option is experimental in Rspack only and subject to change or be removed anytime.
  /// @experimental
  pub require_as_expression: Option<bool>,
  /// This option is experimental in Rspack only and subject to change or be removed anytime.
  /// @experimental
  pub require_dynamic: Option<bool>,
  /// This option is experimental in Rspack only and subject to change or be removed anytime.
  /// @experimental
  pub require_resolve: Option<bool>,
  /// This option is experimental in Rspack only and subject to change or be removed anytime.
  /// @experimental
  pub import_dynamic: Option<bool>,
}

impl From<RawJavascriptParserOptions> for JavascriptParserOptions {
  fn from(value: RawJavascriptParserOptions) -> Self {
    Self {
      dynamic_import_mode: value
        .dynamic_import_mode
        .map(|v| DynamicImportMode::from(v.as_str())),
      dynamic_import_preload: value
        .dynamic_import_preload
        .map(|v| JavascriptParserOrder::from(v.as_str())),
      dynamic_import_prefetch: value
        .dynamic_import_prefetch
        .map(|v| JavascriptParserOrder::from(v.as_str())),
      dynamic_import_fetch_priority: value
        .dynamic_import_fetch_priority
        .map(|x| DynamicImportFetchPriority::from(x.as_str())),
      url: value.url.map(|v| JavascriptParserUrl::from(v.as_str())),
      expr_context_critical: value.expr_context_critical,
      wrapped_context_reg_exp: value.wrapped_context_reg_exp,
      wrapped_context_critical: value.wrapped_context_critical,
      exports_presence: value
        .exports_presence
        .map(|e| ExportPresenceMode::from(e.as_str())),
      import_exports_presence: value
        .import_exports_presence
        .map(|e| ExportPresenceMode::from(e.as_str())),
      reexport_exports_presence: value
        .reexport_exports_presence
        .map(|e| ExportPresenceMode::from(e.as_str())),
      strict_export_presence: value.strict_export_presence,
      worker: value.worker,
      override_strict: value
        .override_strict
        .map(|e| OverrideStrict::from(e.as_str())),
      import_meta: value.import_meta,
      require_as_expression: value.require_as_expression,
      require_dynamic: value.require_dynamic,
      require_resolve: value.require_resolve,
      import_dynamic: value.import_dynamic,
    }
  }
}

#[derive(Debug, Default)]
#[napi(object)]
pub struct RawJsonGeneratorOptions {
  #[napi(js_name = "JSONParse")]
  pub json_parse: Option<bool>,
}

impl From<RawJsonGeneratorOptions> for JsonGeneratorOptions {
  fn from(value: RawJsonGeneratorOptions) -> Self {
    Self {
      json_parse: value.json_parse,
    }
  }
}

#[derive(Debug, Default)]
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

#[derive(Debug, Default)]
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

#[derive(Debug, Clone, Default)]
#[napi(object)]
pub struct RawAssetParserDataUrlOptions {
  pub max_size: Option<f64>,
}

impl From<RawAssetParserDataUrlOptions> for AssetParserDataUrlOptions {
  fn from(value: RawAssetParserDataUrlOptions) -> Self {
    Self {
      max_size: value.max_size,
    }
  }
}

#[derive(Debug, Default)]
#[napi(object)]
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

#[derive(Debug, Default)]
#[napi(object)]
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

#[derive(Debug, Default)]
#[napi(object)]
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

#[derive(Debug, Default)]
#[napi(object, object_to_js = false)]
pub struct RawJsonParserOptions {
  pub exports_depth: Option<u32>,
  #[napi(ts_type = "(source: string) => string")]
  pub parse: Option<ThreadsafeFunction<String, String>>,
}

impl From<RawJsonParserOptions> for JsonParserOptions {
  fn from(value: RawJsonParserOptions) -> Self {
    let parse = match value.parse {
      Some(f) => ParseOption::Func(Arc::new(move |s: String| f.blocking_call_with_sync(s))),
      _ => ParseOption::None,
    };

    Self {
      exports_depth: value.exports_depth,
      parse,
    }
  }
}

#[derive(Debug, Default)]
#[napi(object, object_to_js = false)]
pub struct RawGeneratorOptions {
  #[napi(
    ts_type = r#""asset" | "asset/inline" | "asset/resource" | "css" | "css/auto" | "css/module" | "json""#
  )]
  pub r#type: String,
  pub asset: Option<RawAssetGeneratorOptions>,
  pub asset_inline: Option<RawAssetInlineGeneratorOptions>,
  pub asset_resource: Option<RawAssetResourceGeneratorOptions>,
  pub css: Option<RawCssGeneratorOptions>,
  pub css_auto: Option<RawCssAutoGeneratorOptions>,
  pub css_module: Option<RawCssModuleGeneratorOptions>,
  pub json: Option<RawJsonGeneratorOptions>,
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
      "json" => Self::Json(
        value
          .json
          .expect("should have an \"json\" when RawGeneratorOptions.type is \"json\"")
          .into(),
      ),
      _ => panic!(
        r#"Failed to resolve the RawGeneratorOptions.type {}."#,
        value.r#type
      ),
    }
  }
}

#[derive(Default, Debug)]
#[napi(object, object_to_js = false)]
pub struct RawAssetGeneratorOptions {
  pub emit: Option<bool>,
  pub filename: Option<JsFilename>,
  pub output_path: Option<JsFilename>,
  #[napi(ts_type = "\"auto\" | JsFilename")]
  pub public_path: Option<JsFilename>,
  #[debug(skip)]
  #[napi(
    ts_type = "RawAssetGeneratorDataUrlOptions | ((source: Buffer, context: RawAssetGeneratorDataUrlFnCtx) => string)"
  )]
  pub data_url: Option<RawAssetGeneratorDataUrl>,

  #[napi(ts_type = r#""url" | "preserve""#)]
  pub import_mode: Option<String>,
}

impl From<RawAssetGeneratorOptions> for AssetGeneratorOptions {
  fn from(value: RawAssetGeneratorOptions) -> Self {
    Self {
      emit: value.emit,
      filename: value.filename.map(|i| i.into()),
      output_path: value.output_path.map(|i| i.into()),
      public_path: value.public_path.map(|i| i.into()),
      data_url: value
        .data_url
        .map(|i| RawAssetGeneratorDataUrlWrapper(i).into()),
      import_mode: value.import_mode.map(|n| n.into()),
    }
  }
}

#[derive(Default, Debug)]
#[napi(object, object_to_js = false)]
pub struct RawAssetInlineGeneratorOptions {
  #[debug(skip)]
  #[napi(
    ts_type = "RawAssetGeneratorDataUrlOptions | ((source: Buffer, context: RawAssetGeneratorDataUrlFnCtx) => string)"
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

#[derive(Debug, Default)]
#[napi(object, object_to_js = false)]
pub struct RawAssetResourceGeneratorOptions {
  pub emit: Option<bool>,
  pub filename: Option<JsFilename>,
  pub output_path: Option<JsFilename>,
  #[napi(ts_type = "\"auto\" | JsFilename")]
  pub public_path: Option<JsFilename>,
  #[napi(ts_type = r#""url" | "preserve""#)]
  pub import_mode: Option<String>,
}

impl From<RawAssetResourceGeneratorOptions> for AssetResourceGeneratorOptions {
  fn from(value: RawAssetResourceGeneratorOptions) -> Self {
    Self {
      emit: value.emit,
      filename: value.filename.map(|i| i.into()),
      output_path: value.output_path.map(|i| i.into()),
      public_path: value.public_path.map(|i| i.into()),
      import_mode: value.import_mode.map(|i| i.into()),
    }
  }
}

type RawAssetGeneratorDataUrl = Either<
  RawAssetGeneratorDataUrlOptions,
  ThreadsafeFunction<(Buffer, RawAssetGeneratorDataUrlFnCtx), String>,
>;
struct RawAssetGeneratorDataUrlWrapper(RawAssetGeneratorDataUrl);

#[napi(object)]
pub struct RawAssetGeneratorDataUrlFnCtx {
  pub filename: String,
  #[napi(ts_type = "Module")]
  pub module: ModuleObject,
}

impl From<AssetGeneratorDataUrlFnCtx<'_>> for RawAssetGeneratorDataUrlFnCtx {
  fn from(value: AssetGeneratorDataUrlFnCtx) -> Self {
    Self {
      filename: value.filename,
      module: ModuleObject::with_ref(value.module, value.compilation.compiler_id()),
    }
  }
}

impl From<RawAssetGeneratorDataUrlWrapper> for AssetGeneratorDataUrl {
  fn from(value: RawAssetGeneratorDataUrlWrapper) -> Self {
    use pollster::block_on;
    match value.0 {
      Either::A(a) => Self::Options(a.into()),
      Either::B(b) => Self::Func(Arc::new(move |source, ctx| {
        block_on(b.call((source.into(), ctx.into())))
      })),
    }
  }
}

#[derive(Debug, Default)]
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

#[derive(Debug, Default)]
#[napi(object)]
pub struct RawCssGeneratorOptions {
  pub exports_only: Option<bool>,
  pub es_module: Option<bool>,
}

impl From<RawCssGeneratorOptions> for CssGeneratorOptions {
  fn from(value: RawCssGeneratorOptions) -> Self {
    Self {
      exports_only: value.exports_only,
      es_module: value.es_module,
    }
  }
}

#[derive(Debug, Default)]
#[napi(object)]
pub struct RawCssAutoGeneratorOptions {
  #[napi(ts_type = r#""as-is" | "camel-case" | "camel-case-only" | "dashes" | "dashes-only""#)]
  pub exports_convention: Option<String>,
  pub exports_only: Option<bool>,
  pub local_ident_name: Option<String>,
  pub es_module: Option<bool>,
}

impl From<RawCssAutoGeneratorOptions> for CssAutoGeneratorOptions {
  fn from(value: RawCssAutoGeneratorOptions) -> Self {
    Self {
      exports_convention: value.exports_convention.map(|n| n.into()),
      exports_only: value.exports_only,
      local_ident_name: value.local_ident_name.map(|n| n.into()),
      es_module: value.es_module,
    }
  }
}

#[derive(Debug, Default)]
#[napi(object)]
pub struct RawCssModuleGeneratorOptions {
  #[napi(ts_type = r#""as-is" | "camel-case" | "camel-case-only" | "dashes" | "dashes-only""#)]
  pub exports_convention: Option<String>,
  pub exports_only: Option<bool>,
  pub local_ident_name: Option<String>,
  pub es_module: Option<bool>,
}

impl From<RawCssModuleGeneratorOptions> for CssModuleGeneratorOptions {
  fn from(value: RawCssModuleGeneratorOptions) -> Self {
    Self {
      exports_convention: value.exports_convention.map(|n| n.into()),
      exports_only: value.exports_only,
      local_ident_name: value.local_ident_name.map(|n| n.into()),
      es_module: value.es_module,
    }
  }
}

#[napi(object, object_to_js = false)]
pub struct RawModuleOptions {
  pub rules: Vec<RawModuleRule>,
  pub parser: Option<HashMap<String, RawParserOptions>>,
  pub generator: Option<HashMap<String, RawGeneratorOptions>>,
  #[napi(
    ts_type = "string | RegExp | ((request: string) => boolean) | (string | RegExp | ((request: string) => boolean))[]"
  )]
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

#[derive(Debug, Clone)]
#[napi(object)]
pub struct RawFuncUseCtx {
  pub resource: Option<String>,
  pub real_resource: Option<String>,
  pub resource_query: String,
  pub resource_fragment: String,
  pub issuer: String,
  pub issuer_layer: String,
}

impl From<FuncUseCtx> for RawFuncUseCtx {
  fn from(value: FuncUseCtx) -> Self {
    Self {
      resource: value.resource,
      real_resource: value.real_resource,
      resource_query: value.resource_query.unwrap_or_default(),
      resource_fragment: value.resource_fragment.unwrap_or_default(),
      issuer: value.issuer.map(|s| s.to_string()).unwrap_or_default(),
      issuer_layer: value.issuer_layer.unwrap_or_default(),
    }
  }
}

impl TryFrom<RawModuleRule> for ModuleRule {
  type Error = rspack_error::Error;

  fn try_from(value: RawModuleRule) -> rspack_error::Result<Self> {
    let uses = value.r#use.map(|raw| match raw {
      Either::A(array) => {
        let uses = array
          .into_iter()
          .map(|rule_use| ModuleRuleUseLoader {
            loader: rule_use.loader,
            options: rule_use.options,
          })
          .collect::<Vec<_>>();
        Ok::<ModuleRuleUse, rspack_error::Error>(ModuleRuleUse::Array(uses))
      }
      Either::B(tsfn) => Ok::<ModuleRuleUse, rspack_error::Error>(ModuleRuleUse::Func(Box::new(
        move |ctx: FuncUseCtx| {
          let tsfn = tsfn.clone();
          Box::pin(async move {
            tsfn.call(ctx.into()).await.map(|uses| {
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
      ))),
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

    let with = value
      .with
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
      with,
      issuer: value.issuer.map(|raw| raw.try_into()).transpose()?,
      issuer_layer: value.issuer_layer.map(|raw| raw.try_into()).transpose()?,
      dependency: value.dependency.map(|raw| raw.try_into()).transpose()?,
      scheme: value.scheme.map(|raw| raw.try_into()).transpose()?,
      mimetype: value.mimetype.map(|raw| raw.try_into()).transpose()?,
      one_of,
      rules,
      effect: ModuleRuleEffect {
        r#use: uses.transpose()?.unwrap_or_default(),
        r#type: module_type,
        layer: value.layer,
        parser: value.parser.map(|raw| raw.into()),
        generator: value.generator.map(|raw| raw.into()),
        resolve: value.resolve.map(|raw| raw.try_into()).transpose()?,
        side_effects: value.side_effects,
        enforce,
      },
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
            .map(|(k, v)| Ok((k, v.into())))
            .collect::<std::result::Result<ParserOptionsMap, rspack_error::Error>>()
        })
        .transpose()?,
      generator: value
        .generator
        .map(|x| {
          x.into_iter()
            .map(|(k, v)| Ok((k, v.into())))
            .collect::<std::result::Result<GeneratorOptionsMap, rspack_error::Error>>()
        })
        .transpose()?,
      no_parse: value
        .no_parse
        .map(|x| RawModuleNoParseRulesWrapper(x).into()),
    })
  }
}

type RawModuleNoParseRule = Either3<String, RspackRegex, ThreadsafeFunction<String, Option<bool>>>;
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
      Either3::B(v) => Self::Regexp(v),
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
