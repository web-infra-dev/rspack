use std::collections::HashSet;

use napi::Either;
use napi_derive::napi;
use rspack_plugin_rsdoctor::{
  RsdoctorAsset, RsdoctorAssetPatch, RsdoctorChunk, RsdoctorChunkAssets, RsdoctorChunkGraph,
  RsdoctorChunkModules, RsdoctorDependency, RsdoctorEntrypoint, RsdoctorEntrypointAssets,
  RsdoctorExportInfo, RsdoctorJsonAssetSize, RsdoctorJsonAssetSizesPatch, RsdoctorModule,
  RsdoctorModuleGraph, RsdoctorModuleGraphModule, RsdoctorModuleId, RsdoctorModuleIdsPatch,
  RsdoctorModuleOriginalSource, RsdoctorModuleSourcesPatch, RsdoctorPluginChunkGraphFeature,
  RsdoctorPluginModuleGraphFeature, RsdoctorPluginOptions, RsdoctorPluginSourceMapFeature,
  RsdoctorSideEffect, RsdoctorSourcePosition, RsdoctorSourceRange, RsdoctorStatement,
  RsdoctorVariable,
};

#[napi(object)]
pub struct JsRsdoctorModule {
  pub ukey: i32,
  pub identifier: String,
  pub path: String,
  pub is_entry: bool,
  #[napi(ts_type = "'normal' | 'concatenated'")]
  pub kind: String,
  pub layer: Option<String>,
  pub dependencies: Vec<i32>,
  pub imported: Vec<i32>,
  pub modules: Vec<i32>,
  pub belong_modules: Vec<i32>,
  pub chunks: Vec<i32>,
  pub issuer_path: Vec<i32>,
  pub bailout_reason: Vec<String>,
  pub size: Option<i32>,
}

impl From<RsdoctorModule> for JsRsdoctorModule {
  fn from(value: RsdoctorModule) -> Self {
    JsRsdoctorModule {
      ukey: value.ukey,
      identifier: value.identifier.to_string(),
      path: value.path,
      is_entry: value.is_entry,
      kind: value.kind.into(),
      layer: value.layer,
      dependencies: value.dependencies.into_iter().collect::<Vec<_>>(),
      imported: value.imported.into_iter().collect::<Vec<_>>(),
      modules: value.modules.into_iter().collect::<Vec<_>>(),
      chunks: value.chunks.into_iter().collect::<Vec<_>>(),
      belong_modules: value.belong_modules.into_iter().collect::<Vec<_>>(),
      issuer_path: value
        .issuer_path
        .unwrap_or_default()
        .into_iter()
        .filter_map(|i| i.ukey)
        .collect::<Vec<_>>(),
      bailout_reason: value.bailout_reason.into_iter().collect::<Vec<_>>(),
      size: value.size,
    }
  }
}

#[napi(object)]
pub struct JsRsdoctorDependency {
  pub ukey: i32,
  pub kind: String,
  pub request: String,
  pub module: i32,
  pub dependency: i32,
}

impl From<RsdoctorDependency> for JsRsdoctorDependency {
  fn from(value: RsdoctorDependency) -> Self {
    JsRsdoctorDependency {
      ukey: value.ukey,
      kind: value.kind.to_string(),
      request: value.request,
      module: value.module,
      dependency: value.dependency,
    }
  }
}

#[napi(object)]
pub struct JsRsdoctorChunk {
  pub ukey: i32,
  pub name: String,
  pub initial: bool,
  pub entry: bool,
  pub dependencies: Vec<i32>,
  pub imported: Vec<i32>,
}

impl From<RsdoctorChunk> for JsRsdoctorChunk {
  fn from(value: RsdoctorChunk) -> Self {
    JsRsdoctorChunk {
      ukey: value.ukey,
      name: value.name,
      initial: value.initial,
      entry: value.entry,
      dependencies: value.dependencies.into_iter().collect::<Vec<_>>(),
      imported: value.imported.into_iter().collect::<Vec<_>>(),
    }
  }
}

#[napi(object)]
pub struct JsRsdoctorEntrypoint {
  pub ukey: i32,
  pub name: String,
  pub chunks: Vec<i32>,
}

impl From<RsdoctorEntrypoint> for JsRsdoctorEntrypoint {
  fn from(value: RsdoctorEntrypoint) -> Self {
    JsRsdoctorEntrypoint {
      ukey: value.ukey,
      name: value.name,
      chunks: value.chunks.into_iter().collect::<Vec<_>>(),
    }
  }
}

