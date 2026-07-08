use std::sync::Arc;

use rspack_cacheable::{
  cacheable,
  with::{AsPreset, AsVec},
};
use rspack_error::Result;
use rspack_sources::BoxSource;
use rustc_hash::FxHashMap;

use super::{
  super::{codec::CacheCodec, storage::Storage},
  Occasion,
};
use crate::{
  AssetInfo, BindingCell, CachedChunkInitFragment, ChunkInitFragments, CodeGenerationData,
  CodeGenerationDataAssetInfo, CodeGenerationDataFilename, CodeGenerationDataTopLevelDeclarations,
  CodeGenerationDataUrl, CodeGenerationExportsFinalNames, CodeGenerationPublicPathAutoReplace,
  CodeGenerationResult, CodeGenerationResults, ModuleIdentifier, RuntimeGlobals, RuntimeKey,
  SourceType, URLStaticMode, get_runtime_key,
};

pub const SCOPE: &str = "occasion_modules_codegen";
const ARTIFACT_KEY: &[u8] = b"code_generation_results";

#[cacheable]
struct Entry {
  records: Vec<Record>,
}

#[cacheable]
struct Record {
  module: ModuleIdentifier,
  runtime_keys: Vec<RuntimeKey>,
  result: CachedCodeGenerationResult,
}

#[cacheable]
struct CachedCodeGenerationResult {
  sources: Vec<CachedSource>,
  data: Vec<CachedCodeGenerationData>,
  chunk_init_fragments: Vec<CachedChunkInitFragment>,
  runtime_requirements: RuntimeGlobals,
  hash: Option<rspack_hash::RspackHashDigest>,
}

#[cacheable]
struct CachedSource {
  source_type: SourceType,
  #[cacheable(with=AsPreset)]
  source: BoxSource,
}

#[cacheable]
enum CachedCodeGenerationData {
  Url(String),
  PublicPathAutoReplace(bool),
  UrlStaticMode,
  Filename {
    filename: String,
    public_path: String,
  },
  AssetInfo(AssetInfo),
  TopLevelDeclarations(
    #[cacheable(with=AsVec<AsPreset>)] rustc_hash::FxHashSet<rspack_util::atom::Atom>,
  ),
  ExportsFinalNames(FxHashMap<String, String>),
  ChunkInitFragments(Vec<CachedChunkInitFragment>),
}

impl Entry {
  fn try_from_artifact(artifact: &CodeGenerationResults) -> Option<Self> {
    let (runtime_map, result_map) = artifact.inner();
    let mut records = Vec::new();

    for (module, runtime_results) in runtime_map {
      let mut grouped_runtime_keys = FxHashMap::<_, Vec<RuntimeKey>>::default();
      match runtime_results.mode {
        crate::RuntimeMode::Empty => {}
        crate::RuntimeMode::SingleEntry => {
          let runtime = runtime_results.single_runtime.as_ref()?;
          let result = runtime_results.single_value?;
          grouped_runtime_keys
            .entry(result)
            .or_default()
            .push(get_runtime_key(runtime).clone());
        }
        crate::RuntimeMode::Map => {
          for (runtime_key, result) in &runtime_results.map {
            grouped_runtime_keys
              .entry(*result)
              .or_default()
              .push(runtime_key.clone());
          }
        }
      }

      for (result_id, runtime_keys) in grouped_runtime_keys {
        if runtime_keys.is_empty() {
          continue;
        }
        let result = result_map.get(&result_id)?;
        records.push(Record {
          module: *module,
          runtime_keys,
          result: CachedCodeGenerationResult::try_from_result(result)?,
        });
      }
    }

    Some(Self { records })
  }

  fn into_artifact(self) -> CodeGenerationResults {
    let mut artifact = CodeGenerationResults::default();
    for record in self.records {
      if record.runtime_keys.is_empty() {
        continue;
      }
      artifact.insert_with_runtime_keys(
        record.module,
        record.result.into_result(),
        record.runtime_keys,
      );
    }
    artifact
  }
}

impl CachedCodeGenerationResult {
  fn try_from_result(result: &CodeGenerationResult) -> Option<Self> {
    if result.concatenation_scope.is_some() {
      return None;
    }

    Some(Self {
      sources: result
        .inner
        .as_ref()
        .iter()
        .map(|(source_type, source)| CachedSource {
          source_type: *source_type,
          source: source.clone(),
        })
        .collect(),
      data: cache_code_generation_data(&result.data)?,
      chunk_init_fragments: cache_chunk_init_fragments(&result.chunk_init_fragments)?,
      runtime_requirements: result.runtime_requirements,
      hash: result.hash.clone(),
    })
  }

