use std::{borrow::Cow, cell::RefCell};

use napi::{
  Env,
  bindgen_prelude::{Array, FromNapiValue, JsObjectValue, Object},
  sys::napi_value,
};
use napi_derive::napi;
use rspack_collections::IdentifierMap;
use rspack_core::{
  EntrypointsStatsOption, ExtendedStatsOptions, Stats, StatsChunk, StatsModule, StatsUsedExports,
  rspack_sources::{RawBufferSource, Source, SourceValue},
};
use rspack_error::Severity;
use rspack_napi::napi::{
  Either,
  bindgen_prelude::{Buffer, Result, SharedReference, ToNapiValue},
};
use rspack_util::{atom::Atom, itoa};
use rustc_hash::FxHashMap as HashMap;

use crate::{
  chunk_graph::{JsModuleId, to_js_module_id},
  compilation::JsCompilation,
  error::{RspackError, RspackResultToNapiResultExt},
  identifier::JsIdentifier,
};

// These handles are only used during the `to_json` call,
// so we can store raw `napi_value` here.
thread_local! {
  static MODULE_DESCRIPTOR_REFS: RefCell<IdentifierMap<napi_value>> = Default::default();
  static MODULE_COMMON_ATTRIBUTES_REFS: RefCell<IdentifierMap<napi_value>> = Default::default();
}

pub struct CowStrWrapper<'a>(Cow<'a, str>);

impl<'a> CowStrWrapper<'a> {
  pub fn new(s: Cow<'a, str>) -> Self {
    Self(s)
  }
}

impl<'a> ToNapiValue for CowStrWrapper<'a> {
  unsafe fn to_napi_value(env: napi::sys::napi_env, val: Self) -> Result<napi::sys::napi_value> {
    unsafe { ToNapiValue::to_napi_value(env, val.0.as_ref()) }
  }
}

pub struct StringSliceWrapper<'a>(&'a [String]);

impl<'a> StringSliceWrapper<'a> {
  pub fn new(slice: &'a [String]) -> Self {
    Self(slice)
  }
}

impl<'a> ToNapiValue for StringSliceWrapper<'a> {
  unsafe fn to_napi_value(env: napi::sys::napi_env, val: Self) -> Result<napi::sys::napi_value> {
    unsafe {
      let env_wrapper = Env::from_raw(env);
      let mut array = env_wrapper.create_array(val.0.len() as u32)?;
      for (i, item) in val.0.iter().enumerate() {
        let s = env_wrapper.create_string(item)?;
        array.set(i as u32, s)?;
      }
      ToNapiValue::to_napi_value(env, array)
    }
  }
}

pub struct AtomWrapper(Atom);

impl AtomWrapper {
  pub fn new(atom: Atom) -> Self {
    Self(atom)
  }
}

impl ToNapiValue for AtomWrapper {
  unsafe fn to_napi_value(env: napi::sys::napi_env, val: Self) -> Result<napi::sys::napi_value> {
    unsafe { ToNapiValue::to_napi_value(env, val.0.as_str()) }
  }
}

pub struct AtomVecWrapper(Vec<Atom>);

impl AtomVecWrapper {
  pub fn new(atom_vec: Vec<Atom>) -> Self {
    Self(atom_vec)
  }
}

impl ToNapiValue for AtomVecWrapper {
  unsafe fn to_napi_value(env: napi::sys::napi_env, val: Self) -> Result<napi::sys::napi_value> {
    unsafe {
      let env_wrapper = Env::from_raw(env);
      let mut array = env_wrapper.create_array(val.0.len() as u32)?;
      for (i, item) in val.0.iter().enumerate() {
        let s = env_wrapper.create_string(item.as_str())?;
        array.set(i as u32, s)?;
      }
      ToNapiValue::to_napi_value(env, array)
    }
  }
}

#[napi(object, object_from_js = false)]
pub struct JsModuleDescriptor<'a> {
  #[napi(ts_type = "string")]
  pub identifier: JsIdentifier,
  #[napi(ts_type = "string")]
  pub name: CowStrWrapper<'a>,
  #[napi(ts_type = "string | number | null")]
  pub id: Option<JsModuleId>,
}

pub struct JsModuleDescriptorWrapper<'a>(JsModuleDescriptor<'a>);

impl<'a> JsModuleDescriptorWrapper<'a> {
  pub fn raw(&self) -> &JsModuleDescriptor<'_> {
    &self.0
  }
}

impl<'a> ToNapiValue for JsModuleDescriptorWrapper<'a> {
  unsafe fn to_napi_value(env: napi::sys::napi_env, val: Self) -> Result<napi::sys::napi_value> {
    MODULE_DESCRIPTOR_REFS.with(|ref_cell| {
      let id = val.0.identifier.raw();
      {
        if let Some(raw_value) = ref_cell.borrow().get(&id) {
          return Ok(*raw_value);
        }
      }
      let raw_value = unsafe { ToNapiValue::to_napi_value(env, val.0)? };
      {
        ref_cell.borrow_mut().insert(id, raw_value);
      }
      Ok(raw_value)
    })
  }
}

impl<'a> From<JsModuleDescriptor<'a>> for JsModuleDescriptorWrapper<'a> {
  fn from(value: JsModuleDescriptor<'a>) -> Self {
    Self(value)
  }
}