#[napi(object)]
pub struct JsRsdoctorAsset {
  pub ukey: i32,
  pub path: String,
  pub chunks: Vec<i32>,
  pub size: i32,
}

impl From<RsdoctorAsset> for JsRsdoctorAsset {
  fn from(value: RsdoctorAsset) -> Self {
    JsRsdoctorAsset {
      ukey: value.ukey,
      path: value.path,
      chunks: value.chunks.into_iter().collect::<Vec<_>>(),
      size: value.size,
    }
  }
}

#[napi(object)]
pub struct JsRsdoctorModuleGraphModule {
  pub ukey: i32,
  pub module: i32,
  pub exports: Vec<i32>,
  pub side_effects: Vec<i32>,
  pub variables: Vec<i32>,
  pub dynamic: bool,
}

impl From<RsdoctorModuleGraphModule> for JsRsdoctorModuleGraphModule {
  fn from(value: RsdoctorModuleGraphModule) -> Self {
    JsRsdoctorModuleGraphModule {
      ukey: value.ukey,
      module: value.module,
      exports: value.exports.into_iter().collect::<Vec<_>>(),
      side_effects: value.side_effects.into_iter().collect::<Vec<_>>(),
      variables: value.variables.into_iter().collect::<Vec<_>>(),
      dynamic: value.dynamic,
    }
  }
}

#[napi(object)]
pub struct JsRsdoctorSideEffect {
  pub ukey: i32,
  pub name: String,
  pub origin_name: Option<String>,
  pub module: i32,
  pub identifier: JsRsdoctorStatement,
  pub is_name_space: bool,
  pub from_dependency: Option<i32>,
  pub exports: Vec<i32>,
  pub variable: Option<i32>,
}

impl From<RsdoctorSideEffect> for JsRsdoctorSideEffect {
  fn from(value: RsdoctorSideEffect) -> Self {
    JsRsdoctorSideEffect {
      ukey: value.ukey,
      name: value.name,
      origin_name: value.origin_name,
      module: value.module,
      identifier: value.identifier.into(),
      is_name_space: value.is_name_space,
      from_dependency: value.from_dependency,
      exports: value.exports.into_iter().collect::<Vec<_>>(),
      variable: value.variable,
    }
  }
}

#[napi(object)]
pub struct JsRsdoctorVariable {
  pub ukey: i32,
  pub name: String,
  pub module: i32,
  pub used_info: String,
  pub identififer: JsRsdoctorStatement,
  pub exported: Option<i32>,
}

impl From<RsdoctorVariable> for JsRsdoctorVariable {
  fn from(value: RsdoctorVariable) -> Self {
    JsRsdoctorVariable {
      ukey: value.ukey,
      name: value.name,
      module: value.module,
      used_info: value.used_info,
      identififer: value.identififer.into(),
      exported: value.exported,
    }
  }
}

#[napi(object)]
pub struct JsRsdoctorExportInfo {
  pub ukey: i32,
  pub name: String,
  pub from: Option<i32>,
  pub variable: Option<i32>,
  pub identifier: Option<JsRsdoctorStatement>,
  pub side_effects: Vec<i32>,
}

impl From<RsdoctorExportInfo> for JsRsdoctorExportInfo {
  fn from(value: RsdoctorExportInfo) -> Self {
    JsRsdoctorExportInfo {
      ukey: value.ukey,
      name: value.name,
      from: value.from,
      variable: value.variable,
      identifier: value.identifier.map(|i| i.into()),
      side_effects: value.side_effects.into_iter().collect::<Vec<_>>(),
    }
  }
}

#[napi(object)]
pub struct JsRsdoctorStatement {
  pub module: i32,
  pub source_position: Option<JsRsdoctorSourceRange>,
  pub transformed_position: JsRsdoctorSourceRange,
}

impl From<RsdoctorStatement> for JsRsdoctorStatement {
  fn from(value: RsdoctorStatement) -> Self {
    JsRsdoctorStatement {
      module: value.module,
      source_position: value.source_position.map(|p| p.into()),
      transformed_position: value.transformed_position.into(),
    }
  }
}

#[napi(object)]
pub struct JsRsdoctorSourceRange {
  pub start: JsRsdoctorSourcePosition,
  pub end: Option<JsRsdoctorSourcePosition>,
}

impl From<RsdoctorSourceRange> for JsRsdoctorSourceRange {
  fn from(value: RsdoctorSourceRange) -> Self {
    JsRsdoctorSourceRange {
      start: value.start.into(),
      end: value.end.map(|p| p.into()),
    }
  }
}

