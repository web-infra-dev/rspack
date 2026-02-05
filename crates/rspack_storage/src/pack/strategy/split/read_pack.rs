use std::sync::Arc;

use rspack_paths::Utf8Path;

use super::SplitPackStrategy;
use crate::{
  FSError, FSOperation,
  error::Result,
  pack::{data::PackKeys, strategy::PackMainContents},
};

impl SplitPackStrategy {
  pub async fn read_pack_keys(&self, path: &Utf8Path) -> Result<Option<PackKeys>> {
    if !self.fs.exists(path).await? {
      return Ok(None);
    }

    let mut reader = self.fs.read_file(path).await?;
    // read key lengths
    let key_lengths = reader
      .read_line()
      .await?
      .split(" ")
      .map(|item| {
        item.parse::<usize>().map_err(|e| {
          FSError::from_message(
            path,
            FSOperation::Read,
            format!("parse pack key lengths failed: {e}"),
          )
        })
      })
      .collect::<std::result::Result<Vec<_>, FSError>>()?;

    // skip content lengths
    reader.read_line().await?;
    // skip generations
    reader.read_line().await?;

    // read keys
    let mut keys = vec![];
    for key_len in key_lengths {
      keys.push(Arc::new(reader.read(key_len).await?));
    }
    Ok(Some(keys))
  }

  pub async fn read_pack_contents(&self, path: &Utf8Path) -> Result<Option<PackMainContents>> {
    if !self.fs.exists(path).await? {
      return Ok(None);
    }

    let mut reader = self.fs.read_file(path).await?;
    let total_key_length = reader
      .read_line()
      .await?
      .split(" ")
      .map(|item| {
        item.parse::<usize>().map_err(|e| {
          FSError::from_message(
            path,
            FSOperation::Read,
            format!("parse pack key lengths failed: {e}"),
          )
        })
      })
      .collect::<std::result::Result<Vec<_>, FSError>>()?
      .iter()
      .sum::<usize>();

    let content_lengths: Vec<usize> = reader
      .read_line()
      .await?
      .split(" ")
      .map(|item| {
        item.parse::<usize>().map_err(|e| {
          FSError::from_message(
            path,
            FSOperation::Read,
            format!("parse pack content lengths failed: {e}"),
          )
        })
      })
      .collect::<std::result::Result<Vec<_>, FSError>>()?;

    let generations: Vec<usize> = reader
      .read_line()
      .await?
      .split(" ")
      .map(|item| {
        item.parse::<usize>().map_err(|e| {
          FSError::from_message(
            path,
            FSOperation::Read,
            format!("parse pack generations failed: {e}"),
          )
        })
      })
      .collect::<std::result::Result<Vec<_>, FSError>>()?;

    reader.skip(total_key_length).await?;

    let mut contents = vec![];
    for len in content_lengths {
      contents.push(Arc::new(reader.read(len).await?));
    }

    Ok(Some(PackMainContents {
      contents,
      generations,
    }))
  }
}

#[cfg(test)]
mod tests {

  use rspack_paths::Utf8PathBuf;
  use rustc_hash::FxHashSet as HashSet;

  use crate::{
    error::Result,
    pack::strategy::{
      SplitPackStrategy,
      split::util::test_pack_utils::{clean_strategy, create_strategies, mock_pack_file},
    },
  };

  async fn test_read_keys_non_exists(strategy: &SplitPackStrategy) -> Result<()> {
    let non_exists_keys = strategy
      .read_pack_keys(&strategy.get_path("/non_exists_path"))
      .await?;
    assert!(non_exists_keys.is_none());
    Ok(())
  }

  async fn test_read_contents_non_exists(strategy: &SplitPackStrategy) -> Result<()> {
    let non_exists_contents = strategy
      .read_pack_contents(&strategy.get_path("/non_exists_path"))
      .await?;
    assert!(non_exists_contents.is_none());
    Ok(())
  }

  async fn test_read_keys(path: &Utf8PathBuf, strategy: &SplitPackStrategy) -> Result<()> {
    let keys = strategy
      .read_pack_keys(path)
      .await?
      .unwrap_or_default()
      .into_iter()
      .map(|x| x.as_ref().to_owned())
      .collect::<HashSet<_>>();
    assert!(keys.contains("key_mock_0".as_bytes()));
    assert!(keys.contains("key_mock_19".as_bytes()));
    Ok(())
  }

  async fn test_read_contents(path: &Utf8PathBuf, strategy: &SplitPackStrategy) -> Result<()> {
    let contents = strategy
      .read_pack_contents(path)
      .await?
      .unwrap_or_default()
      .contents
      .into_iter()
      .map(|x| x.as_ref().to_owned())
      .collect::<HashSet<_>>();
    assert!(contents.contains("val_mock_0".as_bytes()));
    assert!(contents.contains("val_mock_19".as_bytes()));
    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn should_read_pack() -> Result<()> {
    for strategy in create_strategies("read_pack") {
      clean_strategy(&strategy).await;
      let dir = strategy.get_path("pack");
      mock_pack_file(&dir.join("./mock_pack"), "mock", 20, strategy.fs.as_ref())
        .await
        .expect("should mock pack file");
      test_read_keys(&dir.join("./mock_pack"), &strategy).await?;
      test_read_contents(&dir.join("./mock_pack"), &strategy).await?;
      test_read_keys_non_exists(&strategy).await?;
      test_read_contents_non_exists(&strategy).await?;
    }
    Ok(())
  }
}
