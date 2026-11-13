use napi::{
  Either,
  bindgen_prelude::{FromNapiValue, Object},
  sys,
};
use napi_derive::napi;
use rspack_core::{
  Alias, AliasMap, ByDependency, DependencyCategory, Resolve, ResolveOptionsWithDependencyType,
  Restriction, TsconfigOptions, TsconfigReferences,
};
use rspack_error::error;
use rspack_napi::owned_value::{FromObject, FromValue, Value, ValueGraph, ValueRef};
use rspack_napi_macros::FromJsObject;
use rspack_regex::RspackRegex;
use rustc_hash::FxHashMap as HashMap;

#[derive(Debug, FromJsObject)]
#[napi(object)]
pub struct RawAliasOptionItem {
  pub path: String,
  #[napi(ts_type = "Array<string | false>")]
  pub redirect: Vec<Either<String, bool>>,
}

#[derive(Debug, FromJsObject)]
#[napi(object)]
pub struct RawResolveTsconfigOptions {
  pub config_file: String,
  #[napi(ts_type = r#""auto" | "manual" | "disabled""#)]
  pub references_type: String,
  pub references: Option<Vec<String>>,
}

#[derive(Debug, FromJsObject)]
#[napi(object, object_to_js = false)]
pub struct RawResolveOptions {
  pub prefer_relative: Option<bool>,
  pub prefer_absolute: Option<bool>,
  pub extensions: Option<Vec<String>>,
  pub main_files: Option<Vec<String>>,
  pub main_fields: Option<Vec<String>>,
  pub condition_names: Option<Vec<String>>,
  #[napi(ts_type = "Array<RawAliasOptionItem> | false")]
  pub alias: Option<Either<Vec<RawAliasOptionItem>, bool>>,
  #[napi(ts_type = "Array<RawAliasOptionItem> | false")]
  pub fallback: Option<Either<Vec<RawAliasOptionItem>, bool>>,
  pub symlinks: Option<bool>,
  pub tsconfig: Option<RawResolveTsconfigOptions>,
  pub modules: Option<Vec<String>>,
  pub by_dependency: Option<HashMap<String, RawConfigValue>>,
  pub fully_specified: Option<bool>,
  pub exports_fields: Option<Vec<String>>,
  pub description_files: Option<Vec<String>>,
  pub enforce_extension: Option<bool>,
  pub imports_fields: Option<Vec<String>>,
  #[napi(ts_type = "Record<string, Array<string>>")]
  pub extension_alias: Option<HashMap<String, Vec<String>>>,
  pub alias_fields: Option<Vec<String>>,
  #[napi(ts_type = "(string | RegExp)[]")]
  #[js_value(with = decode_restrictions)]
  pub restrictions: Option<Vec<Either<String, RspackRegex>>>,
  pub roots: Option<Vec<String>>,
  pub pnp: Option<bool>,
}

#[derive(Debug)]
pub struct RawConfigValue(ValueGraph);

impl FromNapiValue for RawConfigValue {
  unsafe fn from_napi_value(env: sys::napi_env, value: sys::napi_value) -> napi::Result<Self> {
    let object = unsafe { Object::from_napi_value(env, value)? };
    ValueGraph::capture(object, |_| Ok(Value::Unsupported)).map(Self)
  }
}

impl FromValue for RawConfigValue {
  fn from_value(value: ValueRef<'_>) -> Option<Self> {
    value.subgraph().map(Self)
  }
}

impl TryFrom<RawConfigValue> for Resolve {
  type Error = rspack_error::Error;

  fn try_from(value: RawConfigValue) -> Result<Self, Self::Error> {
    let fields = value
      .0
      .root()
      .object()
      .ok_or_else(|| error!("Expected resolve options to be an object"))?;
    RawResolveOptions::from_object(fields)?.try_into()
  }
}

fn normalize_by_dependency(
  by_dependency: Option<HashMap<String, RawConfigValue>>,
) -> rspack_error::Result<Option<ByDependency>> {
  by_dependency
    .map(|options| {
      options
        .into_iter()
        .map(|(key, value)| {
          let resolve = Resolve::try_from(value)
            .map_err(|error| error!("Invalid resolve.byDependency['{key}']: {error}"))?;
          Ok((key.into(), resolve))
        })
        .collect()
    })
    .transpose()
}

