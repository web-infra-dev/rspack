use std::{collections::HashMap, path::PathBuf};

use napi_derive::napi;
use rspack_core::{Alias, AliasMap, ByDependency, Resolve, TsconfigOptions, TsconfigReferences};
use rspack_error::error;

pub type AliasValue = serde_json::Value;

#[derive(Debug)]
#[napi(object)]
pub struct RawAliasOptionItem {
  pub path: String,
  #[napi(ts_type = "Array<string | false>")]
  pub redirect: Vec<AliasValue>,
}

#[derive(Debug)]
#[napi(object)]
pub struct RawResolveTsconfigOptions {
  pub config_file: String,
  #[napi(ts_type = r#""auto" | "manual" | "disabled""#)]
  pub references_type: String,
  pub references: Option<Vec<String>>,
}

#[derive(Debug)]
#[napi(object)]
pub struct RawResolveOptions {
  pub prefer_relative: Option<bool>,
  pub prefer_absolute: Option<bool>,
  pub extensions: Option<Vec<String>>,
  pub main_files: Option<Vec<String>>,
  pub main_fields: Option<Vec<String>>,
  pub condition_names: Option<Vec<String>>,
  pub alias: Option<Vec<RawAliasOptionItem>>,
  pub fallback: Option<Vec<RawAliasOptionItem>>,
  pub symlinks: Option<bool>,
  pub tsconfig: Option<RawResolveTsconfigOptions>,
  pub modules: Option<Vec<String>>,
  pub by_dependency: Option<HashMap<String, RawResolveOptions>>,
  pub fully_specified: Option<bool>,
  pub exports_fields: Option<Vec<String>>,
  pub description_files: Option<Vec<String>>,
  pub enforce_extension: Option<bool>,
  pub imports_fields: Option<Vec<String>>,
  #[napi(ts_type = "Record<string, Array<string>>")]
  pub extension_alias: Option<HashMap<String, Vec<String>>>,
  pub alias_fields: Option<Vec<String>>,
  pub restrictions: Option<Vec<String>>,
  pub roots: Option<Vec<String>>,
}

fn normalize_alias(alias: Option<Vec<RawAliasOptionItem>>) -> rspack_error::Result<Option<Alias>> {
  alias
    .map(|alias| {
      alias
        .into_iter()
        .map(|alias_item| {
          alias_item
            .redirect
            .into_iter()
            .map(|value| {
              if let Some(s) = value.as_str() {
                Ok(AliasMap::Path(s.to_string()))
              } else if let Some(b) = value.as_bool() {
                if b {
                  Err(error!("Alias should not be true in {}", alias_item.path))
                } else {
                  Ok(AliasMap::Ignore)
                }
              } else {
                Err(error!(
                  "Alias should be false or string in {}",
                  alias_item.path
                ))
              }
            })
            .collect::<rspack_error::Result<_>>()
            .map(|value| (alias_item.path, value))
        })
        .collect::<rspack_error::Result<_>>()
    })
    .map_or(Ok(None), |v| v.map(Some))
}

impl TryFrom<RawResolveOptions> for Resolve {
  type Error = rspack_error::Error;

  fn try_from(value: RawResolveOptions) -> Result<Self, Self::Error> {
    let prefer_relative = value.prefer_relative;
    let prefer_absolute = value.prefer_absolute;
    let extensions = value.extensions;
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
    let exports_fields = value
      .exports_fields
      .map(|v| v.into_iter().map(|s| vec![s]).collect());
    let extension_alias = value.extension_alias.map(|v| v.into_iter().collect());
    let alias_fields = value
      .alias_fields
      .map(|v| v.into_iter().map(|s| vec![s]).collect());
    let restrictions = value.restrictions;
    let roots = value.roots;
    let enforce_extension = value.enforce_extension;
    let description_files = value.description_files;
    let imports_fields = value
      .imports_fields
      .map(|v| v.into_iter().map(|s| vec![s]).collect());

    Ok(Resolve {
      modules,
      prefer_relative,
      prefer_absolute,
      extensions,
      main_fields,
      main_files,
      condition_names,
      alias,
      symlinks,
      tsconfig,
      fallback,
      by_dependency,
      fully_specified,
      exports_fields,
      extension_alias,
      alias_fields,
      restrictions,
      roots,
      enforce_extension,
      description_files,
      imports_fields,
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

#[derive(Debug)]
#[napi(object)]
pub struct RawResolveOptionsWithDependencyType {
  pub resolve: RawResolveOptions,
  pub dependency_category: Option<String>,
  pub resolve_to_context: Option<bool>,
}