#[napi(object, object_from_js = false)]
pub struct JsStatsError<'a> {
  pub name: Option<String>,
  #[napi(ts_type = "JsModuleDescriptor")]
  pub module_descriptor: Option<JsModuleDescriptorWrapper<'a>>,
  pub message: String,
  pub chunk_name: Option<&'a str>,
  pub code: Option<String>,
  pub chunk_entry: Option<bool>,
  pub chunk_initial: Option<bool>,
  pub loc: Option<String>,
  pub file: Option<&'a str>,
  pub chunk_id: Option<&'a str>,
  pub details: Option<String>,
  pub stack: Option<String>,
  pub module_trace: Vec<JsStatsModuleTrace<'a>>,
}

impl<'a> From<rspack_core::StatsError<'a>> for JsStatsError<'a> {
  fn from(stats: rspack_core::StatsError<'a>) -> Self {
    Self {
      name: stats.name,
      module_descriptor: stats.module_identifier.map(|identifier| {
        JsModuleDescriptor {
          identifier: identifier.into(),
          name: CowStrWrapper::new(stats.module_name.unwrap_or_default()),
          id: stats.module_id.map(|s| to_js_module_id(&s)),
        }
        .into()
      }),
      message: stats.message,
      code: stats.code,
      loc: stats.loc,
      file: stats.file.map(|f| f.as_str()),
      chunk_name: stats.chunk_name,
      chunk_entry: stats.chunk_entry,
      chunk_initial: stats.chunk_initial,
      chunk_id: stats.chunk_id,
      details: stats.details,
      stack: stats.stack,
      module_trace: stats
        .module_trace
        .into_iter()
        .map(Into::into)
        .collect::<Vec<_>>(),
    }
  }
}

#[napi(object, object_from_js = false)]
pub struct JsStatsModuleTrace<'a> {
  pub origin: JsStatsModuleTraceModule<'a>,
  pub module: JsStatsModuleTraceModule<'a>,
  pub dependencies: Vec<JsStatsModuleTraceDependency>,
}

impl<'a> From<rspack_core::StatsModuleTrace<'a>> for JsStatsModuleTrace<'a> {
  fn from(stats: rspack_core::StatsModuleTrace<'a>) -> Self {
    Self {
      origin: stats.origin.into(),
      module: stats.module.into(),
      dependencies: stats.dependencies.into_iter().map(Into::into).collect(),
    }
  }
}

#[napi(object, object_from_js = false)]
pub struct JsStatsModuleTraceModule<'a> {
  #[napi(ts_type = "JsModuleDescriptor")]
  pub module_descriptor: JsModuleDescriptorWrapper<'a>,
}

impl<'a> From<rspack_core::StatsErrorModuleTraceModule<'a>> for JsStatsModuleTraceModule<'a> {
  fn from(stats: rspack_core::StatsErrorModuleTraceModule<'a>) -> Self {
    Self {
      module_descriptor: JsModuleDescriptor {
        identifier: stats.identifier.into(),
        name: CowStrWrapper::new(stats.name),
        id: stats.id.map(|s| to_js_module_id(&s)),
      }
      .into(),
    }
  }
}

#[napi(object, object_from_js = false)]
pub struct JsStatsModuleTraceDependency {
  pub loc: String,
}

impl From<rspack_core::StatsErrorModuleTraceDependency> for JsStatsModuleTraceDependency {
  fn from(stats: rspack_core::StatsErrorModuleTraceDependency) -> Self {
    Self { loc: stats.loc }
  }
}

#[napi(object, object_from_js = false)]
pub struct JsStatsLogging<'a> {
  pub name: String,
  pub r#type: &'a str,
  pub args: Option<Vec<String>>,
  pub trace: Option<Vec<String>>,
}

impl<'a> From<(String, rspack_core::LogType)> for JsStatsLogging<'a> {
  fn from(value: (String, rspack_core::LogType)) -> Self {
    match value.1 {
      rspack_core::LogType::Error { message, trace } => Self {
        name: value.0,
        r#type: "error",
        args: Some(vec![message]),
        trace: Some(trace),
      },
      rspack_core::LogType::Warn { message, trace } => Self {
        name: value.0,
        r#type: "warn",
        args: Some(vec![message]),
        trace: Some(trace),
      },
      rspack_core::LogType::Info { message } => Self {
        name: value.0,
        r#type: "info",
        args: Some(vec![message]),
        trace: None,
      },
      rspack_core::LogType::Log { message } => Self {
        name: value.0,
        r#type: "log",
        args: Some(vec![message]),
        trace: None,
      },
      rspack_core::LogType::Debug { message } => Self {
        name: value.0,
        r#type: "debug",
        args: Some(vec![message]),
        trace: None,
      },
      rspack_core::LogType::Trace { message, trace } => Self {
        name: value.0,
        r#type: "trace",
        args: Some(vec![message]),
        trace: Some(trace),
      },
      rspack_core::LogType::Group { message } => Self {
        name: value.0,
        r#type: "group",
        args: Some(vec![message]),
        trace: None,
      },
      rspack_core::LogType::GroupCollapsed { message } => Self {
        name: value.0,
        r#type: "groupCollapsed",
        args: Some(vec![message]),
        trace: None,
      },
      rspack_core::LogType::GroupEnd => Self {
        name: value.0,
        r#type: "groupEnd",
        args: None,
        trace: None,
      },
      rspack_core::LogType::Profile { label } => Self {
        name: value.0,
        r#type: "profile",
        args: Some(vec![label.to_string()]),
        trace: None,
      },
      rspack_core::LogType::ProfileEnd { label } => Self {
        name: value.0,
        r#type: "profileEnd",
        args: Some(vec![label.to_string()]),
        trace: None,
      },
      rspack_core::LogType::Time {
        label,
        secs,
        subsec_nanos,
      } => {
        let mut time_buffer = itoa::Buffer::new();
        let time_str = time_buffer.format(secs * 1000 + subsec_nanos as u64 / 1000000);
        Self {
          name: value.0,
          r#type: "time",
          args: Some(vec![format!("{}: {} ms", label, time_str)]),
          trace: None,
        }
      }
      rspack_core::LogType::Clear => Self {
        name: value.0,
        r#type: "clear",
        args: None,
        trace: None,
      },
      rspack_core::LogType::Status { message } => Self {
        name: value.0,
        r#type: "status",
        args: Some(vec![message]),
        trace: None,
      },
      rspack_core::LogType::Cache { label, hit, total } => {
        let mut hit_buffer = itoa::Buffer::new();
        let hit_str = hit_buffer.format(hit);
        let mut total_buffer = itoa::Buffer::new();
        let total_str = total_buffer.format(total);
        Self {
          name: value.0,
          r#type: "cache",
          args: Some(vec![format!(
            "{}: {:.1}% ({}/{})",
            label,
            if total == 0 {
              0 as f32
            } else {
              hit as f32 / total as f32 * 100_f32
            },
            hit_str,
            total_str,
          )]),
          trace: None,
        }
      }
    }
  }
}