pub(crate) fn add_dom_fields(fields: &mut HashMap<String, Vec<String>>) {
  for (name, keys) in [
    ("resolve", RawResolveOptions::FIELDS),
    ("alias", RawAliasOptionItem::FIELDS),
    ("tsconfig", RawResolveTsconfigOptions::FIELDS),
  ] {
    fields.insert(
      name.into(),
      keys.iter().map(|key| (*key).to_owned()).collect(),
    );
  }
}

fn decode_restrictions(value: ValueRef<'_>) -> Option<Vec<Either<String, RspackRegex>>> {
  let Value::Array(values) = value.value() else {
    return None;
  };
  values
    .iter()
    .map(|item| {
      let item = value.child(item);
      match item.value() {
        Value::String(path) => Some(Either::A(path.clone())),
        Value::Object(_) => {
          let fields = item.object()?;
          let (Value::String(source), Value::String(flags)) =
            (fields.get("source")?.value(), fields.get("flags")?.value())
          else {
            return None;
          };
          RspackRegex::with_flags(source, flags).ok().map(Either::B)
        }
        _ => None,
      }
    })
    .collect()
}

fn normalize_alias(
  alias: Option<Either<Vec<RawAliasOptionItem>, bool>>,
) -> rspack_error::Result<Option<Alias>> {
  alias
    .map(|alias| match alias {
      Either::A(alias) => {
        let alias = alias
          .into_iter()
          .map(|alias_item| {
            alias_item
              .redirect
              .into_iter()
              .map(|value| match value {
                Either::A(path) => Ok(AliasMap::Path(path)),
                Either::B(false) => Ok(AliasMap::Ignore),
                Either::B(true) => Err(error!("Alias should not be true in {}", alias_item.path)),
              })
              .collect::<rspack_error::Result<Vec<_>>>()
              .map(|value| (alias_item.path, value))
          })
          .collect::<rspack_error::Result<Vec<_>>>();
        alias.map(Alias::MergeAlias)
      }
      Either::B(falsy) => {
        assert!(!falsy, "Alias should not be true");
        Ok(Alias::OverwriteToNoAlias)
      }
    })
    .map_or(Ok(None), |v| v.map(Some))
}

impl TryFrom<RawResolveOptions> for Resolve {
  type Error = rspack_error::Error;

  fn try_from(value: RawResolveOptions) -> Result<Self, Self::Error> {
    let pnp = value.pnp;
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
    let by_dependency = normalize_by_dependency(value.by_dependency)?;
    let exports_fields = value
      .exports_fields
      .map(|v| v.into_iter().map(|s| vec![s]).collect());
    let extension_alias = value.extension_alias.map(|v| v.into_iter().collect());
    let alias_fields = value
      .alias_fields
      .map(|v| v.into_iter().map(|s| vec![s]).collect());
    let restrictions = value.restrictions.map(|restrictions| {
      restrictions
        .into_iter()
        .map(|restriction| match restriction {
          Either::A(s) => Restriction::Path(s),
          Either::B(r) => Restriction::Regex(r),
        })
        .collect::<Vec<_>>()
    });
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
      pnp,
      builtin_modules: false,
    })
  }
}

