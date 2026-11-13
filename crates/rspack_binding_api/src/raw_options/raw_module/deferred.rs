use std::sync::{Arc, LazyLock, Mutex};

use napi::{
  Either,
  bindgen_prelude::{FromNapiValue, JsValuesTupleIntoVec, Object},
};
use once_cell::sync::OnceCell;
use rspack_core::{
  DeferredModuleRuleOptions, GeneratorOptions, GeneratorOptionsMap, ModuleRuleOptions, ModuleType,
  ParserOptions, ParserOptionsMap,
};
use rspack_error::Result;
use rspack_napi::owned_value::{
  FromObject, FromValue, Value, ValueGraph, ValueRef, enumerable_own_keys,
};
use rspack_regex::RspackRegex;
use rspack_util::fx_hash::FxIndexSet;
use rustc_hash::FxHashMap as HashMap;

use super::{
  RawAssetGeneratorDataUrlOptions, RawAssetGeneratorOptions, RawAssetInlineGeneratorOptions,
  RawAssetParserDataUrlOptions, RawAssetParserOptions, RawAssetResourceGeneratorOptions,
  RawCssAutoOrModuleParserOptions, RawCssGeneratorOptions, RawCssModuleGeneratorOptions,
  RawCssModuleParserOptions, RawCssParserOptions, RawImportMetaOptions,
  RawJavascriptParserCommonjsExports, RawJavascriptParserCommonjsOptions,
  RawJavascriptParserOptions, RawJavascriptParserWorkerOptions, RawJsonGeneratorOptions,
  RawJsonParserOptions,
};
use crate::{
  compiler_scoped_tsfn::{CompilerScopedTsFnHandle as ThreadsafeFunction, capture_scoped_function},
  filename::JsFilename,
};

pub type RawParserOptions = ValueGraph;
pub type RawGeneratorOptions = ValueGraph;

fn capture_options(object: Object<'_>) -> napi::Result<ValueGraph> {
  ValueGraph::capture(object, |value| {
    capture_scoped_function(value).map(Value::Function)
  })
}

pub(super) fn config_value_fields() -> HashMap<String, Vec<String>> {
  let mut fields = HashMap::default();
  RawParserOptionObject::add_fields("parser", &mut fields);
  RawGeneratorOptionObject::add_fields("generator", &mut fields);
  fields.extend([
    (
      "worker".into(),
      field_names(RawJavascriptParserWorkerOptions::FIELDS),
    ),
    (
      "commonjs".into(),
      field_names(RawJavascriptParserCommonjsOptions::FIELDS),
    ),
    (
      "dataUrlCondition".into(),
      field_names(RawAssetParserDataUrlOptions::FIELDS),
    ),
    (
      "dataUrl".into(),
      field_names(RawAssetGeneratorDataUrlOptions::FIELDS),
    ),
  ]);
  crate::options::raw_resolve::add_dom_fields(&mut fields);
  fields
}

fn field_names(fields: &[&str]) -> Vec<String> {
  fields.iter().map(|field| (*field).to_owned()).collect()
}

fn capture_module_options(
  object: Object<'_>,
  fields_for_type: fn(&str) -> Option<&'static [&'static str]>,
  ignore_type: fn(&str) -> bool,
) -> napi::Result<HashMap<String, ValueGraph>> {
  let mut options = HashMap::default();
  for key in enumerable_own_keys(&object)? {
    if fields_for_type(&key).is_none() {
      if ignore_type(&key) {
        continue;
      }
      return Err(napi::Error::from_reason(format!(
        "Unknown module type: {key}"
      )));
    }
    if let Some(value) = object.get::<Object>(&key)? {
      options.insert(key, capture_options(value)?);
    }
  }
  Ok(options)
}

#[derive(Debug)]
pub(super) struct RuleOptions {
  parser: Option<RawParserOptionObject>,
  generator: Option<RawGeneratorOptionObject>,
  // Most rules are used with a single module type. Keep that lookup lock-free once initialized.
  first_type: OnceCell<(ModuleType, Arc<ModuleRuleOptions>)>,
  other_types: Mutex<Vec<(ModuleType, Arc<ModuleRuleOptions>)>>,
}

