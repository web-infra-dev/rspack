use std::sync::Arc;

use async_trait::async_trait;
use futures::{future::join_all, TryFutureExt};
use itertools::Itertools;
use rspack_error::{error, Result};
use rspack_paths::{Utf8Path, Utf8PathBuf};

use super::{util::get_indexed_packs, SplitPackStrategy};
use crate::pack::{
  data::{Pack, PackContents, PackFileMeta, PackKeys, PackScope, ScopeMeta},
  fs::PackFS,
  strategy::{PackReadStrategy, ScopeReadStrategy},
};

#[async_trait]
impl ScopeReadStrategy for SplitPackStrategy {
  async fn ensure_meta(&self, scope: &mut PackScope) -> Result<()> {
    if !scope.meta.loaded() {
      let meta_path = ScopeMeta::get_path(&scope.path);
      let meta = read_scope_meta(&meta_path, self.fs.clone())
        .await?
        .unwrap_or_else(|| ScopeMeta::new(&scope.path, &scope.options));
      scope.meta.set_value(meta);
    }
    Ok(())
  }

  async fn ensure_packs(&self, scope: &mut PackScope) -> Result<()> {
    self.ensure_meta(scope).await?;

    if !scope.packs.loaded() {
      scope.packs.set_value(
        scope
          .meta
          .expect_value()
          .packs
          .iter()
          .enumerate()
          .map(|(bucket_id, bucket_pack_metas)| {
            let bucket_path = scope.path.join(bucket_id.to_string());
            bucket_pack_metas
              .iter()
              .map(|pack_meta| Pack::new(bucket_path.join(&pack_meta.name)))
              .collect_vec()
          })
          .collect_vec(),
      );
    }
    Ok(())
  }

  async fn ensure_keys(&self, scope: &mut PackScope) -> Result<()> {
    self.ensure_packs(scope).await?;

    let read_key_results = read_keys(scope, self).await?;
    let packs = scope.packs.expect_value_mut();
    for result in read_key_results {
      if let Some(pack) = packs
        .get_mut(result.bucket_id)
        .and_then(|packs| packs.get_mut(result.pack_pos))
      {
        pack.keys.set_value(result.keys);
      }
    }
    Ok(())
  }

  async fn ensure_contents(&self, scope: &mut PackScope) -> Result<()> {
    self.ensure_keys(scope).await?;

    let read_content_results = read_contents(scope, self).await?;
    let packs = scope.packs.expect_value_mut();
    for result in read_content_results {
      if let Some(pack) = packs
        .get_mut(result.bucket_id)
        .and_then(|packs| packs.get_mut(result.pack_pos))
      {
        pack.contents.set_value(result.contents);
      }
    }
    Ok(())
  }

  fn get_path(&self, str: &str) -> Utf8PathBuf {
    self.root.join(str)
  }
}

async fn read_scope_meta(path: &Utf8Path, fs: Arc<dyn PackFS>) -> Result<Option<ScopeMeta>> {
  if !fs.exists(path).await? {
    return Ok(None);
  }

  let mut reader = fs.read_file(path).await?;

  let option_items = reader
    .read_line()
    .await?
    .split(" ")
    .map(|item| {
      item
        .parse::<usize>()
        .map_err(|e| error!("parse option meta failed: {}", e))
    })
    .collect::<Result<Vec<usize>>>()?;

  if option_items.len() < 3 {
    return Err(error!("option meta not match"));
  }

  let bucket_size = option_items[0];
  let pack_size = option_items[1];
  let last_modified = option_items[2] as u64;

  let mut packs = vec![];
  for _ in 0..bucket_size {
    packs.push(
      reader
        .read_line()
        .await?
        .split(" ")
        .filter(|i| !i.is_empty())
        .map(|i| i.split(",").collect::<Vec<_>>())
        .map(|i| {
          if i.len() < 3 {
            Err(error!("file meta not match"))
          } else {
            Ok(PackFileMeta {
              name: i[0].to_owned(),
              hash: i[1].to_owned(),
              size: i[2]
                .parse::<usize>()
                .map_err(|e| error!("parse file meta failed: {}", e))?,
              wrote: true,
            })
          }
        })
        .collect::<Result<Vec<PackFileMeta>>>()?,
    );
  }

  if packs.len() < bucket_size {
    return Err(error!("bucket size not match"));
  }

  Ok(Some(ScopeMeta {
    path: path.to_path_buf(),
    bucket_size,
    pack_size,
    last_modified,
    packs,
  }))
}

#[derive(Debug)]
struct ReadKeysResult {
  pub bucket_id: usize,
  pub pack_pos: usize,
  pub keys: PackKeys,
}

fn read_keys_filter(pack: &Pack, _: &PackFileMeta) -> bool {
  !pack.keys.loaded()
}

async fn read_keys(scope: &PackScope, strategy: &SplitPackStrategy) -> Result<Vec<ReadKeysResult>> {
  let (pack_indexes, packs) = get_indexed_packs(scope, Some(&read_keys_filter))?;

  let tasks = packs
    .into_iter()
    .map(|i| {
      let strategy = strategy.clone();
      let path = i.1.path.clone();
      tokio::spawn(async move { strategy.read_pack_keys(&path).await }).map_err(|e| error!("{}", e))
    })
    .collect_vec();

  let pack_keys = join_all(tasks).await.into_iter().process_results(|iter| {
    iter
      .into_iter()
      .process_results(|iter| iter.map(|x| x.unwrap_or_default()).collect_vec())
  })??;

  Ok(
    pack_keys
      .into_iter()
      .zip(pack_indexes.into_iter())
      .map(|(keys, (bucket_id, pack_pos))| ReadKeysResult {
        bucket_id,
        pack_pos,
        keys,
      })
      .collect_vec(),
  )
}