  fn into_result(self) -> CodeGenerationResult {
    let mut sources = FxHashMap::default();
    for source in self.sources {
      sources.insert(source.source_type, source.source);
    }

    CodeGenerationResult {
      inner: BindingCell::from(sources),
      data: restore_code_generation_data(self.data),
      chunk_init_fragments: self
        .chunk_init_fragments
        .into_iter()
        .map(CachedChunkInitFragment::into_fragment)
        .collect(),
      runtime_requirements: self.runtime_requirements,
      hash: self.hash,
      id: Default::default(),
      concatenation_scope: None,
    }
  }
}

fn cache_code_generation_data(data: &CodeGenerationData) -> Option<Vec<CachedCodeGenerationData>> {
  let mut cached = Vec::new();

  if let Some(item) = data.get::<CodeGenerationDataUrl>() {
    cached.push(CachedCodeGenerationData::Url(item.inner().to_string()));
  }
  if let Some(item) = data.get::<CodeGenerationPublicPathAutoReplace>() {
    cached.push(CachedCodeGenerationData::PublicPathAutoReplace(item.0));
  }
  if data.contains::<URLStaticMode>() {
    cached.push(CachedCodeGenerationData::UrlStaticMode);
  }
  if let Some(item) = data.get::<CodeGenerationDataFilename>() {
    cached.push(CachedCodeGenerationData::Filename {
      filename: item.filename().to_string(),
      public_path: item.public_path().to_string(),
    });
  }
  if let Some(item) = data.get::<CodeGenerationDataAssetInfo>() {
    cached.push(CachedCodeGenerationData::AssetInfo(item.inner().clone()));
  }
  if let Some(item) = data.get::<CodeGenerationDataTopLevelDeclarations>() {
    cached.push(CachedCodeGenerationData::TopLevelDeclarations(
      item.inner().clone(),
    ));
  }
  if let Some(item) = data.get::<CodeGenerationExportsFinalNames>() {
    cached.push(CachedCodeGenerationData::ExportsFinalNames(
      item.inner().clone(),
    ));
  }
  if let Some(item) = data.get::<ChunkInitFragments>() {
    cached.push(CachedCodeGenerationData::ChunkInitFragments(
      cache_chunk_init_fragments(item)?,
    ));
  }

  if cached.len() == data.len() {
    Some(cached)
  } else {
    None
  }
}

fn restore_code_generation_data(cached: Vec<CachedCodeGenerationData>) -> CodeGenerationData {
  let mut data = CodeGenerationData::default();

  for item in cached {
    match item {
      CachedCodeGenerationData::Url(url) => {
        data.insert(CodeGenerationDataUrl::new(url));
      }
      CachedCodeGenerationData::PublicPathAutoReplace(value) => {
        data.insert(CodeGenerationPublicPathAutoReplace(value));
      }
      CachedCodeGenerationData::UrlStaticMode => {
        data.insert(URLStaticMode);
      }
      CachedCodeGenerationData::Filename {
        filename,
        public_path,
      } => {
        data.insert(CodeGenerationDataFilename::new(filename, public_path));
      }
      CachedCodeGenerationData::AssetInfo(asset_info) => {
        data.insert(CodeGenerationDataAssetInfo::new(asset_info));
      }
      CachedCodeGenerationData::TopLevelDeclarations(declarations) => {
        data.insert(CodeGenerationDataTopLevelDeclarations::new(declarations));
      }
      CachedCodeGenerationData::ExportsFinalNames(names) => {
        data.insert(CodeGenerationExportsFinalNames::new(names));
      }
      CachedCodeGenerationData::ChunkInitFragments(fragments) => {
        data.insert(
          fragments
            .into_iter()
            .map(CachedChunkInitFragment::into_fragment)
            .collect::<ChunkInitFragments>(),
        );
      }
    }
  }

  data
}

fn cache_chunk_init_fragments(
  fragments: &ChunkInitFragments,
) -> Option<Vec<CachedChunkInitFragment>> {
  fragments
    .iter()
    .map(CachedChunkInitFragment::from_fragment)
    .collect()
}

#[derive(Debug)]
pub struct ModulesCodegenOccasion {
  codec: Arc<CacheCodec>,
}

impl ModulesCodegenOccasion {
  pub fn new(codec: Arc<CacheCodec>) -> Self {
    Self { codec }
  }
}

impl Occasion for ModulesCodegenOccasion {
  type Artifact = CodeGenerationResults;

