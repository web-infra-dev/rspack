use napi::bindgen_prelude::Buffer;
use napi::{bindgen_prelude::Result, Either};
use rspack_core::{
  LogType, Stats, StatsAsset, StatsAssetInfo, StatsAssetsByChunkName, StatsChunk, StatsChunkGroup,
  StatsChunkGroupAsset, StatsError, StatsMillisecond, StatsModule, StatsModuleIssuer,
  StatsModuleProfile, StatsModuleReason, StatsWarning,
};

use super::ToJsCompatSource;

#[napi(object)]
#[derive(Debug)]
pub struct JsStatsError {
  pub message: String,
  pub formatted: String,
  pub title: String,
}

impl From<StatsError> for JsStatsError {
  fn from(stats: StatsError) -> Self {
    Self {
      message: stats.message,
      formatted: stats.formatted,
      title: stats.title,
    }
  }
}

#[napi(object)]
pub struct JsStatsWarning {
  pub message: String,
  pub formatted: String,
}

impl From<StatsWarning> for JsStatsWarning {
  fn from(stats: StatsWarning) -> Self {
    Self {
      message: stats.message,
      formatted: stats.formatted,
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

impl From<(String, LogType)> for JsStatsLogging {
  fn from(value: (String, LogType)) -> Self {
    match value.1 {
      LogType::Error { message, trace } => Self {
        name: value.0,
        r#type: "error".to_string(),
        args: Some(vec![message]),
        trace: Some(trace),
      },
      LogType::Warn { message, trace } => Self {
        name: value.0,
        r#type: "warn".to_string(),
        args: Some(vec![message]),
        trace: Some(trace),
      },
      LogType::Info { message } => Self {
        name: value.0,
        r#type: "info".to_string(),
        args: Some(vec![message]),
        trace: None,
      },
      LogType::Log { message } => Self {
        name: value.0,
        r#type: "log".to_string(),
        args: Some(vec![message]),
        trace: None,
      },
      LogType::Debug { message } => Self {
        name: value.0,
        r#type: "debug".to_string(),
        args: Some(vec![message]),
        trace: None,
      },
      LogType::Trace { message, trace } => Self {
        name: value.0,
        r#type: "trace".to_string(),
        args: Some(vec![message]),
        trace: Some(trace),
      },
      LogType::Group { message } => Self {
        name: value.0,
        r#type: "group".to_string(),
        args: Some(vec![message]),
        trace: None,
      },
      LogType::GroupCollapsed { message } => Self {
        name: value.0,
        r#type: "groupCollapsed".to_string(),
        args: Some(vec![message]),
        trace: None,
      },
      LogType::GroupEnd => Self {
        name: value.0,
        r#type: "groupEnd".to_string(),
        args: None,
        trace: None,
      },
      LogType::Profile { label } => Self {
        name: value.0,
        r#type: "profile".to_string(),
        args: Some(vec![label.to_string()]),
        trace: None,
      },
      LogType::ProfileEnd { label } => Self {
        name: value.0,
        r#type: "profileEnd".to_string(),
        args: Some(vec![label.to_string()]),
        trace: None,
      },
      LogType::Time {
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
      LogType::Clear => Self {
        name: value.0,
        r#type: "clear".to_string(),
        args: None,
        trace: None,
      },
      LogType::Status { message } => Self {
        name: value.0,
        r#type: "status".to_string(),
        args: Some(vec![message]),
        trace: None,
      },
    }
  }
}

#[napi(object)]
pub struct JsStatsAsset {
  pub r#type: &'static str,
  pub name: String,
  pub size: f64,
  pub chunks: Vec<String>,
  pub chunk_names: Vec<String>,
  pub info: JsStatsAssetInfo,
  pub emitted: bool,
}

impl From<StatsAsset> for JsStatsAsset {
  fn from(stats: StatsAsset) -> Self {
    Self {
      r#type: stats.r#type,
      name: stats.name,
      size: stats.size,
      chunks: stats.chunks,
      chunk_names: stats.chunk_names,
      info: stats.info.into(),
      emitted: stats.emitted,
    }
  }
}

#[napi(object)]
pub struct JsStatsAssetInfo {
  pub development: bool,
  pub hot_module_replacement: bool,
}

impl From<StatsAssetInfo> for JsStatsAssetInfo {
  fn from(stats: StatsAssetInfo) -> Self {
    Self {
      development: stats.development,
      hot_module_replacement: stats.hot_module_replacement,
    }
  }
}

type JsStatsModuleSource = Either<String, Buffer>;
#[napi(object)]
pub struct JsStatsModule {
  pub r#type: &'static str,
  pub module_type: String,
  pub identifier: String,
  pub name: String,
  pub id: Option<String>,
  pub chunks: Vec<String>,
  pub size: f64,
  pub issuer: Option<String>,
  pub issuer_name: Option<String>,
  pub issuer_id: Option<String>,
  pub issuer_path: Vec<JsStatsModuleIssuer>,
  pub reasons: Option<Vec<JsStatsModuleReason>>,
  pub assets: Option<Vec<String>>,
  pub modules: Option<Vec<JsStatsModule>>,
  pub source: Option<Either<String, Buffer>>,
  pub profile: Option<JsStatsModuleProfile>,
}

impl JsStatsModule {
  pub fn from_stats_module(
    m: StatsModule,
    reasons: bool,
    module_assets: bool,
    nested_modules: bool,
    source: bool,
  ) -> Result<Self> {
    let js_source = source
      .then(|| m.source)
      .and_then(|i| i)
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
    Ok(Self {
      r#type: m.r#type,
      module_type: m.module_type.to_string(),
      identifier: m.identifier.to_string(),
      name: m.name,
      id: m.id,
      chunks: m.chunks,
      size: m.size,
      issuer: m.issuer,
      issuer_name: m.issuer_name,
      issuer_id: m.issuer_id,
      issuer_path: m.issuer_path.into_iter().map(Into::into).collect(),
      reasons: reasons.then(|| m.reasons.into_iter().map(Into::into).collect()),
      assets: module_assets.then(|| m.assets),
      modules: nested_modules
        .then(|| {
          m.modules
            .into_iter()
            .map(|m| {
              JsStatsModule::from_stats_module(m, reasons, module_assets, nested_modules, source)
            })
            .collect()
        })
        .transpose()?
        .filter(|i: &Vec<JsStatsModule>| !i.is_empty()),
      source: js_source,
      profile: m.profile.map(Into::into),
    })
  }
}

#[napi(object)]
pub struct JsStatsModuleProfile {
  pub factory: JsStatsMillisecond,
  pub integration: JsStatsMillisecond,
  pub building: JsStatsMillisecond,
}

impl From<StatsModuleProfile> for JsStatsModuleProfile {
  fn from(value: StatsModuleProfile) -> Self {
    Self {
      factory: value.factory.into(),
      integration: value.integration.into(),
      building: value.building.into(),
    }
  }
}

#[napi(object)]
pub struct JsStatsMillisecond {
  pub secs: u32,
  pub subsec_millis: u32,
}

impl From<StatsMillisecond> for JsStatsMillisecond {
  fn from(value: StatsMillisecond) -> Self {
    Self {
      secs: value.secs as u32,
      subsec_millis: value.subsec_millis,
    }
  }
}

#[napi(object)]
pub struct JsStatsModuleIssuer {
  pub identifier: String,
  pub name: String,
  pub id: Option<String>,
}

impl From<StatsModuleIssuer> for JsStatsModuleIssuer {
  fn from(stats: StatsModuleIssuer) -> Self {
    Self {
      identifier: stats.identifier,
      name: stats.name,
      id: stats.id,
    }
  }
}

#[napi(object)]
pub struct JsStatsModuleReason {
  pub module_identifier: Option<String>,
  pub module_name: Option<String>,
  pub module_id: Option<String>,
  pub r#type: Option<String>,
  pub user_request: Option<String>,
}

impl From<StatsModuleReason> for JsStatsModuleReason {
  fn from(stats: StatsModuleReason) -> Self {
    Self {
      module_identifier: stats.module_identifier,
      module_name: stats.module_name,
      module_id: stats.module_id,
      r#type: stats.r#type,
      user_request: stats.user_request,
    }
  }
}

#[napi(object)]
pub struct JsStatsChunk {
  pub r#type: &'static str,
  pub files: Vec<String>,
  pub auxiliary_files: Vec<String>,
  pub id: String,
  pub entry: bool,
  pub initial: bool,
  pub names: Vec<String>,
  pub size: f64,
  pub modules: Option<Vec<JsStatsModule>>,
  pub parents: Option<Vec<String>>,
  pub children: Option<Vec<String>>,
  pub siblings: Option<Vec<String>>,
}

impl JsStatsChunk {
  pub fn from_stats_chunk(
    c: StatsChunk,
    chunk_modules: bool,
    chunks_relations: bool,
    reasons: bool,
    module_assets: bool,
    nested_modules: bool,
    source: bool,
  ) -> Result<Self> {
    Ok(Self {
      r#type: c.r#type,
      files: c.files,
      auxiliary_files: c.auxiliary_files,
      id: c.id,
      entry: c.entry,
      initial: c.initial,
      names: c.names,
      size: c.size,
      modules: chunk_modules
        .then(|| {
          c.modules
            .into_iter()
            .map(|m| {
              JsStatsModule::from_stats_module(m, reasons, module_assets, nested_modules, source)
            })
            .collect::<Result<_>>()
        })
        .transpose()?,
      parents: chunks_relations.then_some(c.parents),
      children: chunks_relations.then_some(c.children),
      siblings: chunks_relations.then_some(c.siblings),
    })
  }
}

#[napi(object)]
pub struct JsStatsChunkGroupAsset {
  pub name: String,
  pub size: f64,
}

impl From<StatsChunkGroupAsset> for JsStatsChunkGroupAsset {
  fn from(stats: StatsChunkGroupAsset) -> Self {
    Self {
      name: stats.name,
      size: stats.size,
    }
  }
}

#[napi(object)]
pub struct JsStatsChunkGroup {
  pub name: String,
  pub assets: Vec<JsStatsChunkGroupAsset>,
  pub chunks: Vec<String>,
  pub assets_size: f64,
}

impl From<StatsChunkGroup> for JsStatsChunkGroup {
  fn from(stats: StatsChunkGroup) -> Self {
    Self {
      name: stats.name,
      assets: stats.assets.into_iter().map(Into::into).collect(),
      chunks: stats.chunks,
      assets_size: stats.assets_size,
    }
  }
}

#[napi(object)]
pub struct JsStatsAssetsByChunkName {
  pub name: String,
  pub files: Vec<String>,
}

impl From<StatsAssetsByChunkName> for JsStatsAssetsByChunkName {
  fn from(stats: StatsAssetsByChunkName) -> Self {
    Self {
      name: stats.name,
      files: stats.files,
    }
  }
}

#[napi(object)]
pub struct JsStatsGetAssets {
  pub assets: Vec<JsStatsAsset>,
  pub assets_by_chunk_name: Vec<JsStatsAssetsByChunkName>,
}

#[napi]
pub struct JsStats {
  assets: Vec<StatsAsset>,
  assets_by_chunk_name: Vec<StatsAssetsByChunkName>,
  modules: Vec<StatsModule>,
  chunks: Vec<StatsChunk>,
  entrypoints: Vec<StatsChunkGroup>,
  named_chunk_groups: Vec<StatsChunkGroup>,
  errors: Vec<StatsError>,
  warnings: Vec<StatsWarning>,
  logging: Vec<(String, LogType)>,
  hash: Option<String>,
}

impl TryFrom<Stats<'_>> for JsStats {
  type Error = napi::Error;
  fn try_from(stats: Stats) -> Result<Self> {
    let (assets, assets_by_chunk_name) = stats.get_assets();
    Ok(Self {
      assets,
      assets_by_chunk_name,
      modules: stats
        .get_modules()
        .map_err(|e| napi::Error::from_reason(e.to_string()))?,
      chunks: stats
        .get_chunks()
        .map_err(|e| napi::Error::from_reason(e.to_string()))?,
      entrypoints: stats.get_entrypoints(),
      named_chunk_groups: stats.get_named_chunk_groups(),
      errors: stats.get_errors(),
      warnings: stats.get_warnings(),
      logging: stats.get_logging(),
      hash: stats.get_hash().map(|h| h.to_owned()),
    })
  }
}

#[napi]
impl JsStats {
  #[napi]
  pub fn get_assets(&self) -> JsStatsGetAssets {
    let assets = self.assets.clone().into_iter().map(Into::into).collect();
    let assets_by_chunk_name = self
      .assets_by_chunk_name
      .clone()
      .into_iter()
      .map(Into::into)
      .collect();
    JsStatsGetAssets {
      assets,
      assets_by_chunk_name,
    }
  }

  #[napi]
  pub fn get_modules(
    &self,
    reasons: bool,
    module_assets: bool,
    nested_modules: bool,
    source: bool,
  ) -> Result<Vec<JsStatsModule>> {
    self
      .modules
      .clone()
      .into_iter()
      .map(|m| JsStatsModule::from_stats_module(m, reasons, module_assets, nested_modules, source))
      .collect()
  }

  #[napi]
  pub fn get_chunks(
    &self,
    chunk_modules: bool,
    chunks_relations: bool,
    reasons: bool,
    module_assets: bool,
    nested_modules: bool,
    source: bool,
  ) -> Result<Vec<JsStatsChunk>> {
    self
      .chunks
      .clone()
      .into_iter()
      .map(|c| {
        JsStatsChunk::from_stats_chunk(
          c,
          chunk_modules,
          chunks_relations,
          reasons,
          module_assets,
          nested_modules,
          source,
        )
      })
      .collect()
  }

  #[napi]
  pub fn get_entrypoints(&self) -> Vec<JsStatsChunkGroup> {
    self
      .entrypoints
      .clone()
      .into_iter()
      .map(Into::into)
      .collect()
  }

  #[napi]
  pub fn get_named_chunk_groups(&self) -> Vec<JsStatsChunkGroup> {
    self
      .named_chunk_groups
      .clone()
      .into_iter()
      .map(Into::into)
      .collect()
  }

  #[napi]
  pub fn get_errors(&self) -> Vec<JsStatsError> {
    self.errors.clone().into_iter().map(Into::into).collect()
  }

  #[napi]
  pub fn get_warnings(&self) -> Vec<JsStatsWarning> {
    self.warnings.clone().into_iter().map(Into::into).collect()
  }

  #[napi]
  pub fn get_logging(&self, accepted_types: u32) -> Vec<JsStatsLogging> {
    self
      .logging
      .clone()
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
    self.hash.clone()
  }
}