#[derive(Debug)]
struct ReadContentsResult {
  pub bucket_id: usize,
  pub pack_pos: usize,
  pub contents: PackContents,
}

fn read_contents_filter(pack: &Pack, _: &PackFileMeta) -> bool {
  pack.keys.loaded() && !pack.contents.loaded()
}

async fn read_contents(
  scope: &PackScope,
  strategy: &SplitPackStrategy,
) -> Result<Vec<ReadContentsResult>> {
  let (pack_indexes, packs) = get_indexed_packs(scope, Some(&read_contents_filter))?;
  let tasks = packs
    .into_iter()
    .map(|i| {
      let strategy = strategy.to_owned();
      let path = i.1.path.to_owned();
      tokio::spawn(async move { strategy.read_pack_contents(&path).await })
        .map_err(|e| error!("{}", e))
    })
    .collect_vec();
  let pack_contents = join_all(tasks).await.into_iter().process_results(|iter| {
    iter
      .into_iter()
      .process_results(|iter| iter.map(|x| x.unwrap_or_default()).collect_vec())
  })??;

  Ok(
    pack_contents
      .into_iter()
      .zip(pack_indexes.into_iter())
      .map(|(contents, (bucket_id, pack_pos))| ReadContentsResult {
        bucket_id,
        pack_pos,
        contents,
      })
      .collect_vec(),
  )
}

#[cfg(test)]
mod tests {

  use std::{collections::HashSet, sync::Arc};

  use itertools::Itertools;
  use rspack_error::Result;
  use rspack_paths::Utf8Path;

  use crate::pack::{
    data::{PackOptions, PackScope, ScopeMeta},
    fs::PackFS,
    strategy::{
      split::util::test_pack_utils::{
        clean_strategy, create_strategies, mock_meta_file, mock_pack_file,
      },
      ScopeReadStrategy, SplitPackStrategy,
    },
  };

  async fn mock_scope(path: &Utf8Path, fs: &dyn PackFS, options: &PackOptions) -> Result<()> {
    mock_meta_file(&ScopeMeta::get_path(path), fs, options, 3).await?;
    for bucket_id in 0..options.bucket_size {
      for pack_no in 0..3 {
        let unique_id = format!("{}_{}", bucket_id, pack_no);
        let pack_name = format!("pack_name_{}_{}", bucket_id, pack_no);
        let pack_path = path.join(format!("./{}/{}", bucket_id, pack_name));
        mock_pack_file(&pack_path, &unique_id, 10, fs).await?;
      }
    }

    Ok(())
  }

  async fn test_read_meta(scope: &mut PackScope, strategy: &SplitPackStrategy) -> Result<()> {
    strategy.ensure_meta(scope).await?;
    let meta = scope.meta.expect_value();
    assert_eq!(meta.path, ScopeMeta::get_path(&scope.path));
    assert_eq!(meta.bucket_size, scope.options.bucket_size);
    assert_eq!(meta.pack_size, scope.options.pack_size);
    assert_eq!(meta.packs.len(), scope.options.bucket_size);
    assert_eq!(
      meta
        .packs
        .iter()
        .flatten()
        .map(|i| (i.name.as_str(), i.hash.as_str(), i.size, i.wrote))
        .collect_vec(),
      vec![
        ("pack_name_0_0", "pack_hash_0_0", 100, true),
        ("pack_name_0_1", "pack_hash_0_1", 100, true),
        ("pack_name_0_2", "pack_hash_0_2", 100, true),
      ]
    );

    Ok(())
  }

  async fn test_read_packs(scope: &mut PackScope, strategy: &SplitPackStrategy) -> Result<()> {
    strategy.ensure_keys(scope).await?;

    let all_keys = scope
      .packs
      .expect_value()
      .iter()
      .flatten()
      .flat_map(|pack| pack.keys.expect_value().to_owned())
      .map(|x| x.as_ref().to_owned())
      .collect::<HashSet<_>>();
    assert!(
      all_keys.contains(format!("key_{}_{}_{}", scope.options.bucket_size - 1, 2, 9).as_bytes())
    );

    strategy.ensure_contents(scope).await?;

    let all_contents = scope
      .packs
      .expect_value()
      .iter()
      .flatten()
      .flat_map(|pack| pack.contents.expect_value().to_owned())
      .map(|x| x.as_ref().to_owned())
      .collect::<HashSet<_>>();
    assert!(all_contents
      .contains(format!("val_{}_{}_{}", scope.options.bucket_size - 1, 2, 9).as_bytes()));

    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn should_read_scope() {
    for strategy in create_strategies("read_scope") {
      clean_strategy(&strategy).await;
      let options = Arc::new(PackOptions {
        bucket_size: 1,
        pack_size: 16,
        expire: 60000,
      });
      let mut scope = PackScope::new(strategy.get_path("scope_name"), options.clone());

      mock_scope(&scope.path, strategy.fs.as_ref(), &scope.options)
        .await
        .expect("should mock packs");

      let _ = test_read_meta(&mut scope, &strategy).await.map_err(|e| {
        panic!("{}", e);
      });
      let _ = test_read_packs(&mut scope, &strategy).await.map_err(|e| {
        panic!("{}", e);
      });
    }
  }
}
