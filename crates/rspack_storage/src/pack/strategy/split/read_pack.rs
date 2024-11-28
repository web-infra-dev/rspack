use std::sync::Arc;

use async_trait::async_trait;
use rspack_error::Result;
use rspack_paths::Utf8Path;

use super::SplitPackStrategy;
use crate::pack::{PackContents, PackKeys, PackReadStrategy};

#[async_trait]
impl PackReadStrategy for SplitPackStrategy {
  async fn read_pack_keys(&self, path: &Utf8Path) -> Result<Option<PackKeys>> {
    if !self.fs.exists(path).await? {
      return Ok(None);
    }

    let mut reader = self.fs.read_file(path).await?;
    let key_lengths: Vec<usize> = reader
      .line()
      .await?
      .split(" ")
      .map(|item| item.parse::<usize>().expect("should have meta info"))
      .collect();

    reader.line().await?;

    let mut keys = vec![];
    for key_len in key_lengths {
      keys.push(Arc::new(reader.bytes(key_len).await?));
    }
    Ok(Some(keys))
  }

  async fn read_pack_contents(&self, path: &Utf8Path) -> Result<Option<PackContents>> {
    if !self.fs.exists(path).await? {
      return Ok(None);
    }

    let mut reader = self.fs.read_file(path).await?;
    let total_key_length = reader
      .line()
      .await?
      .split(" ")
      .map(|item| item.parse::<usize>().expect("should have meta info"))
      .sum::<usize>();

    let content_lengths: Vec<usize> = reader
      .line()
      .await?
      .split(" ")
      .map(|item| item.parse::<usize>().expect("should have meta info"))
      .collect();

    reader.skip(total_key_length).await?;

    let mut res = vec![];
    for len in content_lengths {
      res.push(Arc::new(reader.bytes(len).await?));
    }

    Ok(Some(res))
  }
}

#[cfg(test)]
mod tests {

  use std::sync::Arc;

  use rspack_error::Result;
  use rspack_paths::Utf8PathBuf;
  use rustc_hash::FxHashSet as HashSet;

  use crate::{
    pack::{
      strategy::split::util::test_pack_utils::mock_pack_file, PackReadStrategy, SplitPackStrategy,
    },
    PackFs, PackMemoryFs,
  };

  async fn test_read_keys_non_exists(strategy: &SplitPackStrategy) -> Result<()> {
    let non_exists_keys = strategy
      .read_pack_keys(&Utf8PathBuf::from("/non_exists_path"))
      .await?;
    assert!(non_exists_keys.is_none());
    Ok(())
  }

  async fn test_read_contents_non_exists(strategy: &SplitPackStrategy) -> Result<()> {
    let non_exists_contents = strategy
      .read_pack_contents(&Utf8PathBuf::from("/non_exists_path"))
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
      .into_iter()
      .map(|x| x.as_ref().to_owned())
      .collect::<HashSet<_>>();
    assert!(contents.contains("val_mock_0".as_bytes()));
    assert!(contents.contains("val_mock_19".as_bytes()));
    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn should_read_pack() {
    let dir = Utf8PathBuf::from("/cache/test_read_pack");
    let fs = Arc::new(PackMemoryFs::default());
    fs.remove_dir(&dir).await.expect("should clean dir");
    let strategy = SplitPackStrategy::new(dir.clone(), Utf8PathBuf::from("/temp"), fs.clone());
    mock_pack_file(&dir.join("./mock_pack"), "mock", 20, fs)
      .await
      .expect("should mock pack file");
    let _ = test_read_keys(&dir.join("./mock_pack"), &strategy)
      .await
      .map_err(|e| panic!("{:?}", e));
    let _ = test_read_contents(&dir.join("./mock_pack"), &strategy)
      .await
      .map_err(|e| panic!("{:?}", e));
    let _ = test_read_keys_non_exists(&strategy)
      .await
      .map_err(|e| panic!("{:?}", e));
    let _ = test_read_contents_non_exists(&strategy)
      .await
      .map_err(|e| panic!("{:?}", e));
  }
}