impl TryFrom<RawResolveTsconfigOptions> for TsconfigOptions {
  type Error = rspack_error::Error;
  fn try_from(value: RawResolveTsconfigOptions) -> Result<Self, Self::Error> {
    let references = match value.references_type.as_str() {
      "manual" => TsconfigReferences::Paths(
        value
          .references
          .unwrap_or_default()
          .into_iter()
          .map(Into::into)
          .collect(),
      ),
      "auto" => TsconfigReferences::Auto,
      "disabled" => TsconfigReferences::Disabled,
      _ => panic!(
        "Failed to resolve the references type {}. Expected type is `auto`, `manual` or `disabled`.",
        value.references_type
      ),
    };
    Ok(TsconfigOptions {
      config_file: value.config_file.into(),
      references,
    })
  }
}

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawResolveOptionsWithDependencyType {
  pub prefer_relative: Option<bool>,
  pub prefer_absolute: Option<bool>,
  pub extensions: Option<Vec<String>>,
  pub main_files: Option<Vec<String>>,
  pub main_fields: Option<Vec<String>>,
  pub condition_names: Option<Vec<String>>,
  #[napi(ts_type = "Array<RawAliasOptionItem> | false")]
  pub alias: Option<Either<Vec<RawAliasOptionItem>, bool>>,
  #[napi(ts_type = "Array<RawAliasOptionItem> | false")]
  pub fallback: Option<Either<Vec<RawAliasOptionItem>, bool>>,
  pub symlinks: Option<bool>,
  pub tsconfig: Option<RawResolveTsconfigOptions>,
  pub modules: Option<Vec<String>>,
  pub by_dependency: Option<HashMap<String, RawConfigValue>>,
  pub fully_specified: Option<bool>,
  pub exports_fields: Option<Vec<String>>,
  pub description_files: Option<Vec<String>>,
  pub enforce_extension: Option<bool>,
  pub imports_fields: Option<Vec<String>>,
  #[napi(ts_type = "Record<string, Array<string>>")]
  pub extension_alias: Option<HashMap<String, Vec<String>>>,
  pub alias_fields: Option<Vec<String>>,
  #[napi(ts_type = "(string | RegExp)[]")]
  pub restrictions: Option<Vec<Either<String, RspackRegex>>>,
  pub roots: Option<Vec<String>>,

  pub dependency_type: Option<String>,
  pub resolve_to_context: Option<bool>,
  pub pnp: Option<bool>,
}

pub fn normalize_raw_resolve_options_with_dependency_type(
  raw: Option<RawResolveOptionsWithDependencyType>,
  default_resolve_to_context: bool,
) -> rspack_error::Result<ResolveOptionsWithDependencyType> {
  match raw {
    Some(raw) => {
      let tsconfig = match raw.tsconfig {
        Some(config) => Some(TsconfigOptions::try_from(config)?),
        None => None,
      };

      let exports_fields = raw
        .exports_fields
        .map(|v| v.into_iter().map(|s| vec![s]).collect());

      let extension_alias = raw.extension_alias.map(|v| v.into_iter().collect());

      let alias_fields = raw
        .alias_fields
        .map(|v| v.into_iter().map(|s| vec![s]).collect());

      let imports_fields = raw
        .imports_fields
        .map(|v| v.into_iter().map(|s| vec![s]).collect());

      let by_dependency = normalize_by_dependency(raw.by_dependency)?;

      let restrictions = raw.restrictions.map(|restrictions| {
        restrictions
          .into_iter()
          .map(|restriction| match restriction {
            Either::A(s) => Restriction::Path(s),
            Either::B(r) => Restriction::Regex(r),
          })
          .collect::<Vec<_>>()
      });

      let resolve_options = Resolve {
        extensions: raw.extensions,
        alias: normalize_alias(raw.alias)?,
        prefer_relative: raw.prefer_relative,
        prefer_absolute: raw.prefer_absolute,
        symlinks: raw.symlinks,
        main_files: raw.main_files,
        main_fields: raw.main_fields,
        condition_names: raw.condition_names,
        tsconfig,
        pnp: raw.pnp,
        modules: raw.modules,
        fallback: normalize_alias(raw.fallback)?,
        fully_specified: raw.fully_specified,
        exports_fields,
        extension_alias,
        alias_fields,
        roots: raw.roots,
        restrictions,
        imports_fields,
        by_dependency,
        description_files: raw.description_files,
        enforce_extension: raw.enforce_extension,
        builtin_modules: false,
      };
      Ok(ResolveOptionsWithDependencyType {
        resolve_options: Some(Box::new(resolve_options)),
        resolve_to_context: raw.resolve_to_context.unwrap_or(default_resolve_to_context),
        dependency_category: raw
          .dependency_type
          .map_or(DependencyCategory::Unknown, |c| {
            DependencyCategory::from(c.as_str())
          }),
      })
    }
    None => Ok(ResolveOptionsWithDependencyType {
      resolve_options: None,
      resolve_to_context: default_resolve_to_context,
      dependency_category: DependencyCategory::Unknown,
    }),
  }
}