  fn name(&self) -> &'static str {
    "modules codegen"
  }

  #[tracing::instrument(name = "Cache::Occasion::ModulesCodegen::reset", skip_all)]
  fn reset(&self, storage: &mut dyn Storage) {
    storage.reset(SCOPE);
  }

  #[tracing::instrument(name = "Cache::Occasion::ModulesCodegen::save", skip_all)]
  fn save(&self, storage: &mut dyn Storage, artifact: &CodeGenerationResults) {
    let Some(entry) = Entry::try_from_artifact(artifact) else {
      tracing::debug!(
        "skip modules codegen persistent cache because the artifact contains unsupported data"
      );
      storage.reset(SCOPE);
      return;
    };

    match self.codec.encode(&entry) {
      Ok(bytes) => {
        storage.set(SCOPE, ARTIFACT_KEY.to_vec(), bytes);
        tracing::debug!(
          "saved {} modules codegen persistent cache records",
          entry.records.len()
        );
      }
      Err(err) => {
        tracing::warn!("modules codegen persistent cache encode failed: {:?}", err);
        storage.reset(SCOPE);
      }
    }
  }

  #[tracing::instrument(name = "Cache::Occasion::ModulesCodegen::recovery", skip_all)]
  async fn recovery(&self, storage: &dyn Storage) -> Result<CodeGenerationResults> {
    let items = storage.load(SCOPE).await?;
    let Some((_, value)) = items
      .into_iter()
      .find(|(key, _)| key.as_slice() == ARTIFACT_KEY)
    else {
      return Ok(CodeGenerationResults::default());
    };

    let entry = self.codec.decode::<Entry>(&value)?;
    tracing::debug!(
      "recovered {} modules codegen persistent cache records",
      entry.records.len()
    );
    Ok(entry.into_artifact())
  }
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use rspack_sources::{RawStringSource, Source, SourceExt};

  use super::*;
  use crate::{
    InitFragmentKey, InitFragmentStage, NormalInitFragment, RuntimeSpec,
    cache::persistent::storage::MemoryStorage,
  };

  #[tokio::test]
  async fn should_save_and_recover_code_generation_results() {
    let module = ModuleIdentifier::from("module-a");
    let runtime = RuntimeSpec::from_iter(["main".into()]);
    let mut result = CodeGenerationResult::default()
      .with_javascript(RawStringSource::from_static("module.exports = 1;").boxed());
    result.data.insert(CodeGenerationDataFilename::new(
      "asset.png".into(),
      "/".into(),
    ));
    result.chunk_init_fragments.push(Box::new(
      NormalInitFragment::new(
        "var __init = true;\n".into(),
        InitFragmentStage::StageConstants,
        0,
        InitFragmentKey::Const("__init".into()),
        None,
      )
      .with_top_level_decl_symbols(vec!["__init".into()]),
    ));
    result.runtime_requirements.insert(RuntimeGlobals::REQUIRE);

    let mut artifact = CodeGenerationResults::default();
    artifact.insert(module, result, [runtime.clone()]);

    let occasion = ModulesCodegenOccasion::new(Arc::new(CacheCodec::new(None)));
    let mut storage = MemoryStorage::default();
    occasion.save(&mut storage, &artifact);

    let recovered = occasion.recovery(&storage).await.unwrap();
    let recovered_result = recovered.get(&module, Some(&runtime));
    let source = recovered_result.get(&SourceType::JavaScript).unwrap();
    assert_eq!(source.source().into_string_lossy(), "module.exports = 1;");
    assert!(
      recovered_result
        .data
        .get::<CodeGenerationDataFilename>()
        .is_some()
    );
    assert_eq!(recovered_result.chunk_init_fragments.len(), 1);
    assert!(
      recovered_result
        .runtime_requirements
        .contains(RuntimeGlobals::REQUIRE)
    );
  }

  #[tokio::test]
  async fn should_reset_scope_for_unsupported_codegen_data() {
    #[derive(Clone)]
    struct UnsupportedData;

    let module = ModuleIdentifier::from("module-a");
    let runtime = RuntimeSpec::from_iter(["main".into()]);
    let mut result = CodeGenerationResult::default()
      .with_javascript(RawStringSource::from_static("module.exports = 1;").boxed());
    result.data.insert(UnsupportedData);

    let mut artifact = CodeGenerationResults::default();
    artifact.insert(module, result, [runtime]);

    let occasion = ModulesCodegenOccasion::new(Arc::new(CacheCodec::new(None)));
    let mut storage = MemoryStorage::default();
    storage.set(SCOPE, ARTIFACT_KEY.to_vec(), vec![1]);

    occasion.save(&mut storage, &artifact);

    assert!(storage.load(SCOPE).await.unwrap().is_empty());
  }
}
