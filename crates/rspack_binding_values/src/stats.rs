use napi::bindgen_prelude::FromNapiValue;
use napi_derive::napi;
use rspack_core::{ExtendedStatsOptions, Stats, StatsChunk, StatsModule, StatsUsedExports};
use rspack_napi::napi::bindgen_prelude::Buffer;
use rspack_napi::napi::{
  bindgen_prelude::{Result, SharedReference},
  Either,
};
use rustc_hash::FxHashMap as HashMap;

use super::{JsCompilation, ToJsCompatSource};
use crate::identifier::JsIdentifier;

#[napi(object, object_from_js = false)]
pub struct JsStatsError {
  pub message: String,
  pub chunk_name: Option<String>,
  pub chunk_entry: Option<bool>,
  pub chunk_initial: Option<bool>,
  pub file: Option<String>,
  #[napi(ts_type = "Buffer")]
  pub module_identifier: Option<JsIdentifier>,
  pub module_name: Option<String>,
  pub module_id: Option<String>,
  pub chunk_id: Option<String>,
  pub details: Option<String>,
  pub stack: Option<String>,
  pub module_trace: Vec<JsStatsModuleTrace>,
}

impl FromNapiValue for JsStatsError {
  unsafe fn from_napi_value(
    _env: napi::sys::napi_env,
    _napi_val: napi::sys::napi_value,
  ) -> Result<Self> {
    unreachable!()
  }
}

impl From<rspack_core::StatsError<'_>> for JsStatsError {
  fn from(stats: rspack_core::StatsError) -> Self {
    Self {
      message: stats.message,
      module_identifier: stats.module_identifier.map(JsIdentifier::from),
      module_name: stats.module_name.map(|i| i.into_owned()),
      module_id: stats.module_id.map(|i| i.to_owned()),
      file: stats.file.map(|f| f.to_string_lossy().to_string()),
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
  pub message: String,
  pub chunk_name: Option<String>,
  pub chunk_entry: Option<bool>,
  pub chunk_initial: Option<bool>,
  pub file: Option<String>,
  #[napi(ts_type = "Buffer")]
  pub module_identifier: Option<JsIdentifier>,
  pub module_name: Option<String>,
  pub module_id: Option<String>,
  pub chunk_id: Option<String>,
  pub details: Option<String>,
  pub stack: Option<String>,
  pub module_trace: Vec<JsStatsModuleTrace>,
}

impl FromNapiValue for JsStatsWarning {
  unsafe fn from_napi_value(
    _env: napi::sys::napi_env,
    _napi_val: napi::sys::napi_value,
  ) -> Result<Self> {
    unreachable!()
  }
}

impl From<rspack_core::StatsWarning<'_>> for JsStatsWarning {
  fn from(stats: rspack_core::StatsWarning) -> Self {
    Self {
      message: stats.message,
      module_identifier: stats.module_identifier.map(JsIdentifier::from),
      module_name: stats.module_name.map(|i| i.into_owned()),
      module_id: stats.module_id.map(|i| i.to_owned()),
      file: stats.file.map(|f| f.to_string_lossy().to_string()),
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

#[napi(object)]
#[derive(Debug)]
pub struct JsStatsModuleTrace {
  pub origin: JsStatsModuleTraceModule,
  pub module: JsStatsModuleTraceModule,
}

impl From<rspack_core::StatsModuleTrace> for JsStatsModuleTrace {
  fn from(stats: rspack_core::StatsModuleTrace) -> Self {
    Self {
      origin: stats.origin.into(),
      module: stats.module.into(),
    }
  }
}

#[napi(object, object_from_js = false)]
#[derive(Debug)]
pub struct JsStatsModuleTraceModule {
  #[napi(ts_type = "Buffer")]
  pub identifier: JsIdentifier,
  pub name: Option<String>,
  pub id: Option<String>,
}

impl FromNapiValue for JsStatsModuleTraceModule {
  unsafe fn from_napi_value(
    _env: napi::sys::napi_env,
    _napi_val: napi::sys::napi_value,
  ) -> Result<Self> {
    unreachable!()
  }
}

impl From<rspack_core::StatsErrorModuleTraceModule> for JsStatsModuleTraceModule {
  fn from(stats: rspack_core::StatsErrorModuleTraceModule) -> Self {
    Self {
      identifier: JsIdentifier::from(stats.identifier),
      name: stats.name,
      id: stats.id,
    }
  }
}

#[napi(object)]
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
          secs * 1000 + subsec_nanos as u64 / 1000000
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
          hit,
          total,
        )]),
        trace: None,
      },
    }
  }
}

