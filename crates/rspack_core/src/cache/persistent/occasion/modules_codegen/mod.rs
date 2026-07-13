use std::sync::Arc;

use rspack_error::Result;

use super::{
  super::{codec::CacheCodec, storage::Storage},
  Occasion,
};
use crate::CodeGenerationResults;

pub const SCOPE: &str = "occasion_modules_codegen";
const ARTIFACT_KEY: &[u8] = b"code_generation_results";

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
    match self.codec.encode(artifact) {
      Ok(bytes) => {
        storage.set(SCOPE, ARTIFACT_KEY.to_vec(), bytes);
        tracing::debug!("saved modules codegen persistent cache artifact");
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

    let artifact = self.codec.decode::<CodeGenerationResults>(&value)?;
    tracing::debug!("recovered modules codegen persistent cache artifact");
    Ok(artifact)
  }
}
