use async_trait::async_trait;
use futures::{future::join_all, TryFutureExt};
use itertools::Itertools;
use rspack_error::{error, Result};

use super::{util::get_indexed_packs, SplitPackStrategy};
use crate::pack::{
  data::PackScope,
  strategy::{ScopeValidateStrategy, ValidateResult},
};

#[async_trait]
impl ScopeValidateStrategy for SplitPackStrategy {
  async fn validate_meta(&self, scope: &mut PackScope) -> Result<ValidateResult> {
    let meta = scope.meta.expect_value();

    if meta.bucket_size != scope.options.bucket_size {
      return Ok(ValidateResult::invalid("`options.bucketSize` changed"));
    }

    if meta.pack_size != scope.options.pack_size {
      return Ok(ValidateResult::invalid("`options.packSize` changed"));
    }

    return Ok(ValidateResult::Valid);
  }

  async fn validate_packs(&self, scope: &mut PackScope) -> Result<ValidateResult> {
    let (_, pack_list) = get_indexed_packs(scope, None)?;

    let tasks = pack_list
      .iter()
      .filter(|(_, pack)| !pack.loaded())
      .map(|(pack_meta, pack)| {
        let strategy = self.clone();
        let path = pack.path.to_owned();
        let hash = pack_meta.hash.to_owned();
        let keys = pack.keys.expect_value().to_owned();
        tokio::spawn(async move {
          match strategy
            .get_pack_hash(&path, &keys, &Default::default())
            .await
          {
            Ok(res) => hash == res,
            Err(_) => false,
          }
        })
        .map_err(|e| error!("{}", e))
      });

    let validate_results = join_all(tasks)
      .await
      .into_iter()
      .collect::<Result<Vec<_>>>()?;

    let mut invalid_packs = validate_results
      .iter()
      .zip(pack_list.into_iter())
      .filter(|(is_valid, _)| !*is_valid)
      .map(|(_, (_, pack))| pack)
      .collect::<Vec<_>>();
    invalid_packs.sort_by(|a, b| a.path.cmp(&b.path));
    if invalid_packs.is_empty() {
      return Ok(ValidateResult::Valid);
    } else {
      let invalid_pack_paths = invalid_packs
        .iter()
        .map(|pack| pack.path.to_string())
        .collect_vec();
      Ok(ValidateResult::invalid_with_packs(
        "some packs are modified",
        invalid_pack_paths,
      ))
    }
  }
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use rspack_error::Result;
  use rspack_paths::Utf8PathBuf;
  use rustc_hash::FxHashSet as HashSet;

  use crate::pack::{
    data::{PackOptions, PackScope, RootMeta, ScopeMeta},
    fs::PackFS,
    strategy::{
      split::{
        handle_file::prepare_scope,
        util::{
          flag_scope_wrote,
          test_pack_utils::{
            clean_strategy, create_strategies, flush_file_mtime, mock_root_meta_file,
            mock_scope_meta_file, mock_updates, save_scope, UpdateVal,
          },
        },
      },
      ScopeReadStrategy, ScopeValidateStrategy, ScopeWriteStrategy, SplitPackStrategy,
      ValidateResult,
    },
  };

  async fn test_valid_meta(scope_path: Utf8PathBuf, strategy: &SplitPackStrategy) -> Result<()> {
    let same_options = Arc::new(PackOptions {
      bucket_size: 10,
      pack_size: 100,
    });
    let mut scope = PackScope::new(scope_path, same_options);
    strategy.ensure_meta(&mut scope).await?;
    let validated = strategy.validate_meta(&mut scope).await?;
    assert!(validated.is_valid());

    Ok(())
  }

  async fn test_invalid_option_changed(
    scope_path: Utf8PathBuf,
    strategy: &SplitPackStrategy,
  ) -> Result<()> {
    let bucket_changed_options = Arc::new(PackOptions {
      bucket_size: 1,
      pack_size: 100,
    });
    let mut scope = PackScope::new(scope_path.clone(), bucket_changed_options.clone());
    strategy.ensure_meta(&mut scope).await?;
    let validated: ValidateResult = strategy.validate_meta(&mut scope).await?;
    assert_eq!(
      validated.to_string(),
      "validation failed due to `options.bucketSize` changed"
    );

    let max_size_changed_options = Arc::new(PackOptions {
      bucket_size: 10,
      pack_size: 99,
    });
    let mut scope = PackScope::new(scope_path.clone(), max_size_changed_options.clone());
    strategy.ensure_meta(&mut scope).await?;
    let validated: ValidateResult = strategy.validate_meta(&mut scope).await?;
    assert_eq!(
      validated.to_string(),
      "validation failed due to `options.packSize` changed"
    );

    Ok(())
  }