#[napi(object)]
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

#[napi(object)]
pub struct JsStatsAssetInfo {
  pub minimized: bool,
  pub development: bool,
  pub hot_module_replacement: bool,
  pub source_filename: Option<String>,
  pub immutable: bool,
  pub javascript_module: Option<bool>,
  pub chunkhash: Vec<String>,
  pub contenthash: Vec<String>,
  pub fullhash: Vec<String>,
  pub related: Vec<JsStatsAssetInfoRelated>,
}

impl From<rspack_core::StatsAssetInfo> for JsStatsAssetInfo {
  fn from(stats: rspack_core::StatsAssetInfo) -> Self {
    Self {
      minimized: stats.minimized,
      development: stats.development,
      hot_module_replacement: stats.hot_module_replacement,
      source_filename: stats.source_filename,
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
    }
  }
}

#[napi(object)]
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
pub struct JsStatsModule {
  pub r#type: &'static str,
  pub module_type: &'static str,
  #[napi(ts_type = "Buffer")]
  pub identifier: Option<JsIdentifier>,
  pub name: Option<String>,
  pub id: Option<String>,
  pub chunks: Option<Vec<Option<String>>>,
  pub size: f64,
  pub depth: Option<u32>,
  pub dependent: Option<bool>,
  pub issuer: Option<String>,
  pub issuer_name: Option<String>,
  pub issuer_id: Option<String>,
  pub issuer_path: Option<Vec<JsStatsModuleIssuer>>,
  pub name_for_condition: Option<String>,
  pub assets: Option<Vec<String>>,
  pub source: Option<Either<String, Buffer>>,
  pub orphan: Option<bool>,
  pub provided_exports: Option<Vec<String>>,
  pub used_exports: Option<Either<String, Vec<String>>>,
  pub optimization_bailout: Option<Vec<String>>,
  pub pre_order_index: Option<u32>,
  pub post_order_index: Option<u32>,
  pub built: bool,
  pub code_generated: bool,
  pub build_time_executed: bool,
  pub cached: bool,
  pub cacheable: Option<bool>,
  pub optional: Option<bool>,
  pub failed: Option<bool>,
  pub errors: Option<u32>,
  pub warnings: Option<u32>,
  pub sizes: Vec<JsStatsSize>,
  pub profile: Option<JsStatsModuleProfile>,
  pub reasons: Option<Vec<JsStatsModuleReason>>,
  pub modules: Option<Vec<JsStatsModule>>,
}

impl FromNapiValue for JsStatsModule {
  unsafe fn from_napi_value(
    _env: napi::sys::napi_env,
    _napi_val: napi::sys::napi_value,
  ) -> Result<Self> {
    unreachable!()
  }
}

impl TryFrom<StatsModule<'_>> for JsStatsModule {
  type Error = napi::Error;

  fn try_from(stats: StatsModule) -> std::result::Result<Self, Self::Error> {
    let source = stats
      .source
      .map(|source| {
        source.to_js_compat_source().map(|js_compat_source| {
          if js_compat_source.is_raw && js_compat_source.is_buffer {
            JsStatsModuleSource::B(js_compat_source.source)
          } else {
            let s = String::from_utf8_lossy(js_compat_source.source.as_ref()).to_string();
            JsStatsModuleSource::A(s)
          }
        })
      })
      .transpose()
      .map_err(|e| napi::Error::from_reason(e.to_string()))?;

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

    Ok(Self {
      r#type: stats.r#type,
      name: stats.name.map(|n| n.to_string()),
      size: stats.size,
      sizes,
      depth: stats.depth.map(|d| d as u32),
      chunks: stats.chunks,
      module_type: stats.module_type.as_str(),
      identifier: stats.identifier.map(JsIdentifier::from),
      id: stats.id.map(|i| i.to_owned()),
      dependent: stats.dependent,
      issuer: stats.issuer.map(|i| i.to_owned()),
      issuer_name: stats.issuer_name.map(|i| i.into_owned()),
      issuer_id: stats.issuer_id.map(|i| i.to_owned()),
      name_for_condition: stats.name_for_condition,
      issuer_path: stats
        .issuer_path
        .map(|path| path.into_iter().map(Into::into).collect()),
      reasons,
      assets: stats.assets,
      source,
      profile: stats.profile.map(|p| p.into()),
      orphan: stats.orphan,
      provided_exports: stats
        .provided_exports
        .map(|exports| exports.into_iter().map(|i| i.to_string()).collect()),
      used_exports: stats.used_exports.map(|used_exports| match used_exports {
        StatsUsedExports::Bool(b) => JsStatsUsedExports::A(b.to_string()),
        StatsUsedExports::Vec(v) => {
          JsStatsUsedExports::B(v.into_iter().map(|i| i.to_string()).collect())
        }
        StatsUsedExports::Null => JsStatsUsedExports::A("null".to_string()),
      }),
      optimization_bailout: stats.optimization_bailout.map(|bailout| bailout.to_vec()),
      modules,
      pre_order_index: stats.pre_order_index,
      post_order_index: stats.post_order_index,
      built: stats.built,
      code_generated: stats.code_generated,
      build_time_executed: stats.build_time_executed,
      cached: stats.cached,
      cacheable: stats.cacheable,
      optional: stats.optional,
      failed: stats.failed,
      errors: stats.errors,
      warnings: stats.warnings,
    })
  }
}

