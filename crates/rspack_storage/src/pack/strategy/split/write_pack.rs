use std::sync::Arc;

use async_trait::async_trait;
use futures::future::join_all;
use itertools::Itertools;
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use super::{SplitPackStrategy, handle_file::redirect_to_path};
use crate::{
  FSError, FSOperation, ItemKey, ItemValue,
  error::Result,
  pack::{
    ScopeUpdate,
    data::{Pack, PackFileMeta, PackOptions},
    strategy::{PackReadStrategy, PackWriteStrategy, UpdatePacksResult, split::util::get_name},
  },
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
  async fn optimize_packs(
    &self,
    dir: Utf8PathBuf,
    options: &PackOptions,
    packs: Vec<(PackFileMeta, Pack)>,
  ) -> Result<UpdatePacksResult> {
    let mut removed_files = vec![];
    let items = packs
      .into_iter()
      .fold(HashSet::default(), |mut acc, (_, mut pack)| {
        removed_files.push(pack.path.clone());
        pour_pack(&mut acc, &mut pack);
        acc
      });
    Ok(UpdatePacksResult {
      new_packs: create(&dir, options, items),
      remain_packs: vec![],
      removed_files,
    })
  }

  async fn update_packs(
    &self,
    dir: Utf8PathBuf,
    generation: usize,
    options: &PackOptions,
    packs: HashMap<PackFileMeta, Pack>,
    updates: ScopeUpdate,
  ) -> Result<UpdatePacksResult> {
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

    let mut dirty_pack_index = HashSet::default();
    let mut insert_items = HashSet::default();
    let mut remove_items = HashSet::default();

    let mut removed_files = vec![];

    // pour out items from non-full packs
    for (index, (pack_meta, _)) in indexed_packs.iter() {
      if (pack_meta.size as f64) < (options.pack_size as f64) * 0.8_f64 {
        dirty_pack_index.insert(*index);
      }
    }

    // get dirty packs and items for inserting/removing
    for (index, (key, val)) in indexed_updates.iter() {
      if val.is_some() {
        insert_items.insert(*index);
        if let Some(pack_index) = current_items_belong.get(key) {
          dirty_pack_index.insert(*pack_index);
        }
      } else {
        remove_items.insert(*index);
        if let Some(pack_index) = current_items_belong.get(key) {
          dirty_pack_index.insert(*pack_index);
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

    let dirty_packs = reload_released_packs(
      dirty_pack_index
        .into_iter()
        .map(|pack_index| {
          indexed_packs
            .remove(&pack_index)
            .expect("should have bucket pack")
            .1
        })
        .collect_vec(),
      self,
    )
    .await?;

    // pour out items from dirty packs
    items.extend(
      dirty_packs
        .into_iter()
        .fold(HashSet::default(), |mut acc, mut pack| {
          removed_files.push(pack.path.clone());
          pour_pack(&mut acc, &mut pack);
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
    let new_packs: Vec<(PackFileMeta, Pack)> = create(&dir, options, items);

    Ok(UpdatePacksResult {
      new_packs,
      remain_packs,
      removed_files,
    })
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

async fn reload_released_packs(
  packs: Vec<Pack>,
  strategy: &SplitPackStrategy,
) -> Result<Vec<Pack>> {
  let (released_packs, memory_packs): (Vec<_>, Vec<_>) = packs
    .into_iter()
    .partition(|pack| pack.contents.is_released());

  let mut res = join_all(released_packs.into_iter().map(|mut pack| {
    let strategy = strategy.to_owned();
    async move {
      match strategy.read_pack_contents(&pack.path).await {
        Ok(contents) => {
          if let Some(contents) = contents {
            pack.contents.set_value(contents.contents);
            pack.generations = contents.generations;
            Ok(pack)
          } else {
            Err(
              FSError::from_message(
                &pack.path,
                FSOperation::Read,
                format!("pack `{}` is released and deleted", pack.path),
              )
              .into(),
            )
          }
        }
        Err(e) => Err(e),
      }
    }
  }))
  .await
  .into_iter()
  .collect::<Result<Vec<_>>>()?;

  res.extend(memory_packs);

  Ok(res)
}

fn pour_pack(items: &mut HashSet<PackItemCandidate>, pack: &mut Pack) {
  let (Some(keys), Some(contents), generations) = (
    pack.keys.take_value(),
    pack.contents.take_value(),
    std::mem::take(&mut pack.generations),
  ) else {
    panic!("should have pack keys and contents");
  };

  if keys.len() != contents.len() || keys.len() != generations.len() {
    panic!("should have same length keys, contents, and generations");
  }

  items.extend(
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

fn create_pack(dir: &Utf8Path, candidates: Vec<PackItemCandidate>) -> (PackFileMeta, Pack) {
  let mut keys = vec![];
  let mut contents = vec![];
  let mut generations = vec![];
  for candidate in candidates {
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
        PackWriteStrategy, SplitPackStrategy, UpdatePacksResult,
        split::{
          handle_file::redirect_to_path,
          util::test_pack_utils::{UpdateVal, clean_strategy, create_strategies, mock_updates},
        },
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
    let res = strategy
      .update_packs(
        dir.clone(),
        0_usize,
        &options,
        packs,
        mock_updates(0, 50, 10, &UpdateVal::Value("val".into())),
      )
      .await?;
    assert_eq!(res.new_packs.len(), 1);
    assert_eq!(res.remain_packs.len(), 0);
    assert_eq!(get_pack_sizes(&res), vec![1050]);

    packs = update_packs(res);

    // full pack
    let res = strategy
      .update_packs(
        dir.clone(),
        1_usize,
        &options,
        packs,
        mock_updates(50, 100, 10, &UpdateVal::Value("val".into())),
      )
      .await?;
    assert_eq!(res.new_packs.len(), 1);
    assert_eq!(res.remain_packs.len(), 0);
    assert_eq!(res.removed_files.len(), 1);
    assert_eq!(get_pack_sizes(&res), vec![2100]);

    packs = update_packs(res);

    // almost full pack
    let res = strategy
      .update_packs(
        dir.clone(),
        2_usize,
        &options,
        packs,
        mock_updates(100, 190, 10, &UpdateVal::Value("val".into())),
      )
      .await?;
    assert_eq!(res.new_packs.len(), 1);
    assert_eq!(res.remain_packs.len(), 1);
    assert_eq!(res.removed_files.len(), 0);
    assert_eq!(get_pack_sizes(&res), vec![1890, 2100]);

    packs = update_packs(res);

    let res = strategy
      .update_packs(
        dir.clone(),
        3_usize,
        &options,
        packs,
        mock_updates(190, 200, 10, &UpdateVal::Value("val".into())),
      )
      .await?;
    assert_eq!(res.new_packs.len(), 1);
    assert_eq!(res.remain_packs.len(), 2);
    assert_eq!(res.removed_files.len(), 0);
    assert_eq!(get_pack_sizes(&res), vec![210, 1890, 2100]);

    packs = update_packs(res);

    // long item pack
    let mut updates = mock_updates(0, 1, 1200, &UpdateVal::Value("val".into()));
    updates.extend(mock_updates(1, 2, 900, &UpdateVal::Value("val".into())));
    let res = strategy
      .update_packs(dir.clone(), 4_usize, &options, packs, updates)
      .await?;
    assert_eq!(res.new_packs.len(), 3);
    assert_eq!(res.remain_packs.len(), 2);
    assert_eq!(res.removed_files.len(), 1);
    assert_eq!(get_pack_sizes(&res), vec![210, 1801, 1890, 2100, 2401]);

    packs = update_packs(res);

    // remove items pack
    let res = strategy
      .update_packs(
        dir.clone(),
        5_usize,
        &options,
        packs,
        mock_updates(100, 130, 10, &UpdateVal::Removed),
      )
      .await?;
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
    let res = strategy
      .update_packs(dir.clone(), 6_usize, &options, packs, updates)
      .await?;
    assert_eq!(res.new_packs.len(), 1);
    assert_eq!(res.remain_packs.len(), 3);
    assert_eq!(res.removed_files.len(), 1);
    assert_eq!(get_pack_sizes(&res), vec![1473, 1801, 2100, 2401]);

    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn should_write_pack() -> Result<()> {
    for strategy in create_strategies("write_pack") {
      clean_strategy(&strategy).await;

      test_write_pack(&strategy).await?;
    }
    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn should_update_packs() -> Result<()> {
    for strategy in create_strategies("update_packs") {
      clean_strategy(&strategy).await;

      test_update_packs(&strategy).await?;
    }
    Ok(())
  }
}
