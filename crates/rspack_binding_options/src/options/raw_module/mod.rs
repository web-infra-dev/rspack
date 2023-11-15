mod js_loader;

use std::{collections::HashMap, fmt::Debug, sync::Arc};

use derivative::Derivative;
pub use js_loader::JsLoaderAdapter;
pub use js_loader::*;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use rspack_core::{
  AssetGeneratorDataUrl, AssetGeneratorDataUrlOptions, AssetGeneratorOptions,
  AssetInlineGeneratorOptions, AssetParserDataUrl, AssetParserDataUrlOptions, AssetParserOptions,
  AssetResourceGeneratorOptions, BoxLoader, DescriptionData, DynamicImportMode, FuncUseCtx,
  GeneratorOptions, GeneratorOptionsByModuleType, JavascriptParserOptions, ModuleOptions,
  ModuleRule, ModuleRuleEnforce, ModuleRuleUse, ModuleRuleUseLoader, ModuleType, ParserOptions,
  ParserOptionsByModuleType,
};
use rspack_error::internal_error;
use rspack_loader_react_refresh::REACT_REFRESH_LOADER_IDENTIFIER;
use rspack_loader_sass::SASS_LOADER_IDENTIFIER;
use rspack_loader_swc::SWC_LOADER_IDENTIFIER;
use serde::Deserialize;
use {
  rspack_napi_shared::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode},
  rspack_napi_shared::{NapiResultExt, NAPI_ENV},
};

use crate::{RawOptionsApply, RawResolveOptions};

