use std::sync::Arc;

use rspack_cacheable::cacheable;
use rspack_error::Result;
use rspack_tasks::{
  get_current_code_generation_result_id, get_current_dependency_id,
  set_current_code_generation_result_id, set_current_dependency_id,
};

use super::{
  super::{codec::CacheCodec, storage::Storage},
  Occasion,
};

pub const SCOPE: &str = "meta";

/// The meta data.
#[cacheable]
struct Meta {
  pub max_dependencies_id: u32,
  pub max_code_generation_result_id: u32,
}

/// Meta Occasion is used to save compiler state.
#[derive(Debug)]
pub struct MetaOccasion {
  codec: Arc<CacheCodec>,
}

impl MetaOccasion {
  pub fn new(codec: Arc<CacheCodec>) -> Self {
    Self { codec }
  }
}

impl Occasion for MetaOccasion {
  /// Meta has no structured artifact: it reads/writes a single global counter.
  type Artifact = ();

  fn name(&self) -> &'static str {
    "meta"
  }

  #[tracing::instrument("Cache::Occasion::Meta::reset", skip_all)]
  fn reset(&self, storage: &mut dyn Storage) {
    storage.reset(SCOPE);
  }

  #[tracing::instrument("Cache::Occasion::Meta::save", skip_all)]
  fn save(&self, storage: &mut dyn Storage, _artifact: &()) {
    let meta = Meta {
      max_dependencies_id: get_current_dependency_id(),
      max_code_generation_result_id: get_current_code_generation_result_id(),
    };
    storage.set(
      SCOPE,
      "default".as_bytes().to_vec(),
      self.codec.encode(&meta).expect("should encode success"),
    );
  }

  #[tracing::instrument("Cache::Occasion::Meta::recovery", skip_all)]
  async fn recovery(&self, storage: &dyn Storage) -> Result<()> {
    let Some((_, value)) = storage.load(SCOPE).await?.pop() else {
      return Ok(());
    };

    let meta: Meta = self.codec.decode(&value).expect("should decode success");
    if get_current_dependency_id() != 0 {
      panic!("The global dependency id generator is not 0 when the persistent cache is restored.");
    }
    if get_current_code_generation_result_id() != 0 {
      panic!(
        "The global code generation result id generator is not 0 when the persistent cache is restored."
      );
    }
    set_current_dependency_id(meta.max_dependencies_id);
    set_current_code_generation_result_id(meta.max_code_generation_result_id);
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use rspack_tasks::{
    CompilerContext, get_current_code_generation_result_id, get_current_dependency_id,
    set_current_code_generation_result_id, set_current_dependency_id, within_compiler_context,
  };

  use super::{MetaOccasion, Occasion};
  use crate::cache::persistent::storage::MemoryStorage;

  #[tokio::test]
  async fn should_restore_compiler_id_generators() {
    let occasion = MetaOccasion::new(Arc::new(super::CacheCodec::new(None)));
    let mut storage = MemoryStorage::default();

    within_compiler_context(Arc::new(CompilerContext::new()), async {
      set_current_dependency_id(7);
      set_current_code_generation_result_id(11);
      occasion.save(&mut storage, &());
    })
    .await;

    within_compiler_context(Arc::new(CompilerContext::new()), async {
      occasion
        .recovery(&storage)
        .await
        .expect("should recover meta");
      assert_eq!(get_current_dependency_id(), 7);
      assert_eq!(get_current_code_generation_result_id(), 11);
    })
    .await;
  }
}