#[napi(object, object_from_js = false)]
pub struct JsStatsAsset<'a> {
  pub r#type: &'a str,
  pub name: &'a str,
  pub info: JsStatsAssetInfo<'a>,
  pub size: f64,
  pub emitted: bool,
  pub chunk_names: Vec<&'a str>,
  pub chunk_id_hints: Vec<&'a str>,
  pub chunks: Vec<Option<&'a str>>,
  pub auxiliary_chunk_names: Vec<&'a str>,
  pub auxiliary_chunk_id_hints: Vec<&'a str>,
  pub auxiliary_chunks: Vec<Option<&'a str>>,
}

impl<'a> From<rspack_core::StatsAsset<'a>> for JsStatsAsset<'a> {
  fn from(stats: rspack_core::StatsAsset<'a>) -> Self {
    Self {
      r#type: stats.r#type,
      name: stats.name,
      size: stats.size,
      chunks: stats.chunks,
      chunk_names: stats.chunk_names,
      info: stats.info.into(),
      emitted: stats.emitted,
      chunk_id_hints: stats.chunk_id_hints,
      auxiliary_chunk_id_hints: stats.auxiliary_chunk_id_hints,
      auxiliary_chunks: stats.auxiliary_chunks,
      auxiliary_chunk_names: stats.auxiliary_chunk_names,
    }
  }
}

#[napi(object, object_from_js = false)]
pub struct JsStatsAssetInfo<'a> {
  pub minimized: Option<bool>,
  pub development: Option<bool>,
  pub hot_module_replacement: Option<bool>,
  pub source_filename: Option<&'a str>,
  pub copied: Option<bool>,
  pub immutable: Option<bool>,
  pub javascript_module: Option<bool>,
  pub chunkhash: Vec<&'a str>,
  pub contenthash: Vec<&'a str>,
  pub fullhash: Vec<&'a str>,
  pub related: Vec<JsStatsAssetInfoRelated<'a>>,
  pub is_over_size_limit: Option<bool>,
}

impl<'a> From<rspack_core::StatsAssetInfo<'a>> for JsStatsAssetInfo<'a> {
  fn from(stats: rspack_core::StatsAssetInfo<'a>) -> Self {
    Self {
      minimized: stats.minimized,
      development: stats.development,
      hot_module_replacement: stats.hot_module_replacement,
      source_filename: stats.source_filename,
      copied: stats.copied,
      immutable: stats.immutable,
      javascript_module: stats.javascript_module,
      chunkhash: stats.chunk_hash,
      contenthash: stats.content_hash,
      fullhash: stats.full_hash,
      related: stats
        .related
        .into_iter()
        .map(Into::into)
        .collect::<Vec<_>>(),
      is_over_size_limit: stats.is_over_size_limit,
    }
  }
}

#[napi(object, object_from_js = false)]
pub struct JsStatsAssetInfoRelated<'a> {
  pub name: &'a str,
  pub value: Vec<&'a str>,
}

impl<'a> From<rspack_core::StatsAssetInfoRelated<'a>> for JsStatsAssetInfoRelated<'a> {
  fn from(stats: rspack_core::StatsAssetInfoRelated<'a>) -> Self {
    Self {
      name: stats.name,
      value: stats.value,
    }
  }
}

type JsStatsModuleSource<'a> = Either<CowStrWrapper<'a>, Buffer>;
type JsStatsUsedExports = Either<AtomWrapper, AtomVecWrapper>;

