use std::collections::HashMap;

use napi::bindgen_prelude::*;

#[napi]
pub enum DiffStatKind {
  Changed,
  Deleted,
  Added,
}

impl From<u8> for DiffStatKind {
  fn from(n: u8) -> Self {
    match n {
      0 => Self::Changed,
      1 => Self::Deleted,
      2 => Self::Added,
      _ => unreachable!(),
    }
  }
}

// TODO: remove it after hash
#[napi(object)]
pub struct DiffStat {
  pub content: String,
  pub kind: DiffStatKind,
}

#[napi(object)]
pub struct RebuildResult {
  pub diff: HashMap<String, DiffStat>,
  pub stats: StatsCompilation,
}

#[napi(object)]
pub struct StatsError {
  pub message: String,
  pub formatted: String,
}

impl From<rspack_core::StatsError> for StatsError {
  fn from(stats: rspack_core::StatsError) -> Self {
    Self {
      message: stats.message,
      formatted: stats.formatted,
    }
  }
}

#[napi(object)]
pub struct StatsWarning {
  pub message: String,
  pub formatted: String,
}

impl From<rspack_core::StatsWarning> for StatsWarning {
  fn from(stats: rspack_core::StatsWarning) -> Self {
    Self {
      message: stats.message,
      formatted: stats.formatted,
    }
  }
}

#[napi(object)]
pub struct StatsAsset {
  pub r#type: &'static str,
  pub name: String,
  pub size: f64,
  pub chunks: Vec<String>,
  pub chunk_names: Vec<String>,
  pub info: StatsAssetInfo,
}

impl From<rspack_core::StatsAsset> for StatsAsset {
  fn from(stats: rspack_core::StatsAsset) -> Self {
    Self {
      r#type: stats.r#type,
      name: stats.name,
      size: stats.size,
      chunks: stats.chunks,
      chunk_names: stats.chunk_names,
      info: stats.info.into(),
    }
  }
}

#[napi(object)]
pub struct StatsAssetInfo {
  pub development: bool,
}

impl From<rspack_core::StatsAssetInfo> for StatsAssetInfo {
  fn from(stats: rspack_core::StatsAssetInfo) -> Self {
    Self {
      development: stats.development,
    }
  }
}

#[napi(object)]
pub struct StatsModule {
  pub r#type: &'static str,
  pub module_type: String,
  pub identifier: String,
  pub name: String,
  pub id: String,
  pub chunks: Vec<String>,
  pub size: f64,
}

impl From<rspack_core::StatsModule> for StatsModule {
  fn from(stats: rspack_core::StatsModule) -> Self {
    Self {
      r#type: stats.r#type,
      name: stats.name,
      size: stats.size,
      chunks: stats.chunks,
      module_type: stats.module_type.to_string(),
      identifier: stats.identifier,
      id: stats.id,
    }
  }
}

#[napi(object)]
pub struct StatsChunk {
  pub r#type: &'static str,
  pub files: Vec<String>,
  pub id: String,
  pub entry: bool,
  pub initial: bool,
  pub names: Vec<String>,
  pub size: f64,
}

impl From<rspack_core::StatsChunk> for StatsChunk {
  fn from(stats: rspack_core::StatsChunk) -> Self {
    Self {
      r#type: stats.r#type,
      files: stats.files,
      id: stats.id,
      entry: stats.entry,
      initial: stats.initial,
      names: stats.names,
      size: stats.size,
    }
  }
}

#[napi(object)]
pub struct StatsCompilation {
  pub assets: Vec<StatsAsset>,
  pub modules: Vec<StatsModule>,
  pub chunks: Vec<StatsChunk>,
  pub errors: Vec<StatsError>,
  pub errors_count: u32,
  pub warnings: Vec<StatsWarning>,
  pub warnings_count: u32,
}

impl From<rspack_core::StatsCompilation> for StatsCompilation {
  fn from(stats: rspack_core::StatsCompilation) -> Self {
    Self {
      assets: stats.assets.into_iter().map(Into::into).collect(),
      modules: stats.modules.into_iter().map(Into::into).collect(),
      chunks: stats.chunks.into_iter().map(Into::into).collect(),
      errors: stats.errors.into_iter().map(Into::into).collect(),
      errors_count: stats.errors_count as u32,
      warnings: stats.warnings.into_iter().map(Into::into).collect(),
      warnings_count: stats.warnings_count as u32,
    }
  }
}
