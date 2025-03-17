use std::cell::RefCell;

use napi_derive::napi;
use rspack_collections::Identifier;
use rspack_core::{
  rspack_sources::{RawBufferSource, RawSource, Source},
  EntrypointsStatsOption, ExtendedStatsOptions, Stats, StatsChunk, StatsModule, StatsUsedExports,
};
use rspack_napi::{
  napi::{
    bindgen_prelude::{Buffer, Result, SharedReference, ToNapiValue},
    Either,
  },
  OneShotRef,
};
use rspack_util::itoa;
use rustc_hash::FxHashMap as HashMap;

use crate::{identifier::JsIdentifier, JsCompilation};

thread_local! {
  static MODULE_DESCRIPTOR_REFS: RefCell<HashMap<Identifier, OneShotRef>> = Default::default();
  static MODULE_COMMON_ATTRIBUTES_REFS: RefCell<HashMap<Identifier, OneShotRef>> = Default::default();
}

#[napi(object, object_from_js = false)]
pub struct JsModuleDescriptor {
  #[napi(ts_type = "string")]
  pub identifier: JsIdentifier,
  pub name: String,
  pub id: Option<String>,
}

pub struct JsModuleDescriptorWrapper(JsModuleDescriptor);

impl JsModuleDescriptorWrapper {
  pub fn raw(&self) -> &JsModuleDescriptor {
    &self.0
  }
}

impl ToNapiValue for JsModuleDescriptorWrapper {
  unsafe fn to_napi_value(env: napi::sys::napi_env, val: Self) -> Result<napi::sys::napi_value> {
    MODULE_DESCRIPTOR_REFS.with(|refs| {
      let id = val.0.identifier.raw();
      let mut refs = refs.borrow_mut();
      match refs.entry(id) {
        std::collections::hash_map::Entry::Occupied(entry) => {
          let r = entry.get();
          ToNapiValue::to_napi_value(env, r)
        }
        std::collections::hash_map::Entry::Vacant(entry) => {
          let r = OneShotRef::new(env, val.0)?;
          let r = entry.insert(r);
          ToNapiValue::to_napi_value(env, r)
        }
      }
    })
  }
}

impl From<JsModuleDescriptor> for JsModuleDescriptorWrapper {
  fn from(value: JsModuleDescriptor) -> Self {
    Self(value)
  }
}

#[napi(object, object_from_js = false)]
pub struct JsStatsError {
  #[napi(ts_type = "JsModuleDescriptor")]
  pub module_descriptor: Option<JsModuleDescriptorWrapper>,
  pub message: String,
  pub chunk_name: Option<String>,
  pub chunk_entry: Option<bool>,
  pub chunk_initial: Option<bool>,
  pub loc: Option<String>,
  pub file: Option<String>,
  pub chunk_id: Option<String>,
  pub details: Option<String>,
  pub stack: Option<String>,
  pub module_trace: Vec<JsStatsModuleTrace>,
}

