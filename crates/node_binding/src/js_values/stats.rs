use std::collections::HashMap;

use napi::bindgen_prelude::SharedReference;
use rspack_core::Stats;

use super::JsCompilation;

#[napi(object)]
#[derive(Debug)]
pub struct JsStatsError {
  pub message: String,
  pub formatted: String,
}

impl From<rspack_core::StatsError> for JsStatsError {
  fn from(stats: rspack_core::StatsError) -> Self {
    Self {
      message: stats.message,
      formatted: stats.formatted,
    }
  }
}

#[napi(object)]
pub struct JsStatsWarning {
  pub message: String,
  pub formatted: String,
}

impl From<rspack_core::StatsWarning> for JsStatsWarning {
  fn from(stats: rspack_core::StatsWarning) -> Self {
    Self {
      message: stats.message,
      formatted: stats.formatted,
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
    }
  }
}

#[napi(object)]
pub struct JsStatsAssetInfo {
  pub development: bool,
  pub hot_module_replacement: bool,
}

impl From<rspack_core::StatsAssetInfo> for JsStatsAssetInfo {
  fn from(stats: rspack_core::StatsAssetInfo) -> Self {
    Self {
      development: stats.development,
      hot_module_replacement: stats.hot_module_replacement,
    }
  }
}

#[napi(object)]
pub struct JsStatsModule {
  pub r#type: &'static str,
  pub module_type: String,
  pub identifier: String,
  pub name: String,
  pub id: String,
  pub chunks: Vec<String>,
  pub size: f64,
  pub issuer: Option<String>,
  pub issuer_name: Option<String>,
  pub issuer_id: Option<String>,
  pub issuer_path: Vec<JsStatsModuleIssuer>,
  pub reasons: Option<Vec<JsStatsModuleReason>>,
}

impl From<rspack_core::StatsModule> for JsStatsModule {
  fn from(stats: rspack_core::StatsModule) -> Self {
    Self {
      r#type: stats.r#type,
      name: stats.name,
      size: stats.size,
      chunks: stats.chunks,
      module_type: stats.module_type.to_string(),
      identifier: stats.identifier.to_string(),
      id: stats.id,
      issuer: stats.issuer,
      issuer_name: stats.issuer_name,
      issuer_id: stats.issuer_id,
      issuer_path: stats.issuer_path.into_iter().map(Into::into).collect(),
      reasons: stats
        .reasons
        .map(|i| i.into_iter().map(Into::into).collect()),
    }
  }
}

#[napi(object)]
pub struct JsStatsModuleIssuer {
  pub identifier: String,
  pub name: String,
  pub id: String,
}

impl From<rspack_core::StatsModuleIssuer> for JsStatsModuleIssuer {
  fn from(stats: rspack_core::StatsModuleIssuer) -> Self {
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
}

impl From<rspack_core::StatsModuleReason> for JsStatsModuleReason {
  fn from(stats: rspack_core::StatsModuleReason) -> Self {
    Self {
      module_identifier: stats.module_identifier,
      module_name: stats.module_name,
      module_id: stats.module_id,
    }
  }
}

#[napi(object)]
pub struct JsStatsChunk {
  pub r#type: &'static str,
  pub files: Vec<String>,
  pub id: String,
  pub entry: bool,
  pub initial: bool,
  pub names: Vec<String>,
  pub size: f64,
}

impl From<rspack_core::StatsChunk> for JsStatsChunk {
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
pub struct JsStatsEntrypointAsset {
  pub name: String,
  pub size: f64,
}

impl From<rspack_core::StatsEntrypointAsset> for JsStatsEntrypointAsset {
  fn from(stats: rspack_core::StatsEntrypointAsset) -> Self {
    Self {
      name: stats.name,
      size: stats.size,
    }
  }
}

#[napi(object)]
pub struct JsStatsEntrypoint {
  pub name: String,
  pub assets: Vec<JsStatsEntrypointAsset>,
  pub chunks: Vec<String>,
  pub assets_size: f64,
}

impl From<rspack_core::StatsEntrypoint> for JsStatsEntrypoint {
  fn from(stats: rspack_core::StatsEntrypoint) -> Self {
    Self {
      name: stats.name,
      assets: stats.assets.into_iter().map(Into::into).collect(),
      chunks: stats.chunks,
      assets_size: stats.assets_size,
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
  pub assets_by_chunk_name: HashMap<String, Vec<String>>,
}

#[napi]
impl JsStats {
  #[napi]
  pub fn get_assets(&self) -> JsStatsGetAssets {
    let (assets, assets_by_chunk_name) = self.inner.get_assets();
    let assets = assets.into_iter().map(Into::into).collect();
    let assets_by_chunk_name = HashMap::from_iter(assets_by_chunk_name);
    JsStatsGetAssets {
      assets,
      assets_by_chunk_name,
    }
  }

  #[napi]
  pub fn get_modules(&self, show_reasons: bool) -> Vec<JsStatsModule> {
    self
      .inner
      .get_modules(show_reasons)
      .expect("Failed to get modules")
      .into_iter()
      .map(Into::into)
      .collect()
  }

  #[napi]
  pub fn get_chunks(&self) -> Vec<JsStatsChunk> {
    self
      .inner
      .get_chunks()
      .into_iter()
      .map(Into::into)
      .collect()
  }

  #[napi]
  pub fn get_entrypoints(&self) -> HashMap<String, JsStatsEntrypoint> {
    HashMap::from_iter(
      self
        .inner
        .get_entrypoints()
        .into_iter()
        .map(|(name, entrypoint)| (name, entrypoint.into())),
    )
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
  pub fn get_hash(&self) -> String {
    self.inner.get_hash()
  }
}
