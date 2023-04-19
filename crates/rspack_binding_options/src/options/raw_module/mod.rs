mod js_loader;

use std::{collections::HashMap, fmt::Debug, sync::Arc};

use js_loader::JsLoader;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use rspack_core::{
  AssetGeneratorOptions, AssetParserDataUrlOption, AssetParserOptions, BoxLoader, DescriptionData,
  ModuleOptions, ModuleRule, ParserOptions,
};
use rspack_error::internal_error;
use serde::Deserialize;
#[cfg(feature = "node-api")]
use {
  js_loader::JsLoaderAdapter,
  napi::NapiRaw,
  rspack_binding_macros::call_js_function_with_napi_objects,
  rspack_napi_shared::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode},
  rspack_napi_shared::{NapiResultExt, NAPI_ENV},
};

use crate::RawResolveOptions;

fn get_builtin_loader(builtin: &str, options: Option<&str>) -> BoxLoader {
  match builtin {
    "builtin:sass-loader" => Arc::new(rspack_loader_sass::SassLoader::new(
      serde_json::from_str(options.unwrap_or("{}")).unwrap_or_else(|e| {
        panic!("Could not parse builtin:sass-loader options: {options:?}, error: {e:?}")
      }),
    )),
    loader => panic!("{loader} is not supported yet."),
  }
}

/// `loader` is for js side loader, `builtin_loader` is for rust side loader,
/// which is mapped to real rust side loader by [get_builtin_loader].
///
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
  #[serde(skip_deserializing)]
  pub js_loader: Option<JsLoader>,
  pub builtin_loader: Option<String>,
  pub options: Option<String>,
}