impl RuleOptions {
  pub(super) fn deferred(
    parser: Option<RawParserOptionObject>,
    generator: Option<RawGeneratorOptionObject>,
  ) -> Option<Box<dyn DeferredModuleRuleOptions>> {
    if parser.is_none() && generator.is_none() {
      return None;
    }
    Some(Box::new(Self {
      parser,
      generator,
      first_type: OnceCell::new(),
      other_types: Mutex::default(),
    }))
  }
}

impl RuleOptions {
  fn convert(&self, module_type: ModuleType) -> Result<Arc<ModuleRuleOptions>> {
    Ok(Arc::new(ModuleRuleOptions {
      parser: self
        .parser
        .as_ref()
        .map(|parser| RawParserOptionObject::resolve(&parser.0, module_type.as_str()))
        .transpose()?
        .flatten(),
      generator: self
        .generator
        .as_ref()
        .map(|generator| RawGeneratorOptionObject::resolve(&generator.0, module_type.as_str()))
        .transpose()?
        .flatten(),
    }))
  }
}

impl DeferredModuleRuleOptions for RuleOptions {
  fn resolve(&self, module_type: ModuleType) -> Result<Arc<ModuleRuleOptions>> {
    let (first_type, first_options) = self.first_type.get_or_try_init(|| {
      self
        .convert(module_type)
        .map(|options| (module_type, options))
    })?;
    if *first_type == module_type {
      return Ok(Arc::clone(first_options));
    }

    let mut other_types = self.other_types.lock().expect("rule options lock poisoned");
    if let Some((_, options)) = other_types.iter().find(|(kind, _)| *kind == module_type) {
      return Ok(Arc::clone(options));
    }

    let options = self.convert(module_type)?;
    other_types.push((module_type, Arc::clone(&options)));
    Ok(options)
  }
}

// Keep the capture schema and the typed decoder in sync for each module type.
macro_rules! module_options {
  ($object:ident, $raw_options:ident, $options:ident, $map:ident, $ignore_type:expr, {
    $($($kind:literal)|+ => $variant:ident($raw:ident)),+ $(,)?
  }) => {
    #[derive(Debug)]
    pub struct $object(ValueGraph);

    impl FromNapiValue for $object {
      unsafe fn from_napi_value(
        env: napi::sys::napi_env,
        value: napi::sys::napi_value,
      ) -> napi::Result<Self> {
        let object = unsafe { Object::from_napi_value(env, value)? };
        capture_options(object).map(Self)
      }
    }

    impl $object {
      // Read supported fields by name to include accessors and inherited fields.
      fn rule_fields() -> &'static FxIndexSet<&'static str> {
        static FIELDS: LazyLock<FxIndexSet<&'static str>> = LazyLock::new(|| {
          [$($raw::FIELDS),+].into_iter().flatten().copied().collect()
        });
        &FIELDS
      }

      fn add_fields(prefix: &str, fields: &mut HashMap<String, Vec<String>>) {
        fields.insert(format!("{prefix}:rule"), field_names(
          Self::rule_fields().iter().copied().collect::<Vec<_>>().as_slice(),
        ));
        $(
          for kind in [$($kind),+] {
            fields.insert(format!("{prefix}:{kind}"), field_names($raw::FIELDS));
          }
        )+
      }

      fn fields_for_type(module_type: &str) -> Option<&'static [&'static str]> {
        match module_type {
          $($($kind)|+ => Some($raw::FIELDS),)+
          _ => None,
        }
      }

      fn resolve(graph: &ValueGraph, module_type: &str) -> Result<Option<$options>> {
        let fields = graph.root().object().ok_or_else(|| {
          rspack_error::error!("Expected module options to be an object")
        })?;
        Ok(Some(match module_type {
          $($($kind)|+ => $options::$variant($raw::from_object(fields)?.into()),)+
          _ => return Ok(None),
        }))
      }

      pub(super) fn capture_module_options(
        object: Object<'_>,
      ) -> napi::Result<HashMap<String, $raw_options>> {
        capture_module_options(object, Self::fields_for_type, $ignore_type)
      }

      pub(super) fn into_module_options(options: HashMap<String, $raw_options>) -> Result<$map> {
        let mut result = $map::default();
        for (kind, fields) in options {
          if let Some(options) = Self::resolve(&fields, &kind)? {
            result.insert(kind, options);
          }
        }
        Ok(result)
      }
    }
  };
}

