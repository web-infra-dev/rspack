use std::sync::Arc;

use async_trait::async_trait;
use itertools::Itertools;
use rspack_error::{error, Result};
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use super::SplitPackStrategy;
use crate::{
  pack::{
    strategy::split::util::get_name, Pack, PackContents, PackContentsState, PackFileMeta, PackKeys,
    PackKeysState, PackWriteStrategy, ScopeUpdate, UpdatePacksResult,
  },
  PackOptions, StorageItemKey, StorageItemValue,
};

#[async_trait]
impl PackWriteStrategy for SplitPackStrategy {
  fn update_packs(
    &self,
    dir: Utf8PathBuf,
    options: &PackOptions,
    packs: HashMap<PackFileMeta, Pack>,
    updates: ScopeUpdate,
  ) -> UpdatePacksResult {
    let pack_dir = dir.to_path_buf();
    let mut indexed_packs = packs.into_iter().enumerate().collect::<HashMap<_, _>>();
    let mut indexed_updates = updates.into_iter().enumerate().collect::<HashMap<_, _>>();

    let current_items_belong =
      indexed_packs
        .iter()
        .fold(HashMap::default(), |mut acc, (pack_index, (_, pack))| {
          let PackKeysState::Value(keys) = &pack.keys else {
            return acc;
          };
          for key in keys {
            acc.insert(key.clone(), *pack_index);
          }
          acc
        });

    let mut dirty_packs = HashSet::default();
    let mut insert_items = HashSet::default();
    let mut remove_items = HashSet::default();

    let mut removed_files = vec![];

    // pour out items from non-full packs
    for (index, (pack_meta, _)) in indexed_packs.iter() {
      if (pack_meta.size as f64) < (options.pack_size as f64) * 0.8_f64 {
        dirty_packs.insert(*index);
      }
    }

    // get dirty packs and items for inserting/removing
    for (index, (key, val)) in indexed_updates.iter() {
      if val.is_some() {
        insert_items.insert(*index);
        if let Some(pack_index) = current_items_belong.get(key) {
          dirty_packs.insert(*pack_index);
        }
      } else {
        remove_items.insert(*index);
        if let Some(pack_index) = current_items_belong.get(key) {
          dirty_packs.insert(*pack_index);
        }
      }
    }

    // pour out items from dirty packs
    let mut items = dirty_packs
      .iter()
      .fold(HashMap::default(), |mut acc, pack_index| {
        let (_, old_pack) = indexed_packs
          .remove(pack_index)
          .expect("should have bucket pack");

        removed_files.push(old_pack.path.clone());

        let (PackKeysState::Value(keys), PackContentsState::Value(contents)) =
          (old_pack.keys, old_pack.contents)
        else {
          return acc;
        };
        if keys.len() != contents.len() {
          return acc;
        }

        acc.extend(keys.into_iter().zip(contents).collect::<HashMap<_, _>>());

        acc
      })
      .into_iter()
      .collect::<HashMap<_, _>>();

    // insert items
    items.extend(
      insert_items
        .iter()
        .map(|key| {
          let item = indexed_updates
            .remove(key)
            .expect("should have indexed update item");
          (
            Arc::new(item.0),
            Arc::new(item.1.expect("should have item value")),
          )
        })
        .collect::<HashMap<_, _>>(),
    );

    // remove items
    for key in remove_items.iter() {
      let (key, _) = indexed_updates
        .remove(key)
        .expect("should have indexed update item");
      let _ = items.remove(&key);
    }

    // generate packs
    let remain_packs = indexed_packs.into_values().collect_vec();
    let new_packs: Vec<(PackFileMeta, Pack)> = create(&pack_dir, options, items);

    UpdatePacksResult {
      new_packs,
      remain_packs,
      removed_files,
    }
  }

  async fn write_pack(&self, pack: &Pack) -> Result<()> {
    let path = self.get_temp_path(&pack.path)?;
    let keys = pack.keys.expect_value();
    let contents = pack.contents.expect_value();
    if keys.len() != contents.len() {
      return Err(error!("pack keys and contents length not match"));
    }

    let mut writer = self.fs.write_file(&path).await?;

    // key meta line
    writer
      .line(
        keys
          .iter()
          .map(|key| key.len().to_string())
          .collect::<Vec<_>>()
          .join(" ")
          .as_str(),
      )
      .await?;

    // content meta line
    writer
      .line(
        contents
          .iter()
          .map(|content| content.len().to_string())
          .collect::<Vec<_>>()
          .join(" ")
          .as_str(),
      )
      .await?;

    for key in keys {
      writer.bytes(key).await?;
    }

    for content in contents {
      writer.bytes(content).await?;
    }

    writer.flush().await?;

    Ok(())
  }
}