#[napi(object)]
pub struct JsRsdoctorSourcePosition {
  pub line: Option<i32>,
  pub column: Option<i32>,
  pub index: Option<i32>,
}

impl From<RsdoctorSourcePosition> for JsRsdoctorSourcePosition {
  fn from(value: RsdoctorSourcePosition) -> Self {
    JsRsdoctorSourcePosition {
      line: value.line,
      column: value.column,
      index: value.index,
    }
  }
}

#[napi(object)]
pub struct JsRsdoctorChunkModules {
  pub chunk: i32,
  pub modules: Vec<i32>,
}

impl From<RsdoctorChunkModules> for JsRsdoctorChunkModules {
  fn from(value: RsdoctorChunkModules) -> Self {
    JsRsdoctorChunkModules {
      chunk: value.chunk,
      modules: value.modules.into_iter().collect::<Vec<_>>(),
    }
  }
}

#[napi(object)]
pub struct JsRsdoctorModuleGraph {
  pub modules: Vec<JsRsdoctorModule>,
  pub dependencies: Vec<JsRsdoctorDependency>,
  pub chunk_modules: Vec<JsRsdoctorChunkModules>,
}

impl From<RsdoctorModuleGraph> for JsRsdoctorModuleGraph {
  fn from(value: RsdoctorModuleGraph) -> Self {
    JsRsdoctorModuleGraph {
      modules: value.modules.into_iter().map(|m| m.into()).collect(),
      dependencies: value.dependencies.into_iter().map(|d| d.into()).collect(),
      chunk_modules: value.chunk_modules.into_iter().map(|c| c.into()).collect(),
    }
  }
}

#[napi(object)]
pub struct JsRsdoctorChunkGraph {
  pub chunks: Vec<JsRsdoctorChunk>,
  pub entrypoints: Vec<JsRsdoctorEntrypoint>,
}

impl From<RsdoctorChunkGraph> for JsRsdoctorChunkGraph {
  fn from(value: RsdoctorChunkGraph) -> Self {
    JsRsdoctorChunkGraph {
      chunks: value.chunks.into_iter().map(|c| c.into()).collect(),
      entrypoints: value.entrypoints.into_iter().map(|e| e.into()).collect(),
    }
  }
}

#[napi(object)]
pub struct JsRsdoctorModuleId {
  pub module: i32,
  pub render_id: String,
}

impl From<RsdoctorModuleId> for JsRsdoctorModuleId {
  fn from(value: RsdoctorModuleId) -> Self {
    JsRsdoctorModuleId {
      module: value.module,
      render_id: value.render_id,
    }
  }
}

#[napi(object)]
pub struct JsRsdoctorModuleOriginalSource {
  pub module: i32,
  pub source: String,
  pub size: i32,
}

impl From<RsdoctorModuleOriginalSource> for JsRsdoctorModuleOriginalSource {
  fn from(value: RsdoctorModuleOriginalSource) -> Self {
    JsRsdoctorModuleOriginalSource {
      module: value.module,
      source: value.source,
      size: value.size,
    }
  }
}

#[napi(object)]
pub struct JsRsdoctorModuleSourcesPatch {
  pub module_original_sources: Vec<JsRsdoctorModuleOriginalSource>,
}

impl From<RsdoctorModuleSourcesPatch> for JsRsdoctorModuleSourcesPatch {
  fn from(value: RsdoctorModuleSourcesPatch) -> Self {
    JsRsdoctorModuleSourcesPatch {
      module_original_sources: value
        .module_original_sources
        .into_iter()
        .map(|m| m.into())
        .collect(),
    }
  }
}

#[napi(object)]
pub struct JsRsdoctorModuleIdsPatch {
  pub module_ids: Vec<JsRsdoctorModuleId>,
}

impl From<RsdoctorModuleIdsPatch> for JsRsdoctorModuleIdsPatch {
  fn from(value: RsdoctorModuleIdsPatch) -> Self {
    JsRsdoctorModuleIdsPatch {
      module_ids: value.module_ids.into_iter().map(|m| m.into()).collect(),
    }
  }
}

#[napi(object)]
pub struct JsRsdoctorChunkAssets {
  pub chunk: i32,
  pub assets: Vec<i32>,
}

impl From<RsdoctorChunkAssets> for JsRsdoctorChunkAssets {
  fn from(value: RsdoctorChunkAssets) -> Self {
    JsRsdoctorChunkAssets {
      chunk: value.chunk,
      assets: value.assets.into_iter().collect::<Vec<_>>(),
    }
  }
}

