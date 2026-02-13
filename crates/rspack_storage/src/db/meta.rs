use rspack_paths::Utf8Path;

use crate::db::error::{DBError, DBResult};

/// Bucket metadata tracking pages
/// Format:
/// Line 1: cold page IDs (comma-separated)
/// Line 2: hot page ID (or empty if none)
#[derive(Debug, Clone, Default)]
pub struct BucketMeta {
  /// Cold page IDs
  pub pages: Vec<String>,
  /// Hot page ID (if any)
  pub hot: Option<String>,
}

impl BucketMeta {
  pub fn read(base_path: &Utf8Path) -> DBResult<Self> {
    let path = base_path.join("bucket_meta.txt");
    let content = std::fs::read_to_string(&path).map_err(|e| DBError::IO(e))?;

    let lines: Vec<&str> = content.lines().collect();
    if lines.len() < 2 {
      return Ok(Self::default());
    }

    let pages = if lines[0].is_empty() {
      vec![]
    } else {
      lines[0].split(',').map(|s| s.to_string()).collect()
    };

    let hot = if lines[1].is_empty() {
      None
    } else {
      Some(lines[1].to_string())
    };

    Ok(Self { pages, hot })
  }

  pub fn write(base_path: &Utf8Path, meta: &Self) -> DBResult<()> {
    let path = base_path.join("bucket_meta.txt");
    let content = Self::to_string(meta);

    std::fs::write(&path, content).map_err(|e| DBError::IO(e))?;

    Ok(())
  }

  pub fn write_to_bytes(meta: &Self) -> DBResult<Vec<u8>> {
    Ok(Self::to_string(meta).into_bytes())
  }

  fn to_string(meta: &Self) -> String {
    format!(
      "{}\n{}",
      meta.pages.join(","),
      meta.hot.as_deref().unwrap_or("")
    )
  }
}