#[napi(object, object_from_js = false)]
pub struct JsStatsModuleCommonAttributes<'a> {
  pub r#type: &'static str,
  pub module_type: &'static str,
  #[napi(ts_type = "string")]
  pub layer: Option<CowStrWrapper<'a>>,
  pub size: f64,
  pub sizes: Vec<JsStatsSize>,
  pub built: bool,
  pub code_generated: bool,
  pub build_time_executed: bool,
  pub cached: bool,

  // module$visible
  #[napi(ts_type = "JsModuleDescriptor")]
  pub module_descriptor: Option<JsModuleDescriptorWrapper<'a>>,
  pub name_for_condition: Option<String>,
  pub pre_order_index: Option<u32>,
  pub post_order_index: Option<u32>,
  pub cacheable: Option<bool>,
  pub optional: Option<bool>,
  pub orphan: Option<bool>,
  pub failed: Option<bool>,
  pub errors: Option<u32>,
  pub warnings: Option<u32>,
  pub profile: Option<JsStatsModuleProfile>,

  // ids
  pub chunks: Option<Vec<&'a str>>,

  // moduleAssets
  pub assets: Option<Vec<&'a str>>,

  // reasons
  pub reasons: Option<Vec<JsStatsModuleReason<'a>>>,

  // providedExports
  #[napi(ts_type = "Array<string>")]
  pub provided_exports: Option<AtomVecWrapper>,

  // optimizationBailout
  #[napi(ts_type = "Array<string>")]
  pub optimization_bailout: Option<StringSliceWrapper<'a>>,

  // depth
  pub depth: Option<u32>,

  // source
  #[napi(ts_type = "string | Buffer")]
  pub source: Option<Either<CowStrWrapper<'a>, Buffer>>,
}

pub struct JsStatsModuleCommonAttributesWrapper<'a>(JsStatsModuleCommonAttributes<'a>);

impl<'a> From<JsStatsModuleCommonAttributes<'a>> for JsStatsModuleCommonAttributesWrapper<'a> {
  fn from(value: JsStatsModuleCommonAttributes<'a>) -> Self {
    JsStatsModuleCommonAttributesWrapper(value)
  }
}

impl<'a> ToNapiValue for JsStatsModuleCommonAttributesWrapper<'a> {
  unsafe fn to_napi_value(env: napi::sys::napi_env, val: Self) -> Result<napi::sys::napi_value> {
    unsafe {
      MODULE_COMMON_ATTRIBUTES_REFS.with(|ref_cell| {
        match val
          .0
          .module_descriptor
          .as_ref()
          .map(|d| d.raw().identifier.raw())
          .as_ref()
        {
          Some(id) => {
            {
              if let Some(raw_value) = ref_cell.borrow().get(id) {
                return ToNapiValue::to_napi_value(env, *raw_value);
              }
            }
            let raw_value = ToNapiValue::to_napi_value(env, val.0)?;
            {
              ref_cell.borrow_mut().insert(*id, raw_value);
            }
            Ok(raw_value)
          }
          None => ToNapiValue::to_napi_value(env, val.0),
        }
      })
    }
  }
}

#[napi(object, object_from_js = false)]
pub struct JsStatsModule<'a> {
  #[napi(ts_type = "JsStatsModuleCommonAttributes")]
  pub common_attributes: JsStatsModuleCommonAttributesWrapper<'a>,
  pub dependent: Option<bool>,
  #[napi(ts_type = "JsModuleDescriptor")]
  pub issuer_descriptor: Option<JsModuleDescriptorWrapper<'a>>,
  pub issuer_path: Option<Vec<JsStatsModuleIssuer<'a>>>,
  #[napi(ts_type = "string | Array<string>")]
  pub used_exports: Option<Either<AtomWrapper, AtomVecWrapper>>,
  pub modules: Option<Vec<JsStatsModule<'a>>>,
}

impl<'a> TryFrom<StatsModule<'a>> for JsStatsModule<'a> {
  type Error = napi::Error;

  fn try_from(stats: StatsModule<'a>) -> std::result::Result<Self, Self::Error> {
    let source = stats.source.map(|source| match source.source() {
      SourceValue::String(string) => JsStatsModuleSource::A(CowStrWrapper::new(string)),
      SourceValue::Buffer(bytes) => JsStatsModuleSource::B(Buffer::from(bytes.to_vec())),
    });

    let mut sizes = stats
      .sizes
      .into_iter()
      .map(|s| JsStatsSize {
        source_type: s.source_type.to_string(),
        size: s.size,
      })
      .collect::<Vec<_>>();
    sizes.sort_by(|a, b| a.source_type.cmp(&b.source_type));
    let modules: Option<Vec<JsStatsModule>> = stats
      .modules
      .map(|modules| -> Result<_> {
        modules
          .into_iter()
          .map(JsStatsModule::try_from)
          .collect::<Result<_>>()
      })
      .transpose()
      .to_napi_result()?;

    let reasons = match stats.reasons {
      Some(reasons) => {
        let js_reasons = reasons
          .into_iter()
          .map(JsStatsModuleReason::from)
          .collect::<Vec<_>>();
        Some(js_reasons)
      }
      None => None,
    };

    let module_descriptor: Option<JsModuleDescriptorWrapper> = stats.identifier.map(|identifier| {
      JsModuleDescriptor {
        identifier: identifier.into(),
        name: CowStrWrapper::new(stats.name.unwrap_or_default()),
        id: stats.id.map(|s| to_js_module_id(&s)),
      }
      .into()
    });

    let common_attributes: JsStatsModuleCommonAttributesWrapper = JsStatsModuleCommonAttributes {
      r#type: stats.r#type,
      module_type: stats.module_type.as_str(),
      layer: stats.layer.map(CowStrWrapper::new),
      size: stats.size,
      sizes,
      built: stats.built,
      code_generated: stats.code_generated,
      build_time_executed: stats.build_time_executed,
      module_descriptor,
      depth: stats.depth.map(|d| d as u32),
      chunks: stats.chunks,
      name_for_condition: stats.name_for_condition,
      reasons,
      assets: stats.assets,
      source,
      profile: stats.profile.map(|p| p.into()),
      orphan: stats.orphan,
      provided_exports: stats.provided_exports.map(AtomVecWrapper::new),
      optimization_bailout: stats.optimization_bailout.map(StringSliceWrapper::new),
      pre_order_index: stats.pre_order_index,
      post_order_index: stats.post_order_index,
      cached: stats.cached,
      cacheable: stats.cacheable,
      optional: stats.optional,
      failed: stats.failed,
      errors: stats.errors,
      warnings: stats.warnings,
    }
    .into();

    let issuer_descriptor: Option<JsModuleDescriptorWrapper<'a>> = stats.issuer.map(|identifier| {
      JsModuleDescriptor {
        identifier: identifier.into(),
        name: CowStrWrapper::new(stats.issuer_name.unwrap_or_default()),
        id: stats.issuer_id.map(|s| to_js_module_id(&s)),
      }
      .into()
    });

    Ok(Self {
      common_attributes,
      dependent: stats.dependent,
      issuer_descriptor,
      issuer_path: stats
        .issuer_path
        .map(|path| path.into_iter().map(Into::into).collect()),
      used_exports: stats.used_exports.map(|used_exports| match used_exports {
        StatsUsedExports::Bool(b) => JsStatsUsedExports::A(AtomWrapper::new(b.to_string().into())),
        StatsUsedExports::Vec(v) => JsStatsUsedExports::B(AtomVecWrapper::new(v)),
        StatsUsedExports::Null => JsStatsUsedExports::A(AtomWrapper::new("null".into())),
      }),
      modules,
    })
  }
}

