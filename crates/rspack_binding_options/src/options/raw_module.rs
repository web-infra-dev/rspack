use std::fmt::Debug;

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rspack_core::{
  AssetGeneratorOptions, AssetParserDataUrlOption, AssetParserOptions, BoxedLoader,
  CompilerOptionsBuilder, IssuerOptions, ModuleOptions, ModuleRule, ParserOptions,
};
use serde::Deserialize;
#[cfg(feature = "node-api")]
use {
  crate::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode},
  napi::NapiRaw,
  rspack_binding_macros::call_js_function_with_napi_objects,
  rspack_error::{internal_error, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray},
  rspack_napi_utils::NapiResultIntoRspackResult,
};

use crate::{RawOption, RawResolveOptions};

#[cfg(feature = "node-api")]
type JsLoader<R> = ThreadsafeFunction<JsLoaderContext, R>;

fn get_builtin_loader(builtin: &str, options: Option<&str>) -> BoxedLoader {
  match builtin {
    "sass-loader" => Box::new(rspack_loader_sass::SassLoader::new(
      serde_json::from_str(options.unwrap_or("{}")).unwrap_or_else(|e| {
        panic!("Could not parse sass-loader options: {options:?}, error: {e:?}")
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
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawModuleRuleUse {
  #[serde(skip_deserializing)]
  pub loader: Option<JsFunction>,
  pub builtin_loader: Option<String>,
  pub options: Option<String>,
  pub __loader_name: Option<String>,
}

impl Debug for RawModuleRuleUse {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("RawModuleRuleUse")
      .field("loader", &self.__loader_name)
      .field("builtin_loader", &self.builtin_loader)
      .field("options", &self.options)
      .finish()
  }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawModuleRuleCondition {
  /// Condition can be either a `string` or `Regexp`.
  #[napi(ts_type = r#""string" | "regexp""#)]
  pub r#type: String,
  /// Based on the condition type, the value can be either a `string` or `Regexp`.
  ///  - "string": The value will be matched against the string.
  ///  - "regexp": The value will be matched against the raw regexp source from JS side.
  pub matcher: Option<String>,
}

impl TryFrom<RawModuleRuleCondition> for rspack_core::ModuleRuleCondition {
  type Error = anyhow::Error;

  fn try_from(x: RawModuleRuleCondition) -> std::result::Result<Self, Self::Error> {
    let matcher = x
      .matcher
      .ok_or_else(|| anyhow::anyhow!("Matcher is required."))?;

    let result = match x.r#type.as_str() {
      "string" => Self::String(matcher),
      "regexp" => Self::Regexp(rspack_regex::RspackRegex::new(&matcher)?),
      _ => {
        anyhow::bail!(
          "Failed to resolve the condition type {}. Expected type is either `string` or `regexp`.",
          x.r#type
        );
      }
    };

    Ok(result)
  }
}

#[derive(Deserialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawIssuerOptions {
  pub not: Option<Vec<RawModuleRuleCondition>>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawModuleRule {
  /// A condition matcher matching an absolute path.
  /// - String: To match the input must start with the provided string. I. e. an absolute directory path, or absolute path to the file.
  /// - Regexp: It's tested with the input.
  pub test: Option<RawModuleRuleCondition>,
  pub include: Option<Vec<RawModuleRuleCondition>>,
  pub exclude: Option<Vec<RawModuleRuleCondition>>,
  /// A condition matcher matching an absolute path.
  /// See `test` above
  pub resource: Option<RawModuleRuleCondition>,
  /// A condition matcher against the resource query.
  /// TODO: align with webpack's `?` prefixed `resourceQuery`
  pub resource_query: Option<RawModuleRuleCondition>,
  pub side_effects: Option<bool>,
  pub r#use: Option<Vec<RawModuleRuleUse>>,
  #[napi(
    ts_type = r#""js" | "jsx" | "ts" | "tsx" | "css" | "json" | "asset" | "asset/resource" | "asset/source" | "asset/inline""#
  )]
  pub r#type: Option<String>,
  pub parser: Option<RawModuleRuleParser>,
  pub generator: Option<RawModuleRuleGenerator>,
  pub resolve: Option<RawResolveOptions>,
  pub issuer: Option<RawIssuerOptions>,
  // Loader experimental
  #[serde(skip_deserializing)]
  pub func__: Option<JsFunction>,
}

impl RawOption<AssetParserOptions> for RawModuleRuleParser {
  fn to_compiler_option(
    self,
    options: &CompilerOptionsBuilder,
  ) -> anyhow::Result<AssetParserOptions> {
    Ok(AssetParserOptions {
      data_url_condition: self
        .data_url_condition
        .map(|d| d.to_compiler_option(options))
        .transpose()?,
    })
  }

  fn fallback_value(_options: &CompilerOptionsBuilder) -> Self {
    Self::default()
  }
}

impl RawOption<AssetParserDataUrlOption> for RawAssetParserDataUrlOption {
  fn to_compiler_option(
    self,
    _options: &CompilerOptionsBuilder,
  ) -> anyhow::Result<AssetParserDataUrlOption> {
    Ok(AssetParserDataUrlOption {
      max_size: self.max_size,
    })
  }

  fn fallback_value(_options: &CompilerOptionsBuilder) -> Self {
    Self::default()
  }
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawModuleRuleGenerator {
  pub filename: Option<String>,
}

impl RawOption<AssetGeneratorOptions> for RawModuleRuleGenerator {
  fn to_compiler_option(
    self,
    _options: &CompilerOptionsBuilder,
  ) -> anyhow::Result<AssetGeneratorOptions> {
    Ok(AssetGeneratorOptions {
      filename: self.filename.map(Into::into),
    })
  }

  fn fallback_value(_options: &CompilerOptionsBuilder) -> Self {
    Self::default()
  }
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawModuleRuleParser {
  pub data_url_condition: Option<RawAssetParserDataUrlOption>,
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
      .finish()
  }
}

#[derive(Debug, Clone, Deserialize, Default)]
#[napi(object)]
#[serde(rename_all = "camelCase")]
pub struct RawAssetParserDataUrlOption {
  pub max_size: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawAssetParserOptions {
  pub data_url_condition: Option<RawAssetParserDataUrlOption>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawParserOptions {
  pub asset: Option<RawAssetParserOptions>,
}

#[derive(Default, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawModuleOptions {
  pub rules: Vec<RawModuleRule>,
  pub parser: Option<RawParserOptions>,
}

#[cfg(feature = "node-api")]
pub struct JsLoaderAdapter {
  pub loader: JsLoader<LoaderThreadsafeLoaderResult>,
  pub name: String,
}

#[cfg(feature = "node-api")]
impl JsLoaderAdapter {
  pub fn unref(&mut self, env: &napi::Env) -> anyhow::Result<()> {
    self
      .loader
      .unref(env)
      .map_err(|e| anyhow::format_err!("failed to unref tsfn: {}", e))
  }
}

#[cfg(feature = "node-api")]
impl Debug for JsLoaderAdapter {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("JsLoaderAdapter")
      .field("loaders", &self.name)
      .finish()
  }
}

#[cfg(feature = "node-api")]
#[async_trait::async_trait]
impl rspack_core::Loader<rspack_core::CompilerContext, rspack_core::CompilationContext>
  for JsLoaderAdapter
{
  fn name(&self) -> &'static str {
    "js_loader_adapter"
  }

  async fn run(
    &self,
    loader_context: &rspack_core::LoaderContext<
      '_,
      '_,
      rspack_core::CompilerContext,
      rspack_core::CompilationContext,
    >,
  ) -> Result<Option<TWithDiagnosticArray<rspack_core::LoaderResult>>> {
    let loader_context = JsLoaderContext {
      content: loader_context.source.to_owned().into_bytes().into(),
      additional_data: loader_context
        .additional_data
        .to_owned()
        .map(|v| v.into_bytes().into()),
      source_map: loader_context
        .source_map
        .clone()
        .map(|v| v.to_json())
        .transpose()
        .map_err(|e| internal_error!(e.to_string()))?
        .map(|v| v.into_bytes().into()),
      resource: loader_context.resource.to_owned(),
      resource_path: loader_context.resource_path.to_string_lossy().to_string(),
      resource_fragment: loader_context.resource_fragment.map(|r| r.to_owned()),
      resource_query: loader_context.resource_query.map(|r| r.to_owned()),
      cacheable: loader_context.cacheable,
      file_dependencies: loader_context
        .file_dependencies
        .iter()
        .map(|i| i.to_string_lossy().to_string())
        .collect(),
      context_dependencies: loader_context
        .context_dependencies
        .iter()
        .map(|i| i.to_string_lossy().to_string())
        .collect(),
      missing_dependencies: loader_context
        .missing_dependencies
        .iter()
        .map(|i| i.to_string_lossy().to_string())
        .collect(),
      build_dependencies: loader_context
        .build_dependencies
        .iter()
        .map(|i| i.to_string_lossy().to_string())
        .collect(),
    };

    let loader_result = self
      .loader
      .call(loader_context, ThreadsafeFunctionCallMode::NonBlocking)
      .into_rspack_result()?
      .await
      .map_err(|err| internal_error!("Failed to call loader: {err}"))??;

    let source_map = loader_result
      .as_ref()
      .and_then(|r| r.source_map.as_ref())
      .map(|s| rspack_core::rspack_sources::SourceMap::from_slice(s))
      .transpose()
      .map_err(|e| internal_error!(e.to_string()))?;

    Ok(loader_result.map(|loader_result| {
      rspack_core::LoaderResult {
        cacheable: loader_result.cacheable,
        file_dependencies: loader_result
          .file_dependencies
          .into_iter()
          .map(std::path::PathBuf::from)
          .collect(),
        context_dependencies: loader_result
          .context_dependencies
          .into_iter()
          .map(std::path::PathBuf::from)
          .collect(),
        missing_dependencies: loader_result
          .missing_dependencies
          .into_iter()
          .map(std::path::PathBuf::from)
          .collect(),
        build_dependencies: loader_result
          .build_dependencies
          .into_iter()
          .map(std::path::PathBuf::from)
          .collect(),
        content: rspack_core::Content::from(Into::<Vec<u8>>::into(loader_result.content)),
        source_map,
        additional_data: loader_result
          .additional_data
          .map(|item| String::from_utf8_lossy(&item).to_string()),
      }
      .with_empty_diagnostic()
    }))
  }

  fn as_any(&self) -> &dyn std::any::Any {
    self
  }

  fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
    self
  }
}

#[cfg(feature = "node-api")]
#[napi(object)]
pub struct JsLoaderContext {
  pub content: Buffer,
  pub additional_data: Option<Buffer>,
  pub source_map: Option<Buffer>,
  pub resource: String,
  pub resource_path: String,
  pub resource_query: Option<String>,
  pub resource_fragment: Option<String>,
  pub cacheable: bool,
  pub file_dependencies: Vec<String>,
  pub context_dependencies: Vec<String>,
  pub missing_dependencies: Vec<String>,
  pub build_dependencies: Vec<String>,
}

#[cfg(feature = "node-api")]
#[napi(object)]
pub struct JsLoaderResult {
  pub content: Buffer,
  pub file_dependencies: Vec<String>,
  pub context_dependencies: Vec<String>,
  pub missing_dependencies: Vec<String>,
  pub build_dependencies: Vec<String>,
  pub source_map: Option<Buffer>,
  pub additional_data: Option<Buffer>,
  pub cacheable: bool,
}

#[cfg(feature = "node-api")]
pub type LoaderThreadsafeLoaderResult = Option<JsLoaderResult>;

impl RawOption<ModuleRule> for RawModuleRule {
  fn to_compiler_option(self, options: &CompilerOptionsBuilder) -> anyhow::Result<ModuleRule> {
    // Even this part is using the plural version of loader, it's recommended to use singular version from js side to reduce overhead (This behavior maybe changed later for advanced usage).
    let uses = self
      .r#use
      .map(|uses| {
        uses
          .into_iter()
          .map(|rule_use| {
            #[cfg(feature = "node-api")]
            {
              if let Some(raw_js_loader) = rule_use.loader {
                let js_loader = unsafe { raw_js_loader.raw() };


                let loader = crate::NAPI_ENV.with(|env| {
                  let env = env.borrow().expect("Failed to get env, did you forget to call it from node?");
                    ThreadsafeFunction::<JsLoaderContext,LoaderThreadsafeLoaderResult>::create(
                    env,
                    js_loader,
                    0,
                    |ctx| {
                      let (ctx, resolver) = ctx.split_into_parts();

                      let env = ctx.env;
                      let cb = ctx.callback;
                      let resource = ctx.value.resource.clone();

                      let result = tracing::span!(tracing::Level::INFO, "loader_sync_call", resource = &resource).in_scope(|| {
                        unsafe { call_js_function_with_napi_objects!(env, cb, ctx.value) }
                      })?;

                      let resolve_start = std::time::Instant::now();
                      resolver.resolve::<Option<JsLoaderResult>>(result, move |r| {
                        tracing::trace!("Finish resolving loader result for {}, took {}ms", resource, resolve_start.elapsed().as_millis());
                        Ok(r)
                      })
                    })
                  })?;
                return Ok(Box::new(JsLoaderAdapter {
                    loader,
                    name: rule_use.__loader_name.unwrap_or_else(|| "unknown-loaders".to_owned())
                  }) as BoxedLoader);
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

    let module_type = self.r#type.map(|t| (&*t).try_into()).transpose()?;

    let issuer = if let Some(issuer) = self.issuer {
      Some(IssuerOptions {
        not: issuer
          .not
          .map(|raw| raw.into_iter().map(|f| f.try_into()).collect())
          .transpose()?,
      })
    } else {
      None
    };

    Ok(ModuleRule {
      test: self.test.map(|raw| raw.try_into()).transpose()?,
      include: self
        .include
        .map(|raw| raw.into_iter().map(|f| f.try_into()).collect())
        .transpose()?,
      exclude: self
        .exclude
        .map(|raw| raw.into_iter().map(|f| f.try_into()).collect())
        .transpose()?,
      resource_query: self.resource_query.map(|raw| raw.try_into()).transpose()?,
      resource: self.resource.map(|raw| raw.try_into()).transpose()?,
      r#use: uses,
      r#type: module_type,
      parser: self
        .parser
        .map(|raw| raw.to_compiler_option(options))
        .transpose()?,
      generator: self
        .generator
        .map(|raw| raw.to_compiler_option(options))
        .transpose()?,
      resolve: self
        .resolve
        .map(|raw| raw.to_compiler_option(options))
        .transpose()?,
      side_effects: self.side_effects,
      issuer,
      // side_effects: raw.
      // Loader experimental
      func__: None,
    })
  }

  fn fallback_value(_options: &CompilerOptionsBuilder) -> Self {
    RawModuleRule::default()
  }
}

impl RawOption<Option<ModuleOptions>> for RawModuleOptions {
  fn to_compiler_option(
    self,
    options: &CompilerOptionsBuilder,
  ) -> anyhow::Result<Option<ModuleOptions>> {
    // FIXME: temporary implementation
    let rules = self
      .rules
      .into_iter()
      .map(|rule| {
        rule
          .to_compiler_option(options)
          .map_err(|err| anyhow::format_err!("failed to convert rule: {}", err))
      })
      .collect::<anyhow::Result<Vec<ModuleRule>>>()?;
    Ok(Some(ModuleOptions {
      rules,
      parser: self.parser.map(|x| ParserOptions {
        asset: x.asset.map(|y| AssetParserOptions {
          data_url_condition: y.data_url_condition.map(|a| AssetParserDataUrlOption {
            max_size: a.max_size,
          }),
        }),
      }),
    }))
  }

  fn fallback_value(_options: &CompilerOptionsBuilder) -> Self {
    RawModuleOptions {
      rules: vec![],
      parser: None,
    }
  }
}
