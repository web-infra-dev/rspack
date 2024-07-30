use std::collections::HashMap;

use napi_derive::napi;
use rspack_core::{Stats, StatsChunk, StatsModule, StatsUsedExports};
use rspack_napi::napi::bindgen_prelude::Buffer;
use rspack_napi::napi::{
  bindgen_prelude::{Result, SharedReference},
  Either,
};

use super::{JsCompilation, ToJsCompatSource};

#[napi(object)]
#[derive(Debug)]
pub struct JsStatsError {
  pub message: String,
  pub chunk_name: Option<String>,
  pub chunk_entry: Option<bool>,
  pub chunk_initial: Option<bool>,
  pub loc: Option<String>,
  pub file: Option<String>,
  pub module_identifier: Option<&'static str>,
  pub module_name: Option<String>,
  pub module_id: Option<String>,
  pub chunk_id: Option<String>,
  pub details: Option<String>,
  pub stack: Option<String>,
  pub module_trace: Vec<JsStatsModuleTrace>,
}

impl From<rspack_core::StatsError<'_>> for JsStatsError {
  fn from(stats: rspack_core::StatsError) -> Self {
    Self {
      message: stats.message,
      module_identifier: stats.module_identifier,
      module_name: stats.module_name.map(|i| i.into_owned()),
      module_id: stats.module_id.map(|i| i.to_owned()),
      loc: stats.loc,
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
pub struct JsStatsWarning {
  pub message: String,
  pub chunk_name: Option<String>,
  pub chunk_entry: Option<bool>,
  pub chunk_initial: Option<bool>,
  pub file: Option<String>,
  pub module_identifier: Option<&'static str>,
  pub module_name: Option<String>,
  pub module_id: Option<String>,
  pub chunk_id: Option<String>,
  pub details: Option<String>,
  pub stack: Option<String>,
  pub module_trace: Vec<JsStatsModuleTrace>,
}

impl From<rspack_core::StatsWarning<'_>> for JsStatsWarning {
  fn from(stats: rspack_core::StatsWarning) -> Self {
    Self {
      message: stats.message,
      module_identifier: stats.module_identifier,
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

#[napi(object)]
#[derive(Debug)]
pub struct JsStatsModuleTraceModule {
  pub identifier: String,
  pub name: Option<String>,
  pub id: Option<String>,
}

impl From<rspack_core::StatsErrorModuleTraceModule> for JsStatsModuleTraceModule {
  fn from(stats: rspack_core::StatsErrorModuleTraceModule) -> Self {
    Self {
      identifier: stats.identifier,
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

#[napi]
#[rspack_napi_macros::getters]
#[derive(Clone)]
pub struct JsStatsModule {
  r#type: &'static str,
  module_type: &'static str,
  identifier: &'static str,
  layer: Option<String>,
  name: String,
  id: Option<String>,
  chunks: Vec<Option<String>>,
  size: f64,
  depth: Option<u32>,
  dependent: Option<bool>,
  issuer: Option<String>,
  issuer_name: Option<String>,
  issuer_id: Option<String>,
  issuer_path: Vec<JsStatsModuleIssuer>,
  name_for_condition: Option<String>,
  assets: Option<Vec<String>>,
  source: Option<Either<String, Buffer>>,
  orphan: bool,
  provided_exports: Option<Vec<String>>,
  used_exports: Option<Either<String, Vec<String>>>,
  optimization_bailout: Option<Vec<String>>,
  pre_order_index: Option<u32>,
  post_order_index: Option<u32>,
  built: bool,
  code_generated: bool,
  build_time_executed: bool,
  cached: bool,
  cacheable: bool,
  optional: bool,
  failed: bool,
  errors: u32,
  warnings: u32,
  sizes: Vec<JsStatsSize>,
  profile: Option<JsStatsModuleProfile>,
  reasons: Option<Vec<JsStatsModuleReason>>,
  modules: Option<Vec<JsStatsModule>>,
}

impl TryFrom<StatsModule<'_>> for JsStatsModule {
  type Error = napi::Error;

  fn try_from(stats_module: StatsModule) -> std::result::Result<Self, Self::Error> {
    let source = stats_module
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

    let mut sizes = stats_module
      .sizes
      .into_iter()
      .map(|s| JsStatsSize {
        source_type: s.source_type.to_string(),
        size: s.size,
      })
      .collect::<Vec<_>>();
    sizes.sort_by(|a, b| a.source_type.cmp(&b.source_type));
    let modules: Option<Vec<JsStatsModule>> = stats_module
      .modules
      .map(|modules| -> Result<_> {
        modules
          .into_iter()
          .map(JsStatsModule::try_from)
          .collect::<Result<_>>()
      })
      .transpose()
      .map_err(|e| napi::Error::from_reason(e.to_string()))?;

    let reasons = match stats_module.reasons {
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
      r#type: stats_module.r#type,
      name: stats_module.name.into_owned(),
      size: stats_module.size,
      sizes,
      depth: stats_module.depth.map(|d| d as u32),
      chunks: stats_module.chunks,
      module_type: stats_module.module_type.as_str(),
      layer: stats_module.layer.map(|i| i.into_owned()),
      identifier: stats_module.identifier.as_str(),
      id: stats_module.id.map(|i| i.to_owned()),
      dependent: stats_module.dependent,
      issuer: stats_module.issuer.map(|i| i.to_owned()),
      issuer_name: stats_module.issuer_name.map(|i| i.into_owned()),
      issuer_id: stats_module.issuer_id.map(|i| i.to_owned()),
      name_for_condition: stats_module.name_for_condition,
      issuer_path: stats_module
        .issuer_path
        .into_iter()
        .map(Into::into)
        .collect(),
      reasons,
      assets: stats_module.assets,
      source,
      profile: stats_module.profile.map(|p| p.into()),
      orphan: stats_module.orphan,
      provided_exports: stats_module
        .provided_exports
        .map(|exports| exports.into_iter().map(|i| i.to_string()).collect()),
      used_exports: stats_module
        .used_exports
        .map(|used_exports| match used_exports {
          StatsUsedExports::Bool(b) => JsStatsUsedExports::A(b.to_string()),
          StatsUsedExports::Vec(v) => {
            JsStatsUsedExports::B(v.into_iter().map(|i| i.to_string()).collect())
          }
          StatsUsedExports::Null => JsStatsUsedExports::A("null".to_string()),
        }),
      optimization_bailout: Some(stats_module.optimization_bailout.to_vec()),
      modules,
      pre_order_index: stats_module.pre_order_index,
      post_order_index: stats_module.post_order_index,
      built: stats_module.built,
      code_generated: stats_module.code_generated,
      build_time_executed: stats_module.build_time_executed,
      cached: stats_module.cached,
      cacheable: stats_module.cacheable,
      optional: stats_module.optional,
      failed: stats_module.failed,
      errors: stats_module.errors,
      warnings: stats_module.warnings,
    })
  }
}

#[derive(Clone)]
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

#[derive(Clone)]
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

#[derive(Clone)]
#[napi(object)]
pub struct JsStatsModuleIssuer {
  pub identifier: &'static str,
  pub name: String,
  pub id: Option<String>,
}

impl From<rspack_core::StatsModuleIssuer<'_>> for JsStatsModuleIssuer {
  fn from(stats: rspack_core::StatsModuleIssuer) -> Self {
    Self {
      identifier: stats.identifier,
      name: stats.name.into_owned(),
      id: stats.id.map(|i| i.to_owned()),
    }
  }
}

#[napi]
#[rspack_napi_macros::getters]
#[derive(Clone)]
pub struct JsStatsModuleReason {
  module_identifier: Option<&'static str>,
  module_name: Option<String>,
  module_id: Option<String>,
  r#type: Option<&'static str>,
  user_request: Option<String>,
}

impl From<rspack_core::StatsModuleReason<'_>> for JsStatsModuleReason {
  fn from(stats: rspack_core::StatsModuleReason) -> Self {
    Self {
      module_identifier: stats.module_identifier,
      module_name: stats.module_name.map(|i| i.into_owned()),
      module_id: stats.module_id.map(|i| i.to_owned()),
      r#type: stats.r#type,
      user_request: stats.user_request.map(|i| i.to_owned()),
    }
  }
}

#[derive(Clone)]
#[napi(object)]
pub struct JsOriginRecord {
  pub module: String,
  pub module_id: String,
  pub module_identifier: String,
  pub module_name: String,
  pub loc: String,
  pub request: String,
}

#[derive(Clone)]
#[napi]
pub struct JsStatsSize {
  pub source_type: String,
  pub size: f64,
}

#[napi]
#[rspack_napi_macros::getters]
pub struct JsStatsChunk {
  r#type: String,
  files: Vec<String>,
  auxiliary_files: Vec<String>,
  id: Option<String>,
  id_hints: Vec<String>,
  hash: Option<String>,
  entry: bool,
  initial: bool,
  names: Vec<String>,
  size: f64,
  parents: Option<Vec<String>>,
  children: Option<Vec<String>>,
  siblings: Option<Vec<String>>,
  children_by_order: HashMap<String, Vec<String>>,
  runtime: Vec<String>,
  reason: Option<String>,
  rendered: bool,
  sizes: Vec<JsStatsSize>,
  origins: Vec<JsOriginRecord>,
  modules: Option<Vec<JsStatsModule>>,
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
          module: origin.module,
          module_id: origin.module_id,
          module_identifier: origin.module_identifier,
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

  #[allow(clippy::too_many_arguments)]
  #[napi]
  pub fn get_modules(
    &self,
    reasons: bool,
    module_assets: bool,
    nested_modules: bool,
    source: bool,
    used_exports: bool,
    provided_exports: bool,
  ) -> Result<Vec<JsStatsModule>> {
    self
      .inner
      .get_modules(
        reasons,
        module_assets,
        nested_modules,
        source,
        used_exports,
        provided_exports,
        |res| res.into_iter().map(JsStatsModule::try_from).collect(),
      )
      .map_err(|e| napi::Error::from_reason(e.to_string()))?
  }

  #[allow(clippy::too_many_arguments)]
  #[napi]
  pub fn get_chunks(
    &self,
    chunk_modules: bool,
    chunks_relations: bool,
    reasons: bool,
    module_assets: bool,
    nested_modules: bool,
    source: bool,
    used_exports: bool,
    provided_exports: bool,
  ) -> Result<Vec<JsStatsChunk>> {
    self
      .inner
      .get_chunks(
        chunk_modules,
        chunks_relations,
        reasons,
        module_assets,
        nested_modules,
        source,
        used_exports,
        provided_exports,
        |res| res.into_iter().map(JsStatsChunk::try_from).collect(),
      )
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