#[napi(object, object_from_js = false)]
pub struct JsStatsModuleProfile {
  // use f64 to make js side as a number type
  pub factory: f64,
  pub building: f64,
}

impl From<rspack_core::StatsModuleProfile> for JsStatsModuleProfile {
  fn from(value: rspack_core::StatsModuleProfile) -> Self {
    Self {
      // The time is short and no data will be lost when converting from u64 to f64
      factory: value.factory as f64,
      building: value.building as f64,
    }
  }
}

#[napi(object, object_from_js = false)]
pub struct JsStatsModuleIssuer<'a> {
  #[napi(ts_type = "JsModuleDescriptor")]
  pub module_descriptor: JsModuleDescriptorWrapper<'a>,
}

impl<'a> From<rspack_core::StatsModuleIssuer<'a>> for JsStatsModuleIssuer<'a> {
  fn from(stats: rspack_core::StatsModuleIssuer<'a>) -> Self {
    Self {
      module_descriptor: JsModuleDescriptor {
        identifier: stats.identifier.into(),
        name: CowStrWrapper::new(stats.name),
        id: stats.id.map(|s| to_js_module_id(&s)),
      }
      .into(),
    }
  }
}

#[napi(object, object_from_js = false)]
pub struct JsStatsModuleReason<'a> {
  #[napi(ts_type = "JsModuleDescriptor")]
  pub module_descriptor: Option<JsModuleDescriptorWrapper<'a>>,
  #[napi(ts_type = "JsModuleDescriptor")]
  pub resolved_module_descriptor: Option<JsModuleDescriptorWrapper<'a>>,
  pub module_chunks: Option<u32>,
  pub r#type: Option<&'static str>,
  pub user_request: Option<&'a str>,
  pub explanation: Option<&'static str>,
  pub active: bool,
  pub loc: Option<String>,
}

impl<'a> From<rspack_core::StatsModuleReason<'a>> for JsStatsModuleReason<'a> {
  fn from(stats: rspack_core::StatsModuleReason<'a>) -> Self {
    Self {
      module_descriptor: stats.module_identifier.map(|identifier| {
        JsModuleDescriptor {
          identifier: identifier.into(),
          name: CowStrWrapper::new(stats.module_name.clone().unwrap_or_default()),
          id: stats.module_id.map(|s| to_js_module_id(&s)),
        }
        .into()
      }),
      resolved_module_descriptor: stats.resolved_module_identifier.map(|identifier| {
        JsModuleDescriptor {
          identifier: identifier.into(),
          name: CowStrWrapper::new(stats.module_name.unwrap_or_default()),
          id: stats.resolved_module_id.map(|s| to_js_module_id(&s)),
        }
        .into()
      }),
      module_chunks: stats.module_chunks,
      r#type: stats.r#type,
      user_request: stats.user_request,
      explanation: stats.explanation,
      active: stats.active,
      loc: stats.loc,
    }
  }
}

#[napi(object, object_from_js = false)]
pub struct JsOriginRecord<'a> {
  #[napi(ts_type = "JsModuleDescriptor")]
  pub module_descriptor: Option<JsModuleDescriptorWrapper<'a>>,
  pub loc: String,
  pub request: &'a str,
}

#[napi(object, object_from_js = false)]
pub struct JsStatsSize {
  pub source_type: String,
  pub size: f64,
}

#[napi(object, object_from_js = false)]
pub struct JsStatsChunk<'a> {
  pub r#type: &'a str,
  pub files: Vec<&'a str>,
  pub auxiliary_files: Vec<&'a str>,
  pub id: Option<&'a str>,
  pub id_hints: Vec<&'a str>,
  pub hash: Option<&'a str>,
  pub entry: bool,
  pub initial: bool,
  pub names: Vec<&'a str>,
  pub size: f64,
  pub parents: Option<Vec<&'a str>>,
  pub children: Option<Vec<&'a str>>,
  pub siblings: Option<Vec<&'a str>>,
  pub children_by_order: HashMap<String, Vec<String>>,
  pub runtime: Vec<&'a str>,
  pub reason: Option<&'a str>,
  pub rendered: bool,
  pub sizes: Vec<JsStatsSize>,
  pub origins: Vec<JsOriginRecord<'a>>,
  pub modules: Option<Vec<JsStatsModule<'a>>>,
}