#[napi(object)]
pub struct JsRsdoctorEntrypointAssets {
  pub entrypoint: i32,
  pub assets: Vec<i32>,
}

impl From<RsdoctorEntrypointAssets> for JsRsdoctorEntrypointAssets {
  fn from(value: RsdoctorEntrypointAssets) -> Self {
    JsRsdoctorEntrypointAssets {
      entrypoint: value.entrypoint,
      assets: value.assets.into_iter().collect::<Vec<_>>(),
    }
  }
}

#[napi(object)]
pub struct JsRsdoctorAssetPatch {
  pub assets: Vec<JsRsdoctorAsset>,
  pub chunk_assets: Vec<JsRsdoctorChunkAssets>,
  pub entrypoint_assets: Vec<JsRsdoctorEntrypointAssets>,
}

impl From<RsdoctorAssetPatch> for JsRsdoctorAssetPatch {
  fn from(value: RsdoctorAssetPatch) -> Self {
    JsRsdoctorAssetPatch {
      assets: value.assets.into_iter().map(|a| a.into()).collect(),
      chunk_assets: value.chunk_assets.into_iter().map(|c| c.into()).collect(),
      entrypoint_assets: value
        .entrypoint_assets
        .into_iter()
        .map(|e| e.into())
        .collect(),
    }
  }
}

#[napi(object, object_to_js = false)]
pub struct RawRsdoctorPluginOptions {
  #[napi(ts_type = "boolean | Array<'graph' | 'ids' | 'sources'>")]
  pub module_graph_features: Either<bool, Vec<String>>,
  #[napi(ts_type = "boolean | Array<'graph' | 'assets'>")]
  pub chunk_graph_features: Either<bool, Vec<String>>,
  #[napi(ts_type = "{ module?: boolean; cheap?: boolean } | undefined")]
  pub source_map_features: Option<JsRsdoctorSourceMapFeatures>,
}

#[napi(object)]
pub struct JsRsdoctorSourceMapFeatures {
  pub cheap: Option<bool>,
  pub module: Option<bool>,
}

#[napi(object)]
pub struct JsRsdoctorJsonAssetSize {
  pub path: String,
  pub size: i32,
}

impl From<RsdoctorJsonAssetSize> for JsRsdoctorJsonAssetSize {
  fn from(value: RsdoctorJsonAssetSize) -> Self {
    JsRsdoctorJsonAssetSize {
      path: value.path,
      size: value.size,
    }
  }
}

#[napi(object)]
pub struct JsRsdoctorJsonAssetSizesPatch {
  pub json_assets: Vec<JsRsdoctorJsonAssetSize>,
}

impl From<RsdoctorJsonAssetSizesPatch> for JsRsdoctorJsonAssetSizesPatch {
  fn from(value: RsdoctorJsonAssetSizesPatch) -> Self {
    JsRsdoctorJsonAssetSizesPatch {
      json_assets: value.json_assets.into_iter().map(|j| j.into()).collect(),
    }
  }
}

impl From<RawRsdoctorPluginOptions> for RsdoctorPluginOptions {
  fn from(value: RawRsdoctorPluginOptions) -> Self {
    let mut source_map_features = RsdoctorPluginSourceMapFeature {
      cheap: false,
      module: true,
    };

    if let Some(features) = value.source_map_features {
      if let Some(cheap) = features.cheap {
        source_map_features.cheap = cheap;
      }
      if let Some(module) = features.module {
        source_map_features.module = module;
      }
    }

    Self {
      module_graph_features: match value.module_graph_features {
        Either::A(true) => HashSet::from([
          RsdoctorPluginModuleGraphFeature::ModuleGraph,
          RsdoctorPluginModuleGraphFeature::ModuleIds,
          RsdoctorPluginModuleGraphFeature::ModuleSources,
        ]),
        Either::A(false) => HashSet::new(),
        Either::B(features) => features
          .into_iter()
          .map(RsdoctorPluginModuleGraphFeature::from)
          .collect::<HashSet<_>>(),
      },
      chunk_graph_features: match value.chunk_graph_features {
        Either::A(true) => HashSet::from([
          RsdoctorPluginChunkGraphFeature::ChunkGraph,
          RsdoctorPluginChunkGraphFeature::Assets,
        ]),
        Either::A(false) => HashSet::new(),
        Either::B(features) => features
          .into_iter()
          .map(RsdoctorPluginChunkGraphFeature::from)
          .collect::<HashSet<_>>(),
      },
      source_map_features,
    }
  }
}