#[napi(object)]
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

#[napi(object)]
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
  #[napi(ts_type = "Buffer")]
  pub identifier: JsIdentifier,
  pub name: String,
  pub id: Option<String>,
}

impl FromNapiValue for JsStatsModuleIssuer {
  unsafe fn from_napi_value(
    _env: napi::sys::napi_env,
    _napi_val: napi::sys::napi_value,
  ) -> Result<Self> {
    unreachable!()
  }
}

impl From<rspack_core::StatsModuleIssuer<'_>> for JsStatsModuleIssuer {
  fn from(stats: rspack_core::StatsModuleIssuer) -> Self {
    Self {
      identifier: JsIdentifier::from(stats.identifier),
      name: stats.name.into_owned(),
      id: stats.id.map(|i| i.to_owned()),
    }
  }
}

#[napi(object, object_from_js = false)]
pub struct JsStatsModuleReason {
  #[napi(ts_type = "Buffer")]
  pub module_identifier: Option<JsIdentifier>,
  pub module_name: Option<String>,
  pub module_id: Option<String>,
  pub r#type: Option<&'static str>,
  pub user_request: Option<String>,
}

impl FromNapiValue for JsStatsModuleReason {
  unsafe fn from_napi_value(
    _env: napi::sys::napi_env,
    _napi_val: napi::sys::napi_value,
  ) -> Result<Self> {
    unreachable!()
  }
}

impl From<rspack_core::StatsModuleReason<'_>> for JsStatsModuleReason {
  fn from(stats: rspack_core::StatsModuleReason) -> Self {
    Self {
      module_identifier: stats.module_identifier.map(JsIdentifier::from),
      module_name: stats.module_name.map(|i| i.into_owned()),
      module_id: stats.module_id.map(|i| i.to_owned()),
      r#type: stats.r#type,
      user_request: stats.user_request.map(|i| i.to_owned()),
    }
  }
}

#[napi(object, object_from_js = false)]
pub struct JsOriginRecord {
  #[napi(ts_type = "Buffer")]
  pub module: JsIdentifier,
  pub module_id: String,
  #[napi(ts_type = "Buffer")]
  pub module_identifier: JsIdentifier,
  pub module_name: String,
  pub loc: String,
  pub request: String,
}

impl FromNapiValue for JsOriginRecord {
  unsafe fn from_napi_value(
    _env: napi::sys::napi_env,
    _napi_val: napi::sys::napi_value,
  ) -> Result<Self> {
    unreachable!()
  }
}

#[napi(object)]
pub struct JsStatsSize {
  pub source_type: String,
  pub size: f64,
}

#[napi(object)]
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
          module: JsIdentifier::from(origin.module.unwrap_or_default()),
          module_id: origin.module_id,
          module_identifier: JsIdentifier::from(origin.module_identifier.unwrap_or_default()),
          module_name: origin.module_name,
          loc: origin.loc,
          request: origin.request,
        })
        .collect::<Vec<_>>(),
      id_hints: stats.id_hints,
      hash: stats.hash,
    })
  }
}

#[napi(object)]
pub struct JsStatsChunkGroupAsset {
  pub name: String,
  pub size: f64,
}