impl<'a> TryFrom<StatsChunk<'a>> for JsStatsChunk<'a> {
  type Error = napi::Error;

  fn try_from(stats: StatsChunk<'a>) -> std::result::Result<Self, Self::Error> {
    let mut runtime = stats.runtime.iter().map(|r| r.as_ref()).collect::<Vec<_>>();
    runtime.sort();

    let mut sizes = stats
      .sizes
      .iter()
      .map(|(source_type, size)| JsStatsSize {
        source_type: source_type.to_string(),
        size: *size,
      })
      .collect::<Vec<_>>();
    sizes.sort_by(|a, b| a.source_type.cmp(&b.source_type));

    Ok(JsStatsChunk {
      r#type: stats.r#type,
      files: stats.files,
      auxiliary_files: stats.auxiliary_files,
      id: stats.id,
      entry: stats.entry,
      initial: stats.initial,
      names: stats.names,
      size: stats.size,
      modules: stats
        .modules
        .map(|i| {
          i.into_iter()
            .map(JsStatsModule::try_from)
            .collect::<Result<_>>()
        })
        .transpose()?,
      parents: stats.parents,
      children: stats.children,
      siblings: stats.siblings,
      children_by_order: stats
        .children_by_order
        .into_iter()
        .map(|(order, children)| (order.to_string(), children))
        .collect(),
      runtime,
      sizes,
      reason: stats.reason,
      rendered: stats.rendered,
      origins: stats
        .origins
        .into_iter()
        .map(|origin| JsOriginRecord {
          module_descriptor: origin.module_identifier.map(|identifier| {
            JsModuleDescriptor {
              identifier: identifier.into(),
              name: CowStrWrapper::new(origin.module_name),
              id: origin.module_id.map(|s| to_js_module_id(&s)),
            }
            .into()
          }),
          loc: origin.loc,
          request: origin.request,
        })
        .collect::<Vec<_>>(),
      id_hints: stats.id_hints,
      hash: stats.hash,
    })
  }
}

#[napi(object, object_from_js = false)]
pub struct JsStatsChunkGroupAsset<'a> {
  pub name: &'a str,
  pub size: f64,
}

impl<'a> From<rspack_core::StatsChunkGroupAsset<'a>> for JsStatsChunkGroupAsset<'a> {
  fn from(stats: rspack_core::StatsChunkGroupAsset<'a>) -> Self {
    Self {
      name: stats.name,
      size: stats.size as f64,
    }
  }
}

#[napi(object, object_from_js = false)]
pub struct JsStatsChunkGroup<'a> {
  pub name: &'a str,
  pub chunks: Vec<&'a str>,
  pub assets: Vec<JsStatsChunkGroupAsset<'a>>,
  pub assets_size: f64,
  pub auxiliary_assets: Option<Vec<JsStatsChunkGroupAsset<'a>>>,
  pub auxiliary_assets_size: Option<f64>,
  pub is_over_size_limit: Option<bool>,
  pub children: Option<JsStatsChunkGroupChildren<'a>>,
  pub child_assets: Option<JsStatsChildGroupChildAssets<'a>>,
}

impl<'a> From<rspack_core::StatsChunkGroup<'a>> for JsStatsChunkGroup<'a> {
  fn from(stats: rspack_core::StatsChunkGroup<'a>) -> Self {
    Self {
      name: stats.name,
      chunks: stats.chunks,
      assets: stats.assets.into_iter().map(Into::into).collect(),
      assets_size: stats.assets_size as f64,
      auxiliary_assets: stats
        .auxiliary_assets
        .map(|assets| assets.into_iter().map(Into::into).collect()),
      auxiliary_assets_size: stats.auxiliary_assets_size.map(|inner| inner as f64),
      children: stats.children.map(|i| i.into()),
      child_assets: stats.child_assets.map(|i| i.into()),
      is_over_size_limit: stats.is_over_size_limit,
    }
  }
}

#[napi(object, object_from_js = false)]
pub struct JsStatsChildGroupChildAssets<'a> {
  pub preload: Option<Vec<&'a str>>,
  pub prefetch: Option<Vec<&'a str>>,
}

impl<'a> From<rspack_core::StatschunkGroupChildAssets<'a>> for JsStatsChildGroupChildAssets<'a> {
  fn from(stats: rspack_core::StatschunkGroupChildAssets<'a>) -> Self {
    Self {
      preload: (!stats.preload.is_empty()).then_some(stats.preload),
      prefetch: (!stats.prefetch.is_empty()).then_some(stats.prefetch),
    }
  }
}

#[napi(object, object_from_js = false)]
pub struct JsStatsChunkGroupChildren<'a> {
  pub preload: Option<Vec<JsStatsChunkGroup<'a>>>,
  pub prefetch: Option<Vec<JsStatsChunkGroup<'a>>>,
}

