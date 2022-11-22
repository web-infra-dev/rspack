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
}

impl From<rspack_core::StatsModule> for JsStatsModule {
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

#[napi(object)]
pub struct JsStatsCompilation {
  pub assets: Vec<JsStatsAsset>,
  pub modules: Vec<JsStatsModule>,
  pub chunks: Vec<JsStatsChunk>,
  pub entrypoints: Vec<JsStatsEntrypoint>,
  pub errors: Vec<JsStatsError>,
  pub errors_count: u32,
  pub warnings: Vec<JsStatsWarning>,
  pub warnings_count: u32,
}

impl From<rspack_core::StatsCompilation> for JsStatsCompilation {
  fn from(stats: rspack_core::StatsCompilation) -> Self {
    Self {
      assets: stats.assets.into_iter().map(Into::into).collect(),
      modules: stats.modules.into_iter().map(Into::into).collect(),
      chunks: stats.chunks.into_iter().map(Into::into).collect(),
      entrypoints: stats.entrypoints.into_iter().map(Into::into).collect(),
      errors: stats.errors.into_iter().map(Into::into).collect(),
      errors_count: stats.errors_count as u32,
      warnings: stats.warnings.into_iter().map(Into::into).collect(),
      warnings_count: stats.warnings_count as u32,
    }
  }
}
