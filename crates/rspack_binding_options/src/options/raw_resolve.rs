use std::{collections::HashMap, path::PathBuf};

use napi_derive::napi;
use rspack_core::{Alias, AliasMap, ByDependency, Resolve, TsconfigOptions, TsconfigReferences};
use rspack_error::error;
use serde::Deserialize;

pub type AliasValue = serde_json::Value;

type RawAliasOption = HashMap<String, Vec<AliasValue>>;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawResolveTsconfigOptions {
  pub config_file: String,
  #[napi(ts_type = r#""auto" | "manual" | "disabled""#)]
  pub references_type: String,
  pub references: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawResolveOptions {
  pub prefer_relative: Option<bool>,
  pub extensions: Option<Vec<String>>,
  pub main_files: Option<Vec<String>>,
  pub main_fields: Option<Vec<String>>,
  pub browser_field: Option<bool>,
  pub condition_names: Option<Vec<String>>,
  #[serde(serialize_with = "ordered_map")]
  #[napi(ts_type = "Record<string, Array<string | false>>")]
  pub alias: Option<RawAliasOption>,
  #[serde(serialize_with = "ordered_map")]
  #[napi(ts_type = "Record<string, Array<string | false>>")]
  pub fallback: Option<RawAliasOption>,
  pub symlinks: Option<bool>,
  pub tsconfig: Option<RawResolveTsconfigOptions>,
  pub modules: Option<Vec<String>>,
  pub by_dependency: Option<HashMap<String, RawResolveOptions>>,
  pub fully_specified: Option<bool>,
  pub exports_fields: Option<Vec<String>>,
  #[serde(serialize_with = "ordered_map")]
  #[napi(ts_type = "Record<string, Array<string>>")]
  pub extension_alias: Option<HashMap<String, Vec<String>>>,
}

fn normalize_alias(alias: Option<RawAliasOption>) -> rspack_error::Result<Option<Alias>> {
  alias
    .map(|alias| {
      alias
        .into_iter()
        .map(|(key, array)| {
          array
            .into_iter()
            .map(|value| {
              if let Some(s) = value.as_str() {
                Ok(AliasMap::Target(s.to_string()))
              } else if let Some(b) = value.as_bool() {
                if b {
                  Err(error!("Alias should not be true in {key}"))
                } else {
                  Ok(AliasMap::Ignored)
                }
              } else {
                Err(error!("Alias should be false or string in {key}"))
              }
            })
            .collect::<rspack_error::Result<_>>()
            .map(|value| (key, value))
        })
        .collect::<rspack_error::Result<_>>()
    })
    .map_or(Ok(None), |v| v.map(Some))
}

impl TryFrom<RawResolveOptions> for Resolve {
  type Error = rspack_error::Error;

  fn try_from(value: RawResolveOptions) -> Result<Self, Self::Error> {
    let prefer_relative = value.prefer_relative;
    let extensions = value.extensions;
    let browser_field = value.browser_field;
    let main_files = value.main_files;
    let main_fields = value.main_fields;
    let condition_names = value.condition_names;
    let symlinks = value.symlinks;
    let fully_specified = value.fully_specified;
    let alias = normalize_alias(value.alias)?;
    let fallback = normalize_alias(value.fallback)?;
    let modules = value.modules;
    let tsconfig = match value.tsconfig {
      Some(config) => Some(TsconfigOptions::try_from(config)?),
      None => None,
    };
    let by_dependency = value
      .by_dependency
      .map(|i| {
        i.into_iter()
          .map(|(k, v)| Ok((k.into(), v.try_into()?)))
          .collect::<Result<ByDependency, Self::Error>>()
      })
      .transpose()?;
    let exports_field = value
      .exports_fields
      .map(|v| v.into_iter().map(|s| vec![s]).collect());
    let extension_alias = value.extension_alias.map(|v| v.into_iter().collect());
    Ok(Resolve {
      modules,
      prefer_relative,
      extensions,
      browser_field,
      main_fields,
      main_files,
      condition_names,
      alias,
      symlinks,
      tsconfig,
      fallback,
      by_dependency,
      fully_specified,
      exports_field,
      extension_alias,
    })
  }
}

impl TryFrom<RawResolveTsconfigOptions> for TsconfigOptions {
  type Error = rspack_error::Error;
  fn try_from(value: RawResolveTsconfigOptions) -> Result<Self, Self::Error> {
    let references = match value.references_type.as_str() {
      "manual" => TsconfigReferences::Paths(value.references.unwrap_or_default().into_iter().map(PathBuf::from).collect()),
      "auto" => TsconfigReferences::Auto,
      "disabled" => TsconfigReferences::Disabled,
      _ => panic!(
          "Failed to resolve the references type {}. Expected type is `auto`, `manual` or `disabled`.",
          value.references_type
      )
  };
    Ok(TsconfigOptions {
      config_file: PathBuf::from(value.config_file),
      references,
    })
  }
}