impl From<rspack_core::StatsChunkGroupAsset> for JsStatsChunkGroupAsset {
  fn from(stats: rspack_core::StatsChunkGroupAsset) -> Self {
    Self {
      name: stats.name,
      size: stats.size,
    }
  }
}

#[napi(object)]
pub struct JsStatsChunkGroup {
  pub name: String,
  pub chunks: Vec<Option<String>>,
  pub assets: Vec<JsStatsChunkGroupAsset>,
  pub assets_size: f64,
  pub auxiliary_assets: Option<Vec<JsStatsChunkGroupAsset>>,
  pub auxiliary_assets_size: Option<f64>,
  pub children: Option<JsStatsChunkGroupChildren>,
}

impl From<rspack_core::StatsChunkGroup> for JsStatsChunkGroup {
  fn from(stats: rspack_core::StatsChunkGroup) -> Self {
    Self {
      name: stats.name,
      chunks: stats.chunks,
      assets: stats.assets.into_iter().map(Into::into).collect(),
      assets_size: stats.assets_size,
      auxiliary_assets: stats
        .auxiliary_assets
        .map(|assets| assets.into_iter().map(Into::into).collect()),
      auxiliary_assets_size: stats.auxiliary_assets_size,
      children: stats.children.map(|i| i.into()),
    }
  }
}

#[napi(object)]
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

#[napi(object)]
pub struct JsStatsOptimizationBailout {
  pub inner: String,
}

#[napi(object)]
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
  pub cached_modules: bool,
  pub chunk_modules: bool,
  pub chunk_relations: bool,
  pub reasons: bool,
  pub module_assets: bool,
  pub nested_modules: bool,
  pub source: bool,
  pub used_exports: bool,
  pub provided_exports: bool,
  pub ids: bool,
  pub optimization_bailout: bool,
  pub depth: bool,
}

impl From<JsStatsOptions> for ExtendedStatsOptions {
  fn from(value: JsStatsOptions) -> Self {
    Self {
      cached_modules: value.cached_modules,
      chunk_modules: value.chunk_modules,
      chunk_relations: value.chunk_relations,
      reasons: value.reasons,
      module_assets: value.module_assets,
      nested_modules: value.nested_modules,
      source: value.source,
      used_exports: value.used_exports,
      provided_exports: value.provided_exports,
      ids: value.ids,
      optimization_bailout: value.optimization_bailout,
      depth: value.depth,
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

#[napi(object)]
pub struct JsStatsGetAssets {
  pub assets: Vec<JsStatsAsset>,
  pub assets_by_chunk_name: Vec<JsStatsAssetsByChunkName>,
}

#[napi]
impl JsStats {
  #[napi]
  pub fn get_assets(&self) -> JsStatsGetAssets {
    let (assets, assets_by_chunk_name) = self.inner.get_assets();
    let assets = assets.into_iter().map(Into::into).collect();
    let assets_by_chunk_name = assets_by_chunk_name.into_iter().map(Into::into).collect();
    JsStatsGetAssets {
      assets,
      assets_by_chunk_name,
    }
  }

  #[napi]
  pub fn get_modules(&self, js_options: JsStatsOptions) -> Result<Vec<JsStatsModule>> {
    let options = ExtendedStatsOptions::from(js_options);
    self
      .inner
      .get_modules(&options, |res| {
        res.into_iter().map(JsStatsModule::try_from).collect()
      })
      .map_err(|e| napi::Error::from_reason(e.to_string()))?
  }

  #[napi]
  pub fn get_chunks(&self, js_options: JsStatsOptions) -> Result<Vec<JsStatsChunk>> {
    let options = ExtendedStatsOptions::from(js_options);
    self
      .inner
      .get_chunks(&options, |res| {
        res.into_iter().map(JsStatsChunk::try_from).collect()
      })
      .map_err(|e| napi::Error::from_reason(e.to_string()))?
  }

  #[napi]
  pub fn get_entrypoints(
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

  #[napi]
  pub fn get_named_chunk_groups(
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

  #[napi]
  pub fn get_errors(&self) -> Vec<JsStatsError> {
    self
      .inner
      .get_errors()
      .into_iter()
      .map(Into::into)
      .collect()
  }

  #[napi]
  pub fn get_warnings(&self) -> Vec<JsStatsWarning> {
    self
      .inner
      .get_warnings()
      .into_iter()
      .map(Into::into)
      .collect()
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

  #[napi]
  pub fn get_hash(&self) -> Option<String> {
    self.inner.get_hash().map(|hash| hash.to_string())
  }
}