impl From<rspack_core::StatsError<'_>> for JsStatsError {
  fn from(stats: rspack_core::StatsError) -> Self {
    Self {
      module_descriptor: stats.module_identifier.map(|identifier| {
        JsModuleDescriptor {
          identifier: identifier.into(),
          name: stats.module_name.unwrap_or_default().into_owned(),
          id: stats.module_id.map(|s| s.to_string()),
        }
        .into()
      }),
      message: stats.message,
      loc: stats.loc,
      file: stats.file.map(|f| f.as_str().to_string()),
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
pub struct JsStatsWarning {
  #[napi(ts_type = "JsModuleDescriptor")]
  pub module_descriptor: Option<JsModuleDescriptorWrapper>,
  pub message: String,
  pub chunk_name: Option<String>,
  pub chunk_entry: Option<bool>,
  pub chunk_initial: Option<bool>,
  pub file: Option<String>,
  pub chunk_id: Option<String>,
  pub details: Option<String>,
  pub stack: Option<String>,
  pub module_trace: Vec<JsStatsModuleTrace>,
}

impl From<rspack_core::StatsWarning<'_>> for JsStatsWarning {
  fn from(stats: rspack_core::StatsWarning) -> Self {
    Self {
      module_descriptor: stats.module_identifier.map(|identifier| {
        JsModuleDescriptor {
          identifier: identifier.into(),
          name: stats.module_name.unwrap_or_default().into_owned(),
          id: stats.module_id.map(|s| s.to_string()),
        }
        .into()
      }),
      message: stats.message,
      file: stats.file.map(|f| f.as_str().to_string()),
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
pub struct JsStatsModuleTrace {
  pub origin: JsStatsModuleTraceModule,
  pub module: JsStatsModuleTraceModule,
  pub dependencies: Vec<JsStatsModuleTraceDependency>,
}

impl From<rspack_core::StatsModuleTrace> for JsStatsModuleTrace {
  fn from(stats: rspack_core::StatsModuleTrace) -> Self {
    Self {
      origin: stats.origin.into(),
      module: stats.module.into(),
      dependencies: stats.dependencies.into_iter().map(Into::into).collect(),
    }
  }
}

#[napi(object, object_from_js = false)]
pub struct JsStatsModuleTraceModule {
  #[napi(ts_type = "JsModuleDescriptor")]
  pub module_descriptor: JsModuleDescriptorWrapper,
}

impl From<rspack_core::StatsErrorModuleTraceModule> for JsStatsModuleTraceModule {
  fn from(stats: rspack_core::StatsErrorModuleTraceModule) -> Self {
    Self {
      module_descriptor: JsModuleDescriptor {
        identifier: stats.identifier.into(),
        name: stats.name,
        id: stats.id,
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
pub struct JsStatsLogging {
  pub name: String,
  pub r#type: String,
  pub args: Option<Vec<String>>,
  pub trace: Option<Vec<String>>,
}

impl From<(String, rspack_core::LogType)> for JsStatsLogging {
  fn from(value: (String, rspack_core::LogType)) -> Self {
    match value.1 {
      rspack_core::LogType::Error { message, trace } => Self {
        name: value.0,
        r#type: "error".to_string(),
        args: Some(vec![message]),
        trace: Some(trace),
      },
      rspack_core::LogType::Warn { message, trace } => Self {
        name: value.0,
        r#type: "warn".to_string(),
        args: Some(vec![message]),
        trace: Some(trace),
      },
      rspack_core::LogType::Info { message } => Self {
        name: value.0,
        r#type: "info".to_string(),
        args: Some(vec![message]),
        trace: None,
      },
      rspack_core::LogType::Log { message } => Self {
        name: value.0,
        r#type: "log".to_string(),
        args: Some(vec![message]),
        trace: None,
      },
      rspack_core::LogType::Debug { message } => Self {
        name: value.0,
        r#type: "debug".to_string(),
        args: Some(vec![message]),
        trace: None,
      },
      rspack_core::LogType::Trace { message, trace } => Self {
        name: value.0,
        r#type: "trace".to_string(),
        args: Some(vec![message]),
        trace: Some(trace),
      },
      rspack_core::LogType::Group { message } => Self {
        name: value.0,
        r#type: "group".to_string(),
        args: Some(vec![message]),
        trace: None,
      },
      rspack_core::LogType::GroupCollapsed { message } => Self {
        name: value.0,
        r#type: "groupCollapsed".to_string(),
        args: Some(vec![message]),
        trace: None,
      },
      rspack_core::LogType::GroupEnd => Self {
        name: value.0,
        r#type: "groupEnd".to_string(),
        args: None,
        trace: None,
      },
      rspack_core::LogType::Profile { label } => Self {
        name: value.0,
        r#type: "profile".to_string(),
        args: Some(vec![label.to_string()]),
        trace: None,
      },
      rspack_core::LogType::ProfileEnd { label } => Self {
        name: value.0,
        r#type: "profileEnd".to_string(),
        args: Some(vec![label.to_string()]),
        trace: None,
      },
      rspack_core::LogType::Time {
        label,
        secs,
        subsec_nanos,
      } => Self {
        name: value.0,
        r#type: "time".to_string(),
        args: Some(vec![format!(
          "{}: {} ms",
          label,
          itoa!(secs * 1000 + subsec_nanos as u64 / 1000000)
        )]),
        trace: None,
      },
      rspack_core::LogType::Clear => Self {
        name: value.0,
        r#type: "clear".to_string(),
        args: None,
        trace: None,
      },
      rspack_core::LogType::Status { message } => Self {
        name: value.0,
        r#type: "status".to_string(),
        args: Some(vec![message]),
        trace: None,
      },
      rspack_core::LogType::Cache { label, hit, total } => Self {
        name: value.0,
        r#type: "cache".to_string(),
        args: Some(vec![format!(
          "{}: {:.1}% ({}/{})",
          label,
          if total == 0 {
            0 as f32
          } else {
            hit as f32 / total as f32 * 100_f32
          },
          itoa!(hit),
          itoa!(total),
        )]),
        trace: None,
      },
    }
  }
}

#[napi(object, object_from_js = false)]
pub struct JsStatsAsset {
  pub r#type: &'static str,
  pub name: String,
  pub info: JsStatsAssetInfo,
  pub size: f64,
  pub emitted: bool,
  pub chunk_names: Vec<String>,
  pub chunk_id_hints: Vec<String>,
  pub chunks: Vec<Option<String>>,
  pub auxiliary_chunk_names: Vec<String>,
  pub auxiliary_chunk_id_hints: Vec<String>,
  pub auxiliary_chunks: Vec<Option<String>>,
}

impl From<rspack_core::StatsAsset> for JsStatsAsset {
  fn from(stats: rspack_core::StatsAsset) -> Self {
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
pub struct JsStatsAssetInfo {
  pub minimized: Option<bool>,
  pub development: Option<bool>,
  pub hot_module_replacement: Option<bool>,
  pub source_filename: Option<String>,
  pub copied: Option<bool>,
  pub immutable: Option<bool>,
  pub javascript_module: Option<bool>,
  pub chunkhash: Vec<String>,
  pub contenthash: Vec<String>,
  pub fullhash: Vec<String>,
  pub related: Vec<JsStatsAssetInfoRelated>,
  pub is_over_size_limit: Option<bool>,
}

impl From<rspack_core::StatsAssetInfo> for JsStatsAssetInfo {
  fn from(stats: rspack_core::StatsAssetInfo) -> Self {
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
pub struct JsStatsAssetInfoRelated {
  pub name: String,
  pub value: Vec<String>,
}

impl From<rspack_core::StatsAssetInfoRelated> for JsStatsAssetInfoRelated {
  fn from(stats: rspack_core::StatsAssetInfoRelated) -> Self {
    Self {
      name: stats.name,
      value: stats.value,
    }
  }
}

type JsStatsModuleSource = Either<String, Buffer>;
type JsStatsUsedExports = Either<String, Vec<String>>;

#[napi(object, object_from_js = false)]
pub struct JsStatsModuleCommonAttributes {
  pub r#type: &'static str,
  pub module_type: &'static str,
  pub layer: Option<String>,
  pub size: f64,
  pub sizes: Vec<JsStatsSize>,
  pub built: bool,
  pub code_generated: bool,
  pub build_time_executed: bool,
  pub cached: bool,

  // module$visible
  #[napi(ts_type = "JsModuleDescriptor")]
  pub module_descriptor: Option<JsModuleDescriptorWrapper>,
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
  pub chunks: Option<Vec<String>>,

  // moduleAssets
  pub assets: Option<Vec<String>>,

  // reasons
  pub reasons: Option<Vec<JsStatsModuleReason>>,

  // providedExports
  pub provided_exports: Option<Vec<String>>,

  // optimizationBailout
  pub optimization_bailout: Option<Vec<String>>,

  // depth
  pub depth: Option<u32>,

  // source
  pub source: Option<Either<String, Buffer>>,
}

pub struct JsStatsModuleCommonAttributesWrapper(JsStatsModuleCommonAttributes);

impl From<JsStatsModuleCommonAttributes> for JsStatsModuleCommonAttributesWrapper {
  fn from(value: JsStatsModuleCommonAttributes) -> Self {
    JsStatsModuleCommonAttributesWrapper(value)
  }
}

impl ToNapiValue for JsStatsModuleCommonAttributesWrapper {
  unsafe fn to_napi_value(env: napi::sys::napi_env, val: Self) -> Result<napi::sys::napi_value> {
    MODULE_COMMON_ATTRIBUTES_REFS.with(|refs| {
      match val
        .0
        .module_descriptor
        .as_ref()
        .map(|d| d.raw().identifier.raw())
        .as_ref()
      {
        Some(id) => {
          let mut refs = refs.borrow_mut();
          match refs.entry(*id) {
            std::collections::hash_map::Entry::Occupied(entry) => {
              let r = entry.get();
              ToNapiValue::to_napi_value(env, r)
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
              let r = OneShotRef::new(env, val.0)?;
              let r = entry.insert(r);
              ToNapiValue::to_napi_value(env, r)
            }
          }
        }
        None => ToNapiValue::to_napi_value(env, val.0),
      }
    })
  }
}

#[napi(object, object_from_js = false)]
pub struct JsStatsModule {
  #[napi(ts_type = "JsStatsModuleCommonAttributes")]
  pub common_attributes: JsStatsModuleCommonAttributesWrapper,
  pub dependent: Option<bool>,
  #[napi(ts_type = "JsModuleDescriptor")]
  pub issuer_descriptor: Option<JsModuleDescriptorWrapper>,
  pub issuer_path: Option<Vec<JsStatsModuleIssuer>>,
  pub used_exports: Option<Either<String, Vec<String>>>,
  pub modules: Option<Vec<JsStatsModule>>,
}

impl TryFrom<StatsModule<'_>> for JsStatsModule {
  type Error = napi::Error;

  fn try_from(stats: StatsModule) -> std::result::Result<Self, Self::Error> {
    let source = stats.source.map(|source| {
      if let Some(raw_source) = source.as_any().downcast_ref::<RawBufferSource>() {
        return JsStatsModuleSource::B(Buffer::from(raw_source.buffer().to_vec()));
      }
      if let Some(raw_source) = source.as_any().downcast_ref::<RawSource>() {
        if raw_source.is_buffer() {
          return JsStatsModuleSource::B(Buffer::from(raw_source.buffer().to_vec()));
        }
      }
      JsStatsModuleSource::A(source.source().to_string())
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
      .map_err(|e| napi::Error::from_reason(e.to_string()))?;

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
        name: stats.name.unwrap_or_default().into_owned(),
        id: stats.id.map(|s| s.to_string()),
      }
      .into()
    });

    let common_attributes: JsStatsModuleCommonAttributesWrapper = JsStatsModuleCommonAttributes {
      r#type: stats.r#type,
      module_type: stats.module_type.as_str(),
      layer: stats.layer.map(|i| i.into_owned()),
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
      provided_exports: stats
        .provided_exports
        .map(|exports| exports.into_iter().map(|i| i.to_string()).collect()),
      optimization_bailout: stats.optimization_bailout.map(|bailout| bailout.to_vec()),
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

    let issuer_descriptor: Option<JsModuleDescriptorWrapper> = stats.issuer.map(|identifier| {
      JsModuleDescriptor {
        identifier: identifier.into(),
        name: stats.issuer_name.unwrap_or_default().into_owned(),
        id: stats.issuer_id.map(|s| s.to_string()),
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
        StatsUsedExports::Bool(b) => JsStatsUsedExports::A(b.to_string()),
        StatsUsedExports::Vec(v) => {
          JsStatsUsedExports::B(v.into_iter().map(|i| i.to_string()).collect())
        }
        StatsUsedExports::Null => JsStatsUsedExports::A("null".to_string()),
      }),
      modules,
    })
  }
}

#[napi(object, object_from_js = false)]
pub struct JsStatsModuleProfile {
  pub factory: JsStatsMillisecond,
  pub building: JsStatsMillisecond,
}

impl From<rspack_core::StatsModuleProfile> for JsStatsModuleProfile {
  fn from(value: rspack_core::StatsModuleProfile) -> Self {
    Self {
      factory: value.factory.into(),
      building: value.building.into(),
    }
  }
}

#[napi(object, object_from_js = false)]
pub struct JsStatsMillisecond {
  pub secs: u32,
  pub subsec_millis: u32,
}

impl From<rspack_core::StatsMillisecond> for JsStatsMillisecond {
  fn from(value: rspack_core::StatsMillisecond) -> Self {
    Self {
      secs: value.secs as u32,
      subsec_millis: value.subsec_millis,
    }
  }
}

#[napi(object, object_from_js = false)]
pub struct JsStatsModuleIssuer {
  #[napi(ts_type = "JsModuleDescriptor")]
  pub module_descriptor: JsModuleDescriptorWrapper,
}

impl From<rspack_core::StatsModuleIssuer<'_>> for JsStatsModuleIssuer {
  fn from(stats: rspack_core::StatsModuleIssuer) -> Self {
    Self {
      module_descriptor: JsModuleDescriptor {
        identifier: stats.identifier.into(),
        name: stats.name.into_owned(),
        id: stats.id.map(|s| s.to_string()),
      }
      .into(),
    }
  }
}

#[napi(object, object_from_js = false)]
pub struct JsStatsModuleReason {
  #[napi(ts_type = "JsModuleDescriptor")]
  pub module_descriptor: Option<JsModuleDescriptorWrapper>,
  #[napi(ts_type = "JsModuleDescriptor")]
  pub resolved_module_descriptor: Option<JsModuleDescriptorWrapper>,
  pub module_chunks: Option<u32>,
  pub r#type: Option<&'static str>,
  pub user_request: Option<String>,
  pub explanation: Option<&'static str>,
  pub active: bool,
  pub loc: Option<String>,
}

impl From<rspack_core::StatsModuleReason<'_>> for JsStatsModuleReason {
  fn from(stats: rspack_core::StatsModuleReason) -> Self {
    Self {
      module_descriptor: stats.module_identifier.map(|identifier| {
        JsModuleDescriptor {
          identifier: identifier.into(),
          name: stats.module_name.unwrap_or_default().into_owned(),
          id: stats.module_id.map(|s| s.to_string()),
        }
        .into()
      }),
      resolved_module_descriptor: stats.resolved_module_identifier.map(|identifier| {
        JsModuleDescriptor {
          identifier: identifier.into(),
          name: stats.resolved_module_name.unwrap_or_default().into_owned(),
          id: stats.resolved_module_id.map(|s| s.to_string()),
        }
        .into()
      }),
      module_chunks: stats.module_chunks,
      r#type: stats.r#type,
      user_request: stats.user_request.map(|i| i.to_owned()),
      explanation: stats.explanation,
      active: stats.active,
      loc: stats.loc,
    }
  }
}

#[napi(object, object_from_js = false)]
pub struct JsOriginRecord {
  #[napi(ts_type = "JsModuleDescriptor")]
  pub module_descriptor: Option<JsModuleDescriptorWrapper>,
  pub loc: String,
  pub request: String,
}

#[napi(object, object_from_js = false)]
pub struct JsStatsSize {
  pub source_type: String,
  pub size: f64,
}

#[napi(object, object_from_js = false)]
pub struct JsStatsChunk {
  pub r#type: String,
  pub files: Vec<String>,
  pub auxiliary_files: Vec<String>,
  pub id: Option<String>,
  pub id_hints: Vec<String>,
  pub hash: Option<String>,
  pub entry: bool,
  pub initial: bool,
  pub names: Vec<String>,
  pub size: f64,
  pub parents: Option<Vec<String>>,
  pub children: Option<Vec<String>>,
  pub siblings: Option<Vec<String>>,
  pub children_by_order: HashMap<String, Vec<String>>,
  pub runtime: Vec<String>,
  pub reason: Option<String>,
  pub rendered: bool,
  pub sizes: Vec<JsStatsSize>,
  pub origins: Vec<JsOriginRecord>,
  pub modules: Option<Vec<JsStatsModule>>,
}

impl TryFrom<StatsChunk<'_>> for JsStatsChunk {
  type Error = napi::Error;

  fn try_from(stats: StatsChunk<'_>) -> std::result::Result<Self, Self::Error> {
    let mut runtime = stats
      .runtime
      .iter()
      .map(|r| r.to_string())
      .collect::<Vec<_>>();
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
      r#type: stats.r#type.to_string(),
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
        .iter()
        .map(|(order, children)| (order.to_string(), children.to_owned()))
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
              name: origin.module_name,
              id: Some(origin.module_id),
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
pub struct JsStatsChunkGroupAsset {
  pub name: String,
  pub size: f64,
}

impl From<rspack_core::StatsChunkGroupAsset> for JsStatsChunkGroupAsset {
  fn from(stats: rspack_core::StatsChunkGroupAsset) -> Self {
    Self {
      name: stats.name,
      size: stats.size as f64,
    }
  }
}

#[napi(object, object_from_js = false)]
pub struct JsStatsChunkGroup {
  pub name: String,
  pub chunks: Vec<String>,
  pub assets: Vec<JsStatsChunkGroupAsset>,
  pub assets_size: f64,
  pub auxiliary_assets: Option<Vec<JsStatsChunkGroupAsset>>,
  pub auxiliary_assets_size: Option<f64>,
  pub is_over_size_limit: Option<bool>,
  pub children: Option<JsStatsChunkGroupChildren>,
  pub child_assets: Option<JsStatsChildGroupChildAssets>,
}

impl From<rspack_core::StatsChunkGroup> for JsStatsChunkGroup {
  fn from(stats: rspack_core::StatsChunkGroup) -> Self {
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
pub struct JsStatsChildGroupChildAssets {
  pub preload: Option<Vec<String>>,
  pub prefetch: Option<Vec<String>>,
}

impl From<rspack_core::StatschunkGroupChildAssets> for JsStatsChildGroupChildAssets {
  fn from(stats: rspack_core::StatschunkGroupChildAssets) -> Self {
    Self {
      preload: (!stats.preload.is_empty()).then_some(stats.preload),
      prefetch: (!stats.prefetch.is_empty()).then_some(stats.prefetch),
    }
  }
}

#[napi(object, object_from_js = false)]
pub struct JsStatsChunkGroupChildren {
  pub preload: Option<Vec<JsStatsChunkGroup>>,
  pub prefetch: Option<Vec<JsStatsChunkGroup>>,
}

impl From<rspack_core::StatsChunkGroupChildren> for JsStatsChunkGroupChildren {
  fn from(stats: rspack_core::StatsChunkGroupChildren) -> Self {
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
pub struct JsStatsAssetsByChunkName {
  pub name: String,
  pub files: Vec<String>,
}

impl From<rspack_core::StatsAssetsByChunkName> for JsStatsAssetsByChunkName {
  fn from(stats: rspack_core::StatsAssetsByChunkName) -> Self {
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
pub struct JsStatsGetAssets {
  pub assets: Vec<JsStatsAsset>,
  pub assets_by_chunk_name: Vec<JsStatsAssetsByChunkName>,
}

#[napi(object, object_from_js = false)]
pub struct JsStatsCompilation {
  pub assets: Option<Vec<JsStatsAsset>>,
  pub assets_by_chunk_name: Option<Vec<JsStatsAssetsByChunkName>>,
  pub chunks: Option<Vec<JsStatsChunk>>,
  pub entrypoints: Option<Vec<JsStatsChunkGroup>>,
  pub errors: Vec<JsStatsError>,
  pub hash: Option<String>,
  pub modules: Option<Vec<JsStatsModule>>,
  pub named_chunk_groups: Option<Vec<JsStatsChunkGroup>>,
  pub warnings: Vec<JsStatsWarning>,
}

pub struct JsStatsCompilationWrapper(JsStatsCompilation);

impl ToNapiValue for JsStatsCompilationWrapper {
  unsafe fn to_napi_value(env: napi::sys::napi_env, val: Self) -> Result<napi::sys::napi_value> {
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
  pub fn to_json(&self, js_options: JsStatsOptions) -> Result<JsStatsCompilationWrapper> {
    let options = ExtendedStatsOptions::from(js_options);

    let hash = options.hash.then(|| self.hash()).flatten();

    let (assets, assets_by_chunk_name) = if options.assets {
      let asts = self.assets();
      (Some(asts.assets), Some(asts.assets_by_chunk_name))
    } else {
      (None, None)
    };

    let modules = if options.modules {
      let mds = self.modules(&options)?;
      Some(mds)
    } else {
      None
    };

    let chunks = if options.chunks {
      let chks = self.chunks(&options)?;
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

    let errors = self.errors();

    let warnings = self.warnings();

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

  fn assets(&self) -> JsStatsGetAssets {
    let (assets, assets_by_chunk_name) = self.inner.get_assets();
    let assets = assets.into_iter().map(Into::into).collect();
    let assets_by_chunk_name = assets_by_chunk_name.into_iter().map(Into::into).collect();
    JsStatsGetAssets {
      assets,
      assets_by_chunk_name,
    }
  }

  fn modules(&self, options: &ExtendedStatsOptions) -> Result<Vec<JsStatsModule>> {
    self
      .inner
      .get_modules(options, |res| {
        res.into_iter().map(JsStatsModule::try_from).collect()
      })
      .map_err(|e| napi::Error::from_reason(e.to_string()))?
  }

  fn chunks(&self, options: &ExtendedStatsOptions) -> Result<Vec<JsStatsChunk>> {
    self
      .inner
      .get_chunks(options, |res| {
        res.into_iter().map(JsStatsChunk::try_from).collect()
      })
      .map_err(|e| napi::Error::from_reason(e.to_string()))?
  }

  fn entrypoints(
    &self,
    chunk_group_auxiliary: bool,
    chunk_group_children: bool,
  ) -> Vec<JsStatsChunkGroup> {
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
  ) -> Vec<JsStatsChunkGroup> {
    self
      .inner
      .get_named_chunk_groups(chunk_group_auxiliary, chunk_group_children)
      .into_iter()
      .map(Into::into)
      .collect()
  }

  fn errors(&self) -> Vec<JsStatsError> {
    self
      .inner
      .get_errors()
      .into_iter()
      .map(Into::into)
      .collect()
  }

  fn warnings(&self) -> Vec<JsStatsWarning> {
    self
      .inner
      .get_warnings()
      .into_iter()
      .map(Into::into)
      .collect()
  }

  #[napi]
  pub fn has_warnings(&self) -> bool {
    !self.inner.get_warnings().is_empty()
  }

  #[napi]
  pub fn has_errors(&self) -> bool {
    !self.inner.get_errors().is_empty()
  }

  #[napi]
  pub fn get_logging(&self, accepted_types: u32) -> Vec<JsStatsLogging> {
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

  fn hash(&self) -> Option<String> {
    self.inner.get_hash().map(|hash| hash.to_string())
  }
}