module_options!(RawParserOptionObject, RawParserOptions, ParserOptions, ParserOptionsMap, |_| false, {
  "javascript" => Javascript(RawJavascriptParserOptions),
  "javascript/auto" => JavascriptAuto(RawJavascriptParserOptions),
  "javascript/dynamic" => JavascriptDynamic(RawJavascriptParserOptions),
  "javascript/esm" => JavascriptEsm(RawJavascriptParserOptions),
  "asset" => Asset(RawAssetParserOptions),
  "css/auto" | "css/module" => CssAutoOrModule(RawCssAutoOrModuleParserOptions),
  "css/global" => CssModule(RawCssModuleParserOptions),
  "css" => Css(RawCssParserOptions),
  "json" => Json(RawJsonParserOptions),
});

module_options!(RawGeneratorOptionObject, RawGeneratorOptions, GeneratorOptions, GeneratorOptionsMap, |key| matches!(
  key,
  "asset/source" | "asset/bytes" | "javascript" | "javascript/auto" | "javascript/dynamic" | "javascript/esm"
), {
  "asset" => Asset(RawAssetGeneratorOptions),
  "asset/inline" => AssetInline(RawAssetInlineGeneratorOptions),
  "asset/resource" => AssetResource(RawAssetResourceGeneratorOptions),
  "css" => Css(RawCssGeneratorOptions),
  "css/auto" | "css/global" | "css/module" => CssModule(RawCssModuleGeneratorOptions),
  "json" => Json(RawJsonGeneratorOptions),
});

impl<T: 'static + JsValuesTupleIntoVec, R> FromValue for ThreadsafeFunction<T, R> {
  fn from_value(value: ValueRef<'_>) -> Option<Self> {
    match value.value() {
      Value::Function(callback) => Some(Self::from_dynamic(callback.clone())),
      _ => None,
    }
  }
}

pub(super) fn wrapped_context_reg_exp(value: ValueRef<'_>) -> Option<RspackRegex> {
  let fields = value.object()?;
  let (Value::String(source), Value::String(flags)) =
    (fields.get("source")?.value(), fields.get("flags")?.value())
  else {
    return None;
  };
  RspackRegex::with_flags(source, flags).ok()
}

impl FromValue for JsFilename {
  fn from_value(value: ValueRef<'_>) -> Option<Self> {
    let filename = match value.value() {
      Value::String(value) => Either::A(value.clone()),
      Value::Function(callback) => Either::B(ThreadsafeFunction::from_dynamic(callback.clone())),
      _ => return None,
    };
    Some(Self { filename })
  }
}

impl FromValue for RawJavascriptParserCommonjsExports {
  fn from_value(value: ValueRef<'_>) -> Option<Self> {
    match value.value() {
      Value::String(value) if value == "skipInEsm" => Some(Self::SkipInEsm),
      _ => None,
    }
  }
}

pub(super) fn stringified(value: ValueRef<'_>) -> Option<String> {
  match value.value() {
    Value::Bool(value) => Some(value.to_string()),
    Value::Number(value) => Some(rspack_util::ryu_js::Buffer::new().format(*value).to_owned()),
    _ => String::from_value(value),
  }
}

pub(super) fn false_or_string(value: ValueRef<'_>) -> Option<String> {
  match value.value() {
    Value::Bool(false) => Some("false".into()),
    _ => String::from_value(value),
  }
}

pub(super) fn import_meta(value: ValueRef<'_>) -> Option<Either<String, RawImportMetaOptions>> {
  match value.value() {
    Value::Object(_) => Some(Either::B(
      value
        .object()?
        .iter()
        .filter_map(|(key, value)| bool::from_value(value).map(|value| (key.clone(), value)))
        .collect(),
    )),
    Value::Bool(value) => Some(Either::A(value.to_string())),
    _ => String::from_value(value).map(Either::A),
  }
}
