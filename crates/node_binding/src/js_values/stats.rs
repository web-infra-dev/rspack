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
#[napi]
pub struct DiffStat {
  pub content: String,
  pub kind: DiffStatKind,
}

#[napi(object)]
pub struct StatsError {
  pub message: String,
}

impl From<rspack_core::StatsError> for StatsError {
  fn from(stats: rspack_core::StatsError) -> Self {
    Self {
      message: stats.message,
    }
  }
}

#[napi(object)]
pub struct StatsAsset {
  pub r#type: &'static str,
  pub name: String,
  pub size: f64,
  pub chunks: Vec<String>,
}

impl From<rspack_core::StatsAsset> for StatsAsset {
  fn from(stats: rspack_core::StatsAsset) -> Self {
    Self {
      r#type: stats.r#type,
      name: stats.name,
      size: stats.size,
      chunks: stats.chunks,
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
}

impl From<rspack_core::StatsChunk> for StatsChunk {
  fn from(stats: rspack_core::StatsChunk) -> Self {
    Self {
      r#type: stats.r#type,
      files: Vec::from_iter(stats.files.into_iter()),
      id: stats.id,
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
}

impl From<rspack_core::StatsCompilation> for StatsCompilation {
  fn from(stats: rspack_core::StatsCompilation) -> Self {
    Self {
      assets: stats.assets.into_iter().map(Into::into).collect(),
      modules: stats.modules.into_iter().map(Into::into).collect(),
      chunks: stats.chunks.into_iter().map(Into::into).collect(),
      errors: stats.errors.into_iter().map(Into::into).collect(),
      errors_count: stats.errors_count as u32,
    }
  }
}