fn create(
  dir: &Utf8Path,
  options: &PackOptions,
  items: HashMap<Arc<StorageItemKey>, Arc<StorageItemValue>>,
) -> Vec<(PackFileMeta, Pack)> {
  let mut items = items.into_iter().collect_vec();
  items.sort_unstable_by(|a, b| a.1.len().cmp(&b.1.len()));

  let mut new_packs = vec![];

  fn create_pack(dir: &Utf8Path, keys: PackKeys, contents: PackContents) -> (PackFileMeta, Pack) {
    let file_name = get_name(&keys, &contents);
    let mut new_pack = Pack::new(dir.join(&file_name));
    new_pack.keys = PackKeysState::Value(keys);
    new_pack.contents = PackContentsState::Value(contents);
    (
      PackFileMeta {
        name: file_name,
        hash: Default::default(),
        size: new_pack.size(),
        wrote: false,
      },
      new_pack,
    )
  }

  loop {
    if items.is_empty() {
      break;
    }
    let last_item = items.last().expect("should have first item");
    // handle big single item pack
    if last_item.0.len() as f64 + last_item.1.len() as f64 > options.pack_size as f64 * 0.8_f64 {
      let (key, value) = items.pop().expect("should have first item");
      new_packs.push(create_pack(dir, vec![key], vec![value]));
    } else {
      break;
    }
  }

  items.reverse();

  loop {
    let mut batch_keys: PackKeys = vec![];
    let mut batch_contents: PackContents = vec![];
    let mut batch_size = 0_usize;

    loop {
      if items.is_empty() {
        break;
      }

      let last_item = items.last().expect("should have first item");

      if batch_size + last_item.0.len() + last_item.1.len() > options.pack_size {
        break;
      }

      let (key, value) = items.pop().expect("should have first item");
      batch_size += value.len() + key.len();
      batch_keys.push(key);
      batch_contents.push(value);
    }

    if !batch_keys.is_empty() {
      new_packs.push(create_pack(dir, batch_keys, batch_contents));
    }

    if items.is_empty() {
      break;
    }
  }

  new_packs
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use itertools::Itertools;
  use rspack_error::Result;
  use rspack_paths::Utf8PathBuf;
  use rustc_hash::FxHashMap as HashMap;

  use crate::{
    pack::{
      strategy::split::util::test_pack_utils::{mock_updates, UpdateVal},
      Pack, PackContentsState, PackFileMeta, PackKeysState, PackWriteStrategy, SplitPackStrategy,
      UpdatePacksResult,
    },
    PackFs, PackMemoryFs, PackOptions,
  };

  async fn test_write_pack(strategy: &SplitPackStrategy) -> Result<()> {
    let mut pack = Pack::new(Utf8PathBuf::from("/cache/test_write_pack/pack"));
    pack.keys = PackKeysState::Value(vec![
      Arc::new("key_1".as_bytes().to_vec()),
      Arc::new("key_2".as_bytes().to_vec()),
    ]);
    pack.contents = PackContentsState::Value(vec![
      Arc::new("val_1".as_bytes().to_vec()),
      Arc::new("val_2".as_bytes().to_vec()),
    ]);
    strategy.write_pack(&pack).await?;

    let mut reader = strategy
      .fs
      .read_file(
        &strategy
          .get_temp_path(&pack.path)
          .expect("should get temp path"),
      )
      .await?;
    assert_eq!(reader.line().await?, "5 5");
    assert_eq!(reader.line().await?, "5 5");
    assert_eq!(reader.bytes(5).await?, "key_1".as_bytes());
    assert_eq!(reader.bytes(5).await?, "key_2".as_bytes());
    assert_eq!(reader.bytes(5).await?, "val_1".as_bytes());
    assert_eq!(reader.bytes(5).await?, "val_2".as_bytes());
    Ok(())
  }

  fn update_packs(update_res: UpdatePacksResult) -> HashMap<PackFileMeta, Pack> {
    update_res
      .remain_packs
      .into_iter()
      .chain(update_res.new_packs)
      .collect::<HashMap<PackFileMeta, Pack>>()
  }

  fn get_pack_sizes(update_res: &UpdatePacksResult) -> Vec<usize> {
    let mut sizes = update_res
      .remain_packs
      .iter()
      .map(|(_, pack)| pack.size())
      .chain(update_res.new_packs.iter().map(|(_, pack)| pack.size()))
      .collect_vec();
    sizes.sort_unstable();
    sizes
  }

  async fn test_update_packs(strategy: &SplitPackStrategy) -> Result<()> {
    let dir = Utf8PathBuf::from("/cache/test_update_packs");
    let options = PackOptions {
      bucket_size: 1,
      pack_size: 2000,
      expire: 100000,
    };

    // half pack
    let mut packs = HashMap::default();
    let res = strategy.update_packs(
      dir.clone(),
      &options,
      packs,
      mock_updates(0, 50, 10, UpdateVal::Value("val".into())),
    );
    assert_eq!(res.new_packs.len(), 1);
    assert_eq!(res.remain_packs.len(), 0);
    assert_eq!(get_pack_sizes(&res), vec![1000]);

    packs = update_packs(res);

    // full pack
    let res = strategy.update_packs(
      dir.clone(),
      &options,
      packs,
      mock_updates(50, 100, 10, UpdateVal::Value("val".into())),
    );
    assert_eq!(res.new_packs.len(), 1);
    assert_eq!(res.remain_packs.len(), 0);
    assert_eq!(res.removed_files.len(), 1);
    assert_eq!(get_pack_sizes(&res), vec![2000]);

    packs = update_packs(res);

    // almost full pack
    let res = strategy.update_packs(
      dir.clone(),
      &options,
      packs,
      mock_updates(100, 190, 10, UpdateVal::Value("val".into())),
    );
    assert_eq!(res.new_packs.len(), 1);
    assert_eq!(res.remain_packs.len(), 1);
    assert_eq!(res.removed_files.len(), 0);
    assert_eq!(get_pack_sizes(&res), vec![1800, 2000]);

    packs = update_packs(res);

    let res = strategy.update_packs(
      dir.clone(),
      &options,
      packs,
      mock_updates(190, 200, 10, UpdateVal::Value("val".into())),
    );
    assert_eq!(res.new_packs.len(), 1);
    assert_eq!(res.remain_packs.len(), 2);
    assert_eq!(res.removed_files.len(), 0);
    assert_eq!(get_pack_sizes(&res), vec![200, 1800, 2000]);

    packs = update_packs(res);

    // long item pack
    let mut updates = mock_updates(0, 1, 1200, UpdateVal::Value("val".into()));
    updates.extend(mock_updates(1, 2, 900, UpdateVal::Value("val".into())));
    let res = strategy.update_packs(dir.clone(), &options, packs, updates);
    assert_eq!(res.new_packs.len(), 3);
    assert_eq!(res.remain_packs.len(), 2);
    assert_eq!(res.removed_files.len(), 1);
    assert_eq!(get_pack_sizes(&res), vec![200, 1800, 1800, 2000, 2400]);

    packs = update_packs(res);

    // remove items pack
    let res = strategy.update_packs(
      dir.clone(),
      &options,
      packs,
      mock_updates(100, 130, 10, UpdateVal::Removed),
    );
    assert_eq!(res.new_packs.len(), 1);
    assert_eq!(res.remain_packs.len(), 3);
    assert_eq!(res.removed_files.len(), 2);
    assert_eq!(get_pack_sizes(&res), vec![1400, 1800, 2000, 2400]);

    packs = update_packs(res);

    // update items pack
    let mut updates = HashMap::default();
    updates.insert(
      format!("{:0>6}_key", 131).as_bytes().to_vec(),
      Some(format!("{:0>6}_valaaa", 131).as_bytes().to_vec()),
    );
    let res = strategy.update_packs(dir.clone(), &options, packs, updates);
    assert_eq!(res.new_packs.len(), 1);
    assert_eq!(res.remain_packs.len(), 3);
    assert_eq!(res.removed_files.len(), 1);
    assert_eq!(get_pack_sizes(&res), vec![1403, 1800, 2000, 2400]);

    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn should_write_pack() {
    let fs = Arc::new(PackMemoryFs::default());
    fs.remove_dir(&Utf8PathBuf::from("/cache/test_write_pack"))
      .await
      .expect("should clean dir");
    let strategy = SplitPackStrategy::new(
      Utf8PathBuf::from("/cache/test_write_pack"),
      Utf8PathBuf::from("/temp/test_write_pack"),
      fs.clone(),
    );

    let _ = test_write_pack(&strategy)
      .await
      .map_err(|e| panic!("{}", e));
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn should_update_packs() {
    let fs = Arc::new(PackMemoryFs::default());
    fs.remove_dir(&Utf8PathBuf::from("/cache/test_update_packs"))
      .await
      .expect("should clean dir");
    let strategy = SplitPackStrategy::new(
      Utf8PathBuf::from("/cache/test_update_packs"),
      Utf8PathBuf::from("/temp/test_update_packs"),
      fs.clone(),
    );

    let _ = test_update_packs(&strategy)
      .await
      .map_err(|e| panic!("{}", e));
  }
}