  async fn test_valid_packs(
    scope_path: Utf8PathBuf,
    strategy: &SplitPackStrategy,
    options: Arc<PackOptions>,
  ) -> Result<()> {
    let mut scope = PackScope::new(scope_path, options);
    strategy.ensure_keys(&mut scope).await?;
    let validated = strategy.validate_packs(&mut scope).await?;
    assert!(validated.is_valid());

    Ok(())
  }

  async fn test_invalid_packs_changed(
    scope_path: Utf8PathBuf,
    strategy: &SplitPackStrategy,
    fs: Arc<dyn PackFS>,
    options: Arc<PackOptions>,
    files: HashSet<Utf8PathBuf>,
  ) -> Result<()> {
    let mut scope = PackScope::new(scope_path, options);
    for file in files {
      if !file.to_string().contains("scope_meta") {
        flush_file_mtime(&file, fs.clone()).await?;
      }
    }

    strategy.ensure_keys(&mut scope).await?;
    let validated = strategy.validate_packs(&mut scope).await?;
    assert!(validated
      .to_string()
      .starts_with("validation failed due to some packs are modified:"));
    assert_eq!(validated.to_string().split("\n").count(), 7);

    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn should_validate_scope_meta() {
    for strategy in create_strategies("valid_scope_meta") {
      clean_strategy(&strategy).await;
      mock_root_meta_file(
        &RootMeta::get_path(strategy.root.as_ref()),
        strategy.fs.as_ref(),
      )
      .await
      .expect("should mock root meta file");

      let scope_path = strategy.get_path("scope_meta");
      let pack_options = Arc::new(PackOptions {
        bucket_size: 10,
        pack_size: 100,
      });
      mock_scope_meta_file(
        &ScopeMeta::get_path(&scope_path),
        strategy.fs.as_ref(),
        pack_options.as_ref(),
        100,
      )
      .await
      .expect("should mock meta file");

      let _ = test_valid_meta(scope_path.clone(), &strategy)
        .await
        .map_err(|e| panic!("{}", e));

      let _ = test_invalid_option_changed(scope_path.clone(), &strategy)
        .await
        .map_err(|e| panic!("{}", e));
    }
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn should_validate_scope_packs() {
    for strategy in create_strategies("validate_scope_packs") {
      clean_strategy(&strategy).await;
      mock_root_meta_file(
        &RootMeta::get_path(strategy.root.as_ref()),
        strategy.fs.as_ref(),
      )
      .await
      .expect("should mock root meta file");

      let scope_path = strategy.get_path("scope_packs");
      let pack_options = Arc::new(PackOptions {
        bucket_size: 10,
        pack_size: 100,
      });
      let mut mock_scope = PackScope::empty(scope_path.clone(), pack_options.clone());
      let updates = mock_updates(0, 100, 30, UpdateVal::Value("val".to_string()));
      strategy
        .update_scope(&mut mock_scope, updates)
        .expect("should update scope");

      prepare_scope(
        &mock_scope.path,
        &strategy.root,
        &strategy.temp_root,
        strategy.fs.clone(),
      )
      .await
      .expect("should prepare dirs");
      let changed = save_scope(&mut mock_scope, &strategy)
        .await
        .expect("should write scope");
      strategy
        .merge_changed(changed.clone())
        .await
        .expect("should merge changed");

      flag_scope_wrote(&mut mock_scope);

      let _ = test_valid_packs(scope_path.clone(), &strategy, pack_options.clone())
        .await
        .map_err(|e| panic!("{}", e));

      let _ = test_invalid_packs_changed(
        scope_path.clone(),
        &strategy,
        strategy.fs.clone(),
        pack_options.clone(),
        changed.wrote_files,
      )
      .await
      .map_err(|e| panic!("{}", e));
    }
  }
}
