use async_trait::async_trait;
use futures::{TryFutureExt, future::join_all};
use itertools::Itertools;

use super::{SplitPackStrategy, util::get_indexed_packs};
use crate::{
  error::{Error, ErrorType, Result, ValidateResult},
  pack::{data::PackScope, strategy::ScopeValidateStrategy},
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
    let (_, pack_list) = get_indexed_packs(scope, None);

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
        .map_err(|e| Error::from_error(Some(ErrorType::Validate), Some(scope.name), e.into()))
      });

    let validate_results = join_all(tasks)
      .await
      .into_iter()
      .collect::<Result<Vec<_>>>()?;

    let mut invalid_packs = validate_results
      .iter()
      .zip(pack_list)
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

  use rspack_paths::Utf8PathBuf;
  use rustc_hash::FxHashSet as HashSet;

  use crate::{
    FileSystem,
    error::{Error, ErrorType, Result, ValidateResult},
    pack::{
      data::{PackOptions, PackScope, RootMeta, ScopeMeta},
      strategy::{
        ScopeReadStrategy, ScopeValidateStrategy, ScopeWriteStrategy, SplitPackStrategy,
        split::{
          handle_file::prepare_scope,
          util::{
            flag_scope_wrote,
            test_pack_utils::{
              UpdateVal, clean_strategy, create_strategies, flush_file_mtime, mock_root_meta_file,
              mock_scope_meta_file, mock_updates, save_scope,
            },
          },
        },
      },
    },
  };

  async fn test_valid_meta(scope_path: Utf8PathBuf, strategy: &SplitPackStrategy) -> Result<()> {
    let same_options = Arc::new(PackOptions {
      bucket_size: 10,
      pack_size: 100,
    });
    let mut scope = PackScope::new("scope_name", scope_path, same_options);
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
    let mut scope = PackScope::new(
      "scope_name",
      scope_path.clone(),
      bucket_changed_options.clone(),
    );
    strategy.ensure_meta(&mut scope).await?;
    if let ValidateResult::Invalid(detail) = strategy.validate_meta(&mut scope).await? {
      let error = Error::from_detail(Some(ErrorType::Validate), Some("test_scope"), detail);
      assert_eq!(
        error.to_string(),
        "validate scope `test_scope` failed due to `options.bucketSize` changed"
      );
    } else {
      panic!("should be invalid");
    }

    let max_size_changed_options = Arc::new(PackOptions {
      bucket_size: 10,
      pack_size: 99,
    });
    let mut scope = PackScope::new(
      "scope_name",
      scope_path.clone(),
      max_size_changed_options.clone(),
    );
    strategy.ensure_meta(&mut scope).await?;
    if let ValidateResult::Invalid(detail) = strategy.validate_meta(&mut scope).await? {
      let error = Error::from_detail(Some(ErrorType::Validate), Some("test_scope"), detail);
      assert_eq!(
        error.to_string(),
        "validate scope `test_scope` failed due to `options.packSize` changed"
      );
    } else {
      panic!("should be invalid");
    }

    Ok(())
  }

  async fn test_valid_packs(
    scope_path: Utf8PathBuf,
    strategy: &SplitPackStrategy,
    options: Arc<PackOptions>,
  ) -> Result<()> {
    let mut scope = PackScope::new("scope_name", scope_path, options);
    strategy.ensure_keys(&mut scope).await?;
    let validated = strategy.validate_packs(&mut scope).await?;
    assert!(validated.is_valid());

    Ok(())
  }

  async fn test_flush_packs_mtime(
    scope_path: Utf8PathBuf,
    strategy: &SplitPackStrategy,
    fs: Arc<dyn FileSystem>,
    options: Arc<PackOptions>,
    files: HashSet<Utf8PathBuf>,
  ) -> Result<()> {
    let mut scope = PackScope::new("scope_name", scope_path, options);
    // test refresh mtime
    for file in files {
      if !file.to_string().contains("scope_meta") {
        flush_file_mtime(&file, fs.clone()).await?;
      }
    }

    strategy.ensure_keys(&mut scope).await?;
    assert!(strategy.validate_packs(&mut scope).await?.is_valid());

    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn should_validate_scope_meta() -> Result<()> {
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

      test_valid_meta(scope_path.clone(), &strategy).await?;
      test_invalid_option_changed(scope_path.clone(), &strategy).await?;
    }
    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn should_validate_scope_packs() -> Result<()> {
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
      let mut mock_scope = PackScope::empty("scope_name", scope_path.clone(), pack_options.clone());
      let updates = mock_updates(0, 100, 30, UpdateVal::Value("val".to_string()));
      strategy
        .update_scope(&mut mock_scope, updates)
        .await
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

      test_valid_packs(scope_path.clone(), &strategy, pack_options.clone()).await?;

      test_flush_packs_mtime(
        scope_path.clone(),
        &strategy,
        strategy.fs.clone(),
        pack_options.clone(),
        changed.wrote_files,
      )
      .await?;
    }
    Ok(())
  }
}