impl<'a> From<rspack_core::StatsChunkGroupChildren<'a>> for JsStatsChunkGroupChildren<'a> {
  fn from(stats: rspack_core::StatsChunkGroupChildren<'a>) -> Self {
    Self {
      preload: (!stats.preload.is_empty())
        .then(|| stats.preload.into_iter().map(Into::into).collect()),
      prefetch: (!stats.prefetch.is_empty())
        .then(|| stats.prefetch.into_iter().map(Into::into).collect()),
    }
  }
}

#[napi(object, object_from_js = false)]
pub struct JsStatsOptimizationBailout {
  pub inner: String,
}

#[napi(object, object_from_js = false)]
pub struct JsStatsAssetsByChunkName<'a> {
  pub name: &'a str,
  pub files: Vec<&'a str>,
}

impl<'a> From<rspack_core::StatsAssetsByChunkName<'a>> for JsStatsAssetsByChunkName<'a> {
  fn from(stats: rspack_core::StatsAssetsByChunkName<'a>) -> Self {
    Self {
      name: stats.name,
      files: stats.files,
    }
  }
}

#[napi(object, object_to_js = false)]
pub struct JsStatsOptions {
  pub assets: bool,
  pub cached_modules: bool,
  pub chunks: bool,
  pub chunk_group_auxiliary: bool,
  pub chunk_group_children: bool,
  pub chunk_groups: bool,
  pub chunk_modules: bool,
  pub chunk_relations: bool,
  pub depth: bool,
  pub entrypoints: Either<bool, String>,
  pub errors: bool,
  pub hash: bool,
  pub ids: bool,
  pub modules: bool,
  pub module_assets: bool,
  pub nested_modules: bool,
  pub optimization_bailout: bool,
  pub provided_exports: bool,
  pub reasons: bool,
  pub source: bool,
  pub used_exports: bool,
  pub warnings: bool,
}

impl From<JsStatsOptions> for ExtendedStatsOptions {
  fn from(value: JsStatsOptions) -> Self {
    let entrypoints = match value.entrypoints {
      Either::A(b) => EntrypointsStatsOption::Bool(b),
      Either::B(s) => EntrypointsStatsOption::String(s),
    };

    Self {
      assets: value.assets,
      cached_modules: value.cached_modules,
      chunks: value.chunks,
      chunk_group_auxiliary: value.chunk_group_auxiliary,
      chunk_group_children: value.chunk_group_children,
      chunk_groups: value.chunk_groups,
      chunk_modules: value.chunk_modules,
      chunk_relations: value.chunk_relations,
      depth: value.depth,
      entrypoints,
      errors: value.errors,
      hash: value.hash,
      ids: value.ids,
      modules: value.modules,
      module_assets: value.module_assets,
      nested_modules: value.nested_modules,
      optimization_bailout: value.optimization_bailout,
      provided_exports: value.provided_exports,
      reasons: value.reasons,
      source: value.source,
      used_exports: value.used_exports,
      warnings: value.warnings,
    }
  }
}

#[napi(object, object_from_js = false)]
pub struct JsStatsGetAssets<'a> {
  pub assets: Vec<JsStatsAsset<'a>>,
  pub assets_by_chunk_name: Vec<JsStatsAssetsByChunkName<'a>>,
}

#[napi(object, object_from_js = false)]
pub struct JsStatsCompilation<'a> {
  pub assets: Option<Vec<JsStatsAsset<'a>>>,
  pub assets_by_chunk_name: Option<Vec<JsStatsAssetsByChunkName<'a>>>,
  #[napi(ts_type = "Array<JsStatsChunk>")]
  pub chunks: Option<napi_value>,
  pub entrypoints: Option<Vec<JsStatsChunkGroup<'a>>>,
  #[napi(ts_type = "Array<JsStatsError>")]
  pub errors: napi_value,
  pub hash: Option<&'a str>,
  #[napi(ts_type = "Array<JsStatsModule>")]
  pub modules: Option<napi_value>,
  pub named_chunk_groups: Option<Vec<JsStatsChunkGroup<'a>>>,
  #[napi(ts_type = "Array<JsStatsError>")]
  pub warnings: napi_value,
}

pub struct JsStatsCompilationWrapper<'a>(JsStatsCompilation<'a>);

impl<'a> ToNapiValue for JsStatsCompilationWrapper<'a> {
  unsafe fn to_napi_value(env: napi::sys::napi_env, val: Self) -> Result<napi::sys::napi_value> {
    unsafe {
      let napi_value = ToNapiValue::to_napi_value(env, val.0);

      MODULE_DESCRIPTOR_REFS.with(|refs| {
        let mut refs = refs.borrow_mut();
        refs.drain();
      });

      MODULE_COMMON_ATTRIBUTES_REFS.with(|refs| {
        let mut refs = refs.borrow_mut();
        refs.drain();
      });

      napi_value
    }
  }
}

#[napi]
pub struct JsStats {
  inner: SharedReference<JsCompilation, Stats<'static>>,
}

impl JsStats {
  pub fn new(inner: SharedReference<JsCompilation, Stats<'static>>) -> Self {
    Self { inner }
  }
}

#[napi]
impl JsStats {
  #[napi(ts_return_type = "JsStatsCompilation")]
  pub fn to_json(
    &self,
    env: &Env,
    js_options: JsStatsOptions,
  ) -> Result<JsStatsCompilationWrapper<'_>> {
    let options = ExtendedStatsOptions::from(js_options);

    let hash = options.hash.then(|| self.hash()).flatten();

    let (assets, assets_by_chunk_name) = if options.assets {
      let asts = self.assets();
      (Some(asts.assets), Some(asts.assets_by_chunk_name))
    } else {
      (None, None)
    };