impl Debug for RawModuleRuleUse {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("RawModuleRuleUse")
      .field("loader", &self.js_loader.as_ref().map(|i| &i.name))
      .field("builtin_loader", &self.builtin_loader)
      .field("options", &self.options)
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
      "regexp" => Self::Regexp(rspack_regex::RspackRegex::new(
        &x.regexp_matcher.ok_or_else(|| {
          internal_error!(
            "should have a regexp_matcher when RawRuleSetCondition.type is \"regexp\""
          )
        })?,
      )?),
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
      #[cfg(feature = "node-api")]
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
              rspack_binding_macros::js_fn_into_theadsafe_fn!(func_matcher, &Env::from(env));
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

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawModuleRule {
  /// A condition matcher matching an absolute path.
  pub test: Option<RawRuleSetCondition>,
  pub include: Option<RawRuleSetCondition>,
  pub exclude: Option<RawRuleSetCondition>,
  /// A condition matcher matching an absolute path.
  pub resource: Option<RawRuleSetCondition>,
  /// A condition matcher against the resource query.
  pub resource_query: Option<RawRuleSetCondition>,
  pub description_data: Option<HashMap<String, RawRuleSetCondition>>,
  pub side_effects: Option<bool>,
  pub r#use: Option<Vec<RawModuleRuleUse>>,
  pub r#type: Option<String>,
  pub parser: Option<RawModuleRuleParser>,
  pub generator: Option<RawModuleRuleGenerator>,
  pub resolve: Option<RawResolveOptions>,
  pub issuer: Option<RawRuleSetCondition>,
  pub dependency: Option<RawRuleSetCondition>,
  pub one_of: Option<Vec<RawModuleRule>>,
}

impl Debug for RawModuleRule {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("RawModuleRule")
      .field("test", &self.test)
      .field("include", &self.include)
      .field("exclude", &self.exclude)
      .field("resource", &self.resource)
      .field("resource_query", &self.resource_query)
      .field("type", &self.r#type)
      .field("side_effects", &self.side_effects)
      .field("use", &self.r#use)
      .field("issuer", &self.issuer)
      .field("one_of", &self.one_of)
      .finish()
  }
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawModuleRuleGenerator {
  pub filename: Option<String>,
}

impl From<RawModuleRuleGenerator> for AssetGeneratorOptions {
  fn from(value: RawModuleRuleGenerator) -> Self {
    Self {
      filename: value.filename.map(|i| i.into()),
    }
  }
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawModuleRuleParser {
  pub data_url_condition: Option<RawAssetParserDataUrlOption>,
}

impl From<RawModuleRuleParser> for AssetParserOptions {
  fn from(value: RawModuleRuleParser) -> Self {
    Self {
      data_url_condition: value.data_url_condition.map(|i| i.into()),
    }
  }
}

#[derive(Debug, Clone, Deserialize, Default)]
#[napi(object)]
#[serde(rename_all = "camelCase")]
pub struct RawAssetParserDataUrlOption {
  pub max_size: Option<u32>,
}

impl From<RawAssetParserDataUrlOption> for AssetParserDataUrlOption {
  fn from(value: RawAssetParserDataUrlOption) -> Self {
    Self {
      max_size: value.max_size,
    }
  }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawAssetParserOptions {
  pub data_url_condition: Option<RawAssetParserDataUrlOption>,
}

impl From<RawAssetParserOptions> for AssetParserOptions {
  fn from(value: RawAssetParserOptions) -> Self {
    Self {
      data_url_condition: value.data_url_condition.map(|i| i.into()),
    }
  }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawParserOptions {
  pub asset: Option<RawAssetParserOptions>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawModuleOptions {
  pub rules: Vec<RawModuleRule>,
  pub parser: Option<RawParserOptions>,
}

impl TryFrom<RawModuleRule> for ModuleRule {
  type Error = rspack_error::Error;

  fn try_from(value: RawModuleRule) -> std::result::Result<Self, Self::Error> {
    // Even this part is using the plural version of loader, it's recommended to use singular version from js side to reduce overhead (This behavior maybe changed later for advanced usage).
    let uses = value
      .r#use
      .map(|uses| {
        uses
          .into_iter()
          .map(|rule_use| {
            #[cfg(feature = "node-api")]
            {
              if let Some(raw_js_loader) = rule_use.js_loader {
                return JsLoaderAdapter::try_from(raw_js_loader).map(|i| Arc::new(i) as BoxLoader);
              }
            }
            if let Some(builtin_loader) = rule_use.builtin_loader {
              return Ok(get_builtin_loader(&builtin_loader, rule_use.options.as_deref()));
            }
            panic!("`loader` field or `builtin_loader` field in `use` must not be `None` at the same time.");
          })
          .collect::<anyhow::Result<Vec<_>>>()
      })
      .transpose()?
      .unwrap_or_default();

    let module_type = value.r#type.map(|t| (&*t).try_into()).transpose()?;

    let one_of = value
      .one_of
      .map(|one_of| {
        one_of
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

    Ok(ModuleRule {
      test: value.test.map(|raw| raw.try_into()).transpose()?,
      include: value.include.map(|raw| raw.try_into()).transpose()?,
      exclude: value.exclude.map(|raw| raw.try_into()).transpose()?,
      resource_query: value.resource_query.map(|raw| raw.try_into()).transpose()?,
      resource: value.resource.map(|raw| raw.try_into()).transpose()?,
      description_data,
      r#use: uses,
      r#type: module_type,
      parser: value.parser.map(|raw| raw.into()),
      generator: value.generator.map(|raw| raw.into()),
      resolve: value.resolve.map(|raw| raw.try_into()).transpose()?,
      side_effects: value.side_effects,
      issuer: value.issuer.map(|raw| raw.try_into()).transpose()?,
      dependency: value.dependency.map(|raw| raw.try_into()).transpose()?,
      one_of,
    })
  }
}

impl TryFrom<RawModuleOptions> for ModuleOptions {
  type Error = rspack_error::Error;

  fn try_from(value: RawModuleOptions) -> std::result::Result<Self, Self::Error> {
    // FIXME: temporary implementation
    let rules = value
      .rules
      .into_iter()
      .map(|rule| rule.try_into())
      .collect::<rspack_error::Result<Vec<ModuleRule>>>()?;
    Ok(ModuleOptions {
      rules,
      parser: value.parser.map(|x| ParserOptions {
        asset: x.asset.map(|y| y.into()),
      }),
    })
  }
}
