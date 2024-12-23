use std::sync::Arc;

use async_trait::async_trait;
use itertools::Itertools;
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use super::{handle_file::redirect_to_path, SplitPackStrategy};
use crate::{
  error::Result,
  pack::{
    data::{Pack, PackFileMeta, PackOptions},
    strategy::{split::util::get_name, PackWriteStrategy, UpdatePacksResult},
    ScopeUpdate,
  },
  ItemKey, ItemValue,
};

#[derive(Debug, Eq)]
struct PackItemCandidate {
  key: Arc<ItemKey>,
  value: Arc<ItemValue>,
  generation: usize,
}

impl std::hash::Hash for PackItemCandidate {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.key.hash(state);
  }
}

impl PartialEq for PackItemCandidate {
  fn eq(&self, other: &Self) -> bool {
    self.key == other.key
  }
}

#[async_trait]
impl PackWriteStrategy for SplitPackStrategy {
  fn update_packs(
    &self,
    dir: Utf8PathBuf,
    generation: usize,
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
          let Some(keys) = pack.keys.get_value() else {
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

    // insert items
    let mut items = insert_items
      .iter()
      .map(|key| {
        let item = indexed_updates
          .remove(key)
          .expect("should have indexed update item");
        PackItemCandidate {
          key: Arc::new(item.0),
          value: Arc::new(item.1.expect("should have item value")),
          generation,
        }
      })
      .collect::<HashSet<_>>();

    // pour out items from dirty packs
    items.extend(
      dirty_packs
        .iter()
        .fold(HashSet::default(), |mut acc, pack_index| {
          let (_, mut old_pack) = indexed_packs
            .remove(pack_index)
            .expect("should have bucket pack");

          removed_files.push(old_pack.path.clone());

          let (Some(keys), Some(contents), generations) = (
            old_pack.keys.take_value(),
            old_pack.contents.take_value(),
            std::mem::take(&mut old_pack.generations),
          ) else {
            return acc;
          };

          if keys.len() != contents.len() || keys.len() != generations.len() {
            return acc;
          }

          acc.extend(
            keys
              .into_iter()
              .zip(contents.into_iter().zip(generations))
              .map(|(key, (value, generation))| PackItemCandidate {
                key,
                value,
                generation,
              })
              .collect::<HashSet<_>>(),
          );

          acc
        }),
    );

    // remove items
    let mut removed_keys = HashSet::default();
    for key in remove_items.iter() {
      let (key, _) = indexed_updates
        .remove(key)
        .expect("should have indexed update item");
      removed_keys.insert(key);
    }
    items.retain(|candidate| !removed_keys.contains(candidate.key.as_ref()));

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
    let path = redirect_to_path(&pack.path, &self.root, &self.temp_root)?;
    let keys = pack.keys.expect_value();
    let contents = pack.contents.expect_value();
    let generations = &pack.generations;
    if keys.len() != contents.len() {
      panic!("pack keys and contents length not match");
    }

    let mut writer = self.fs.write_file(&path).await?;

    // key meta line
    writer
      .write_line(
        keys
          .iter()
          .map(|key| key.len().to_string())
          .join(" ")
          .as_str(),
      )
      .await?;

    // content meta line
    writer
      .write_line(
        contents
          .iter()
          .map(|content| content.len().to_string())
          .join(" ")
          .as_str(),
      )
      .await?;

    // content generation line
    writer
      .write_line(generations.iter().map(|g| g.to_string()).join(" ").as_str())
      .await?;

    // keys blob
    for key in keys {
      writer.write(key).await?;
    }

    // contents blob
    for content in contents {
      writer.write(content).await?;
    }

    writer.flush().await?;

    Ok(())
  }
}

fn create(
  dir: &Utf8Path,
  options: &PackOptions,
  candidates: HashSet<PackItemCandidate>,
) -> Vec<(PackFileMeta, Pack)> {
  let candidate_sizes = candidates
    .iter()
    .map(|item| {
      let key_size = item.key.len();
      let value_size = item.value.len();
      let generation_size = item.generation.to_string().len();
      key_size + value_size + generation_size
    })
    .collect_vec();
  let (big_items, mut items): (Vec<_>, Vec<_>) = candidates
    .into_iter()
    .zip(candidate_sizes)
    .partition(|(_, size)| *size as f64 > options.pack_size as f64 * 0.8_f64);

  let mut new_packs = vec![];

  for item in big_items {
    new_packs.push(create_pack(dir, vec![item.0]));
  }

  items.sort_unstable_by(|(a, _), (b, _)| b.generation.cmp(&a.generation));

  loop {
    let mut batch_items = vec![];
    let mut batch_size = 0_usize;

    loop {
      if items.is_empty() {
        break;
      }

      if batch_size + items.last().expect("should have first item").1 > options.pack_size {
        break;
      }

      let (last_item, last_item_size) = items.pop().expect("should have first item");
      batch_size += last_item_size;
      batch_items.push(last_item);
    }

    if !batch_items.is_empty() {
      new_packs.push(create_pack(dir, batch_items));
    }

    if items.is_empty() {
      break;
    }
  }

  new_packs
}

fn create_pack(dir: &Utf8Path, candiates: Vec<PackItemCandidate>) -> (PackFileMeta, Pack) {
  let mut keys = vec![];
  let mut contents = vec![];
  let mut generations = vec![];
  for candidate in candiates {
    keys.push(candidate.key);
    contents.push(candidate.value);
    generations.push(candidate.generation);
  }

  let file_name = get_name(&keys, &contents);
  let mut new_pack = Pack::new(dir.join(&file_name));
  let latest_generation = *generations
    .iter()
    .max()
    .expect("should have latest generation");
  new_pack.keys.set_value(keys);
  new_pack.contents.set_value(contents);
  new_pack.generations = generations;
  (
    PackFileMeta {
      name: file_name,
      hash: Default::default(),
      size: new_pack.size(),
      wrote: false,
      generation: latest_generation,
    },
    new_pack,
  )
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use itertools::Itertools;
  use rustc_hash::FxHashMap as HashMap;

  use crate::{
    error::Result,
    pack::{
      data::{Pack, PackFileMeta, PackOptions},
      strategy::{
        split::{
          handle_file::redirect_to_path,
          util::test_pack_utils::{clean_strategy, create_strategies, mock_updates, UpdateVal},
        },
        PackWriteStrategy, SplitPackStrategy, UpdatePacksResult,
      },
    },
  };

  async fn test_write_pack(strategy: &SplitPackStrategy) -> Result<()> {
    let dir = strategy.root.join("write");
    let mut pack = Pack::new(dir);
    pack.keys.set_value(vec![
      Arc::new("key_1".as_bytes().to_vec()),
      Arc::new("key_2".as_bytes().to_vec()),
    ]);
    pack.contents.set_value(vec![
      Arc::new("val_1".as_bytes().to_vec()),
      Arc::new("val_2".as_bytes().to_vec()),
    ]);
    pack.generations = vec![1_usize, 2_usize];
    strategy.write_pack(&pack).await?;

    let mut reader = strategy
      .fs
      .read_file(&redirect_to_path(
        &pack.path,
        &strategy.root,
        &strategy.temp_root,
      )?)
      .await?;
    assert_eq!(reader.read_line().await?, "5 5");
    assert_eq!(reader.read_line().await?, "5 5");
    assert_eq!(reader.read_line().await?, "1 2");
    assert_eq!(reader.read(5).await?, "key_1".as_bytes());
    assert_eq!(reader.read(5).await?, "key_2".as_bytes());
    assert_eq!(reader.read(5).await?, "val_1".as_bytes());
    assert_eq!(reader.read(5).await?, "val_2".as_bytes());
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
    let dir = strategy.root.join("update");
    let options = PackOptions {
      bucket_size: 1,
      pack_size: 2100,
    };

    // half pack
    let mut packs = HashMap::default();
    let res = strategy.update_packs(
      dir.clone(),
      0_usize,
      &options,
      packs,
      mock_updates(0, 50, 10, UpdateVal::Value("val".into())),
    );
    assert_eq!(res.new_packs.len(), 1);
    assert_eq!(res.remain_packs.len(), 0);
    assert_eq!(get_pack_sizes(&res), vec![1050]);

    packs = update_packs(res);

    // full pack
    let res = strategy.update_packs(
      dir.clone(),
      1_usize,
      &options,
      packs,
      mock_updates(50, 100, 10, UpdateVal::Value("val".into())),
    );
    assert_eq!(res.new_packs.len(), 1);
    assert_eq!(res.remain_packs.len(), 0);
    assert_eq!(res.removed_files.len(), 1);
    assert_eq!(get_pack_sizes(&res), vec![2100]);

    packs = update_packs(res);

    // almost full pack
    let res = strategy.update_packs(
      dir.clone(),
      2_usize,
      &options,
      packs,
      mock_updates(100, 190, 10, UpdateVal::Value("val".into())),
    );
    assert_eq!(res.new_packs.len(), 1);
    assert_eq!(res.remain_packs.len(), 1);
    assert_eq!(res.removed_files.len(), 0);
    assert_eq!(get_pack_sizes(&res), vec![1890, 2100]);

    packs = update_packs(res);

    let res = strategy.update_packs(
      dir.clone(),
      3_usize,
      &options,
      packs,
      mock_updates(190, 200, 10, UpdateVal::Value("val".into())),
    );
    assert_eq!(res.new_packs.len(), 1);
    assert_eq!(res.remain_packs.len(), 2);
    assert_eq!(res.removed_files.len(), 0);
    assert_eq!(get_pack_sizes(&res), vec![210, 1890, 2100]);

    packs = update_packs(res);

    // long item pack
    let mut updates = mock_updates(0, 1, 1200, UpdateVal::Value("val".into()));
    updates.extend(mock_updates(1, 2, 900, UpdateVal::Value("val".into())));
    let res = strategy.update_packs(dir.clone(), 4_usize, &options, packs, updates);
    assert_eq!(res.new_packs.len(), 3);
    assert_eq!(res.remain_packs.len(), 2);
    assert_eq!(res.removed_files.len(), 1);
    assert_eq!(get_pack_sizes(&res), vec![210, 1801, 1890, 2100, 2401]);

    packs = update_packs(res);

    // remove items pack
    let res = strategy.update_packs(
      dir.clone(),
      5_usize,
      &options,
      packs,
      mock_updates(100, 130, 10, UpdateVal::Removed),
    );
    assert_eq!(res.new_packs.len(), 1);
    assert_eq!(res.remain_packs.len(), 3);
    assert_eq!(res.removed_files.len(), 2);
    assert_eq!(get_pack_sizes(&res), vec![1470, 1801, 2100, 2401]);

    packs = update_packs(res);

    // update items pack
    let mut updates = HashMap::default();
    updates.insert(
      format!("{:0>6}_key", 131).as_bytes().to_vec(),
      Some(format!("{:0>6}_valaaa", 131).as_bytes().to_vec()),
    );
    let res = strategy.update_packs(dir.clone(), 6_usize, &options, packs, updates);
    assert_eq!(res.new_packs.len(), 1);
    assert_eq!(res.remain_packs.len(), 3);
    assert_eq!(res.removed_files.len(), 1);
    assert_eq!(get_pack_sizes(&res), vec![1473, 1801, 2100, 2401]);

    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn should_write_pack() {
    for strategy in create_strategies("write_pack") {
      clean_strategy(&strategy).await;

      let _ = test_write_pack(&strategy)
        .await
        .map_err(|e| panic!("{}", e));
    }
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn should_update_packs() {
    for strategy in create_strategies("update_packs") {
      clean_strategy(&strategy).await;

      let _ = test_update_packs(&strategy)
        .await
        .map_err(|e| panic!("{}", e));
    }
  }
}