    let modules = if options.modules {
      let mds = self.modules(env, &options)?;
      Some(mds)
    } else {
      None
    };

    let chunks = if options.chunks {
      let chks = self.chunks(env, &options)?;
      Some(chks)
    } else {
      None
    };

    let entrypoints = match options.entrypoints {
      EntrypointsStatsOption::Bool(true) | EntrypointsStatsOption::String(_) => {
        Some(self.entrypoints(options.chunk_group_auxiliary, options.chunk_group_children))
      }
      _ => None,
    };

    let named_chunk_groups = options.chunk_groups.then(|| {
      self.named_chunk_groups(options.chunk_group_auxiliary, options.chunk_group_children)
    });

    let errors = self.errors(env)?;

    let warnings = self.warnings(env)?;

    Ok(JsStatsCompilationWrapper(JsStatsCompilation {
      assets,
      assets_by_chunk_name,
      chunks,
      entrypoints,
      errors,
      hash,
      modules,
      named_chunk_groups,
      warnings,
    }))
  }

  fn assets(&self) -> JsStatsGetAssets<'_> {
    let (assets, assets_by_chunk_name) = self.inner.get_assets();
    let assets = assets.into_iter().map(Into::into).collect();
    let assets_by_chunk_name = assets_by_chunk_name.into_iter().map(Into::into).collect();
    JsStatsGetAssets {
      assets,
      assets_by_chunk_name,
    }
  }

  fn modules(&self, env: &Env, options: &ExtendedStatsOptions) -> Result<napi_value> {
    self
      .inner
      .get_modules(options, |res| {
        let val = res
          .into_iter()
          .map(JsStatsModule::try_from)
          .collect::<Result<Vec<_>>>()?;
        unsafe { ToNapiValue::to_napi_value(env.raw(), val) }
      })
      .to_napi_result()?
  }

  fn chunks(&self, env: &Env, options: &ExtendedStatsOptions) -> Result<napi_value> {
    self
      .inner
      .get_chunks(options, |res| {
        let val = res
          .into_iter()
          .map(JsStatsChunk::try_from)
          .collect::<Result<Vec<_>>>()?;
        unsafe { ToNapiValue::to_napi_value(env.raw(), val) }
      })
      .to_napi_result()?
  }

  fn entrypoints(
    &self,
    chunk_group_auxiliary: bool,
    chunk_group_children: bool,
  ) -> Vec<JsStatsChunkGroup<'_>> {
    self
      .inner
      .get_entrypoints(chunk_group_auxiliary, chunk_group_children)
      .into_iter()
      .map(Into::into)
      .collect()
  }

  fn named_chunk_groups(
    &self,
    chunk_group_auxiliary: bool,
    chunk_group_children: bool,
  ) -> Vec<JsStatsChunkGroup<'_>> {
    self
      .inner
      .get_named_chunk_groups(chunk_group_auxiliary, chunk_group_children)
      .into_iter()
      .map(Into::into)
      .collect()
  }

  fn errors(&self, env: &Env) -> napi::Result<napi_value> {
    self.inner.get_errors(|errors| {
      let val = errors
        .into_iter()
        .map(JsStatsError::from)
        .collect::<Vec<_>>();
      unsafe { ToNapiValue::to_napi_value(env.raw(), val) }
    })
  }

  fn warnings(&self, env: &Env) -> napi::Result<napi_value> {
    self.inner.get_warnings(|warnings| {
      let val = warnings
        .into_iter()
        .map(JsStatsError::from)
        .collect::<Vec<_>>();
      unsafe { ToNapiValue::to_napi_value(env.raw(), val) }
    })
  }

  #[napi]
  pub fn get_logging(&self, accepted_types: u32) -> Vec<JsStatsLogging<'_>> {
    self
      .inner
      .get_logging()
      .into_iter()
      .filter(|log| {
        let bit = log.1.to_bit_flag();
        accepted_types & bit == bit
      })
      .map(Into::into)
      .collect()
  }

  fn hash(&self) -> Option<&str> {
    self.inner.get_hash()
  }
}

pub fn create_stats_warnings<'a>(
  env: &'a Env,
  compilation: &rspack_core::Compilation,
  warnings: Vec<RspackError>,
  colored: Option<bool>,
) -> Result<Array<'a>> {
  let module_graph = compilation.get_module_graph();

  let mut diagnostics = warnings
    .into_iter()
    .map(|warning| warning.into_diagnostic(Severity::Warning))
    .collect::<Vec<_>>();

  let stats_warnings = rspack_core::create_stats_errors(
    compilation,
    &module_graph,
    &mut diagnostics,
    colored.unwrap_or(false),
  );

  let mut array = env.create_array(stats_warnings.len() as u32)?;
  let raw_env = env.raw();
  for (i, warning) in stats_warnings.into_iter().enumerate() {
    let js_warning = JsStatsError::from(warning);
    let napi_val = unsafe { ToNapiValue::to_napi_value(raw_env, js_warning)? };
    let object = unsafe { Object::from_napi_value(raw_env, napi_val)? };
    array.set_element(i as u32, object)?;
  }

  MODULE_DESCRIPTOR_REFS.with(|refs| {
    let mut refs = refs.borrow_mut();
    refs.drain();
  });
  MODULE_COMMON_ATTRIBUTES_REFS.with(|refs| {
    let mut refs = refs.borrow_mut();
    refs.drain();
  });
  Ok(array)
}
