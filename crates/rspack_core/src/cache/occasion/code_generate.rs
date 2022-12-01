use crate::{cache::storage, BoxModule, CodeGenerationResult, NormalModuleAstOrSource};
use futures::future::BoxFuture;
use rspack_error::Result;
use tokio::sync::RwLock;

type Storage = dyn storage::Storage<CodeGenerationResult>;

#[derive(Debug)]
pub struct CodeGenerateOccasion {
  storage: RwLock<Box<Storage>>,
}

impl CodeGenerateOccasion {
  pub fn new(storage: Box<Storage>) -> Self {
    Self {
      storage: RwLock::new(storage),
    }
  }

  pub async fn use_cache<'a, F>(
    &self,
    module: &'a BoxModule,
    generator: F,
  ) -> Result<CodeGenerationResult>
  where
    F: Fn(&'a BoxModule) -> BoxFuture<'a, Result<CodeGenerationResult>>,
  {
    let mut need_cache = false;
    let id = module.identifier().as_ref().to_string();
    if let Some(module) = module.as_normal_module() {
      // only cache normal module
      // TODO cache all module type
      if matches!(module.ast_or_source(), NormalModuleAstOrSource::Unbuild) {
        let storage = self.storage.read().await;
        if let Some(data) = storage.get(&id) {
          return Ok(data);
        }
        // unbuild and no cache is unexpected
        panic!("unexpected unbuild module");
      }
      need_cache = true;
    }

    // run generator and save to cache
    let data = generator(module).await?;
    if need_cache {
      self.storage.write().await.set(id.clone(), data.clone());
    }
    Ok(data)
  }
}