pub fn get_builtin_loader(builtin: &str, options: Option<&str>) -> BoxLoader {
  if builtin.starts_with(SASS_LOADER_IDENTIFIER) {
    return Arc::new(rspack_loader_sass::SassLoader::new(
      serde_json::from_str(options.unwrap_or("{}")).unwrap_or_else(|e| {
        panic!("Could not parse builtin:sass-loader options: {options:?}, error: {e:?}")
      }),
    ));
  }

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
#[napi(object)]
pub struct RawModuleRuleUses {
  #[napi(ts_type = r#""array" | "function""#)]
  pub r#type: String,
  pub array_use: Option<Vec<RawModuleRuleUse>>,
  #[serde(skip_deserializing)]
  pub func_use: Option<JsFunction>,
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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawRuleSetCondition {
  #[napi(ts_type = r#""string" | "regexp" | "logical" | "array" | "function""#)]
  pub r#type: String,
  pub string_matcher: Option<String>,
  pub regexp_matcher: Option<String>,
  pub logical_matcher: Option<Vec<RawRuleSetLogicalConditions>>,
  pub array_matcher: Option<Vec<RawRuleSetCondition>>,
  #[serde(skip_deserializing)]
  #[napi(ts_type = r#"(value: string) => boolean"#)]
  pub func_matcher: Option<JsFunction>,
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
#[napi(object)]
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
        internal_error!("should have a string_matcher when RawRuleSetCondition.type is \"string\"")
      })?),
      "regexp" => {
        let reg = rspack_regex::RspackRegex::new(
            x.regexp_matcher.as_ref().ok_or_else(|| {
              internal_error!(
                "should have a regexp_matcher when RawRuleSetCondition.type is \"regexp\""
              )
            })?,
        )?;
        tracing::debug!(regex_matcher = ?x.regexp_matcher, algo_type = ?reg.algo);
        Self::Regexp(reg)
      },
      "logical" => {
        let mut logical_matcher = x.logical_matcher.ok_or_else(|| {
          internal_error!(
            "should have a logical_matcher when RawRuleSetCondition.type is \"logical\""
          )
        })?;
        let logical_matcher = logical_matcher.get_mut(0).ok_or_else(|| {
          internal_error!(
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
            internal_error!(
              "should have a array_matcher when RawRuleSetCondition.type is \"array\""
            )
          })?
          .into_iter()
          .map(|i| i.try_into())
          .collect::<rspack_error::Result<Vec<_>>>()?,
      ),
      "function" => {
        let func_matcher = x.func_matcher.ok_or_else(|| {
          internal_error!(
            "should have a func_matcher when RawRuleSetCondition.type is \"function\""
          )
        })?;
        let func_matcher: ThreadsafeFunction<String, bool> =
          NAPI_ENV.with(|env| -> anyhow::Result<_> {
            let env = env
              .borrow()
              .expect("Failed to get env, did you forget to call it from node?");
            let func_matcher =
              rspack_binding_macros::js_fn_into_threadsafe_fn!(func_matcher, &Env::from(env));
            Ok(func_matcher)
          })?;
        let func_matcher = Arc::new(func_matcher);

        Self::Func(Box::new(move |data: &str| {
          let func_matcher = func_matcher.clone();
          let data = data.to_string();
          Box::pin(async move {
            func_matcher
              .call(data, ThreadsafeFunctionCallMode::NonBlocking)
              .into_rspack_result()?
              .await
              .map_err(|err| {
                internal_error!("Failed to call RuleSetCondition func_matcher: {err}")
              })?
          })
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
#[napi(object)]
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
  #[napi(ts_type = r#""asset" | "javascript" | "unknown""#)]
  pub r#type: String,
  pub asset: Option<RawAssetParserOptions>,
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
        value.javascript.expect("should have an \"javascript\" when RawParserOptions.type is \"javascript\"").into()
      ),
      "unknown" => Self::Unknown,
      _ => panic!(
        "Failed to resolve the RawParserOptions.type {}. Expected type is \"asset\", \"javascript\",  \"unknown\".",
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
}

impl From<RawJavascriptParserOptions> for JavascriptParserOptions {
  fn from(value: RawJavascriptParserOptions) -> Self {
    Self {
      dynamic_import_mode: DynamicImportMode::from(value.dynamic_import_mode.as_str()),
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
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawGeneratorOptions {
  #[napi(ts_type = r#""asset" | "asset/inline" | "asset/resource" | "unknown""#)]
  pub r#type: String,
  pub asset: Option<RawAssetGeneratorOptions>,
  pub asset_inline: Option<RawAssetInlineGeneratorOptions>,
  pub asset_resource: Option<RawAssetResourceGeneratorOptions>,
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
      "unknown" => Self::Unknown,
      _ => panic!(
        "Failed to resolve the RawGeneratorOptions.type {}. Expected type is \"asset\", \"asset/inline\", \"asset/resource\", \"unknown\".",
        value.r#type
      ),
    }
  }
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawAssetGeneratorOptions {
  pub filename: Option<String>,
  pub public_path: Option<String>,
  pub data_url: Option<RawAssetGeneratorDataUrl>,
}

impl From<RawAssetGeneratorOptions> for AssetGeneratorOptions {
  fn from(value: RawAssetGeneratorOptions) -> Self {
    Self {
      filename: value.filename.map(|i| i.into()),
      public_path: value.public_path.map(|i| i.into()),
      data_url: value.data_url.map(|i| i.into()),
    }
  }
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawAssetInlineGeneratorOptions {
  pub data_url: Option<RawAssetGeneratorDataUrl>,
}

impl From<RawAssetInlineGeneratorOptions> for AssetInlineGeneratorOptions {
  fn from(value: RawAssetInlineGeneratorOptions) -> Self {
    Self {
      data_url: value.data_url.map(|i| i.into()),
    }
  }
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawAssetResourceGeneratorOptions {
  pub filename: Option<String>,
  pub public_path: Option<String>,
}

impl From<RawAssetResourceGeneratorOptions> for AssetResourceGeneratorOptions {
  fn from(value: RawAssetResourceGeneratorOptions) -> Self {
    Self {
      filename: value.filename.map(|i| i.into()),
      public_path: value.public_path.map(|i| i.into()),
    }
  }
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawAssetGeneratorDataUrl {
  #[napi(ts_type = r#""options""#)]
  pub r#type: String,
  pub options: Option<RawAssetGeneratorDataUrlOptions>,
  // TODO: pub function
}

impl From<RawAssetGeneratorDataUrl> for AssetGeneratorDataUrl {
  fn from(value: RawAssetGeneratorDataUrl) -> Self {
    match value.r#type.as_str() {
      "options" => Self::Options(
        value
          .options
          .expect("should have an \"options\" when RawAssetGeneratorDataUrl.type is \"options\"")
          .into(),
      ),
      _ => panic!(
        "Failed to resolve the RawAssetGeneratorDataUrl.type {}. Expected type is `options`.",
        value.r#type
      ),
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawModuleOptions {
  pub rules: Vec<RawModuleRule>,
  pub parser: Option<HashMap<String, RawParserOptions>>,
  pub generator: Option<HashMap<String, RawGeneratorOptions>>,
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

impl RawOptionsApply for RawModuleRule {
  type Options = ModuleRule;

  fn apply(
    self,
    _plugins: &mut Vec<rspack_core::BoxPlugin>,
  ) -> std::result::Result<Self::Options, rspack_error::Error> {
    // Even this part is using the plural version of loader, it's recommended to use singular version from js side to reduce overhead (This behavior maybe changed later for advanced usage).
    let uses = self.r#use.map(|raw| match raw.r#type.as_str() {
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
          internal_error!(
            "should have a func_matcher when RawRuleSetCondition.type is \"function\""
          )
        })?;
        let func_use: ThreadsafeFunction<RawFuncUseCtx, Vec<RawModuleRuleUse>> =
          NAPI_ENV.with(|env| -> anyhow::Result<_> {
            let env = env.borrow().expect("Failed to get env with external");
            let func_use =
              rspack_binding_macros::js_fn_into_threadsafe_fn!(func_use, &Env::from(env));
            Ok(func_use)
          })?;
        let func_use = Arc::new(func_use);
        Ok::<ModuleRuleUse, rspack_error::Error>(ModuleRuleUse::Func(Box::new(
          move |ctx: FuncUseCtx| {
            let func_use = func_use.clone();
            Box::pin(async move {
              func_use
                .call(ctx.into(), ThreadsafeFunctionCallMode::NonBlocking)
                .into_rspack_result()?
                .await
                .map_err(|err| internal_error!("Failed to call rule.use function: {err}"))?
                .map(|uses| {
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

    let module_type = self.r#type.map(|t| (&*t).try_into()).transpose()?;

    let one_of = self
      .one_of
      .map(|one_of| {
        one_of
          .into_iter()
          .map(|raw| raw.apply(_plugins))
          .collect::<rspack_error::Result<Vec<_>>>()
      })
      .transpose()?;

    let rules = self
      .rules
      .map(|rule| {
        rule
          .into_iter()
          .map(|raw| raw.apply(_plugins))
          .collect::<rspack_error::Result<Vec<_>>>()
      })
      .transpose()?;

    let description_data = self
      .description_data
      .map(|data| {
        data
          .into_iter()
          .map(|(k, v)| Ok((k, v.try_into()?)))
          .collect::<rspack_error::Result<DescriptionData>>()
      })
      .transpose()?;

    let enforce = self
      .enforce
      .map(|enforce| match &*enforce {
        "pre" => Ok(ModuleRuleEnforce::Pre),
        "post" => Ok(ModuleRuleEnforce::Post),
        _ => Err(internal_error!(
          "Unsupported Rule.enforce type, supported: 'pre' | 'post' | undefined"
        )),
      })
      .transpose()?
      .unwrap_or_default();

    Ok(ModuleRule {
      rspack_resource: self.rspack_resource.map(|raw| raw.try_into()).transpose()?,
      test: self.test.map(|raw| raw.try_into()).transpose()?,
      include: self.include.map(|raw| raw.try_into()).transpose()?,
      exclude: self.exclude.map(|raw| raw.try_into()).transpose()?,
      resource_query: self.resource_query.map(|raw| raw.try_into()).transpose()?,
      resource_fragment: self
        .resource_fragment
        .map(|raw| raw.try_into())
        .transpose()?,
      resource: self.resource.map(|raw| raw.try_into()).transpose()?,
      description_data,
      r#use: uses.transpose()?.unwrap_or_default(),
      r#type: module_type,
      parser: self.parser.map(|raw| raw.into()),
      generator: self.generator.map(|raw| raw.into()),
      resolve: self.resolve.map(|raw| raw.try_into()).transpose()?,
      side_effects: self.side_effects,
      issuer: self.issuer.map(|raw| raw.try_into()).transpose()?,
      dependency: self.dependency.map(|raw| raw.try_into()).transpose()?,
      scheme: self.scheme.map(|raw| raw.try_into()).transpose()?,
      mimetype: self.mimetype.map(|raw| raw.try_into()).transpose()?,
      one_of,
      rules,
      enforce,
    })
  }
}

impl RawOptionsApply for RawModuleOptions {
  type Options = ModuleOptions;

  fn apply(
    self,
    plugins: &mut Vec<rspack_core::BoxPlugin>,
  ) -> std::result::Result<Self::Options, rspack_error::Error> {
    let rules = self
      .rules
      .into_iter()
      .map(|rule| rule.apply(plugins))
      .collect::<rspack_error::Result<Vec<ModuleRule>>>()?;
    Ok(ModuleOptions {
      rules,
      parser: self
        .parser
        .map(|x| {
          x.into_iter()
            .map(|(k, v)| Ok((ModuleType::try_from(k.as_str())?, v.into())))
            .collect::<std::result::Result<ParserOptionsByModuleType, rspack_error::Error>>()
        })
        .transpose()?,
      generator: self
        .generator
        .map(|x| {
          x.into_iter()
            .map(|(k, v)| Ok((ModuleType::try_from(k.as_str())?, v.into())))
            .collect::<std::result::Result<GeneratorOptionsByModuleType, rspack_error::Error>>()
        })
        .transpose()?,
    })
  }
}
