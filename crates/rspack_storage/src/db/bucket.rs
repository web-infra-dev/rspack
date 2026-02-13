use rspack_paths::{Utf8Path, Utf8PathBuf};
use rustc_hash::FxHashMap as HashMap;

use crate::{
  db::{
    error::{DBError, DBResult},
    meta::BucketMeta,
    options::DBOptions,
    page::Page,
  },
  fs::FileSystem,
};

pub struct SaveResult {
  /// Files to add (paths relative to DB root, e.g., "bucket1/0.hot.pack")
  pub files_to_add: Vec<(Utf8PathBuf, Vec<u8>)>,
  /// Files to remove (paths relative to DB root, e.g., "bucket1/1.cold.pack")
  pub files_to_remove: Vec<Utf8PathBuf>,
}

/// Bucket manages pages for a scope
#[derive(Debug)]
pub struct Bucket {
  /// Scope name
  scope: String,
  /// Base path for this bucket (absolute path)
  base_path: Utf8PathBuf,
  /// Root path for the DB
  root: Utf8PathBuf,
  /// Filesystem
  fs: FileSystem,
  /// Options
  options: DBOptions,
}

impl Bucket {
  pub fn new(scope: String, root: &Utf8Path, fs: FileSystem, options: DBOptions) -> Self {
    let base_path = root.join(&scope);

    Self {
      scope,
      base_path,
      root: root.to_path_buf(),
      fs,
      options,
    }
  }

  pub async fn load(&self) -> DBResult<HashMap<Vec<u8>, Vec<u8>>> {
    let meta = BucketMeta::read(&self.base_path).unwrap_or_default();
    let mut data = HashMap::default();

    for page_id in &meta.pages {
      let page = Page::new(&self.base_path, page_id.clone(), false);

      let pack_content = self.read_file(&page.pack_path).await?;
      let pack = page.read_pack(&pack_content)?;

      data.extend(pack.into_map());
    }

    if let Some(hot_id) = &meta.hot {
      let page = Page::new(&self.base_path, hot_id.clone(), true);

      let pack_content = self.read_file(&page.pack_path).await?;
      let pack = page.read_pack(&pack_content)?;

      data.extend(pack.into_map());
    }

    Ok(data)
  }

  pub async fn prepare_save(&mut self, data: HashMap<Vec<u8>, Vec<u8>>) -> DBResult<SaveResult> {
    let mut files_to_add = Vec::new();
    let mut files_to_remove = Vec::new();

    let mut meta = BucketMeta::read(&self.base_path).unwrap_or_default();

    if let Some(hot_id) = &meta.hot {
      let page = Page::new(&self.base_path, hot_id.clone(), true);
      let old_pack_content = self.read_file(&page.pack_path).await?;
      let old_pack = page.read_pack(&old_pack_content)?;

      // Merge old and new data
      let merged = old_pack.merge(data);

      if page.should_split(merged.len(), &self.options) {
        let result = self.split_hot_page_prepare(&mut meta, merged)?;
        files_to_add.extend(result.files_to_add);
        files_to_remove.extend(result.files_to_remove);
      } else {
        let (pack_buf, index_buf) = self.prepare_page_data(&page, merged)?;
        files_to_add.push((self.to_relative_path(&page.pack_path)?, pack_buf));
        files_to_add.push((self.to_relative_path(&page.index_path)?, index_buf));
      }
    } else {
      let new_hot_id = self.allocate_page_id(&meta);
      meta.hot = Some(new_hot_id.clone());

      let page = Page::new(&self.base_path, new_hot_id, true);
      let pack = crate::db::pack::Pack::new(data.into_iter().collect());
      let (pack_buf, index_buf) = self.prepare_page_data_from_pack(&page, pack)?;
      files_to_add.push((self.to_relative_path(&page.pack_path)?, pack_buf));
      files_to_add.push((self.to_relative_path(&page.index_path)?, index_buf));
    }

    let meta_buf = BucketMeta::write_to_bytes(&meta)?;
    let meta_path = self.base_path.join("bucket_meta.txt");
    files_to_add.push((self.to_relative_path(&meta_path)?, meta_buf));

    Ok(SaveResult {
      files_to_add,
      files_to_remove,
    })
  }

  fn split_hot_page_prepare(
    &self,
    meta: &mut BucketMeta,
    pack: crate::db::pack::Pack,
  ) -> DBResult<SaveResult> {
    let mut files_to_add = Vec::new();
    let mut files_to_remove = Vec::new();

    let hot_id = meta.hot.take().unwrap();
    meta.pages.push(hot_id.clone());

    let old_hot_page = Page::new(&self.base_path, hot_id.clone(), true);
    files_to_remove.push(self.to_relative_path(&old_hot_page.pack_path)?);
    files_to_remove.push(self.to_relative_path(&old_hot_page.index_path)?);

    let entries = pack.into_entries();
    let split_point = (entries.len() as f64 * 0.8) as usize;
    let (cold_entries, hot_entries) = entries.split_at(split_point);

    let cold_page = Page::new(&self.base_path, hot_id, false);
    let (cold_pack, cold_index) = cold_page.write_data(cold_entries)?;
    files_to_add.push((self.to_relative_path(&cold_page.pack_path)?, cold_pack));
    files_to_add.push((self.to_relative_path(&cold_page.index_path)?, cold_index));

    let new_hot_id = self.allocate_page_id(meta);
    meta.hot = Some(new_hot_id.clone());

    let new_hot_page = Page::new(&self.base_path, new_hot_id, true);
    let (hot_pack, hot_index) = new_hot_page.write_data(hot_entries)?;
    files_to_add.push((self.to_relative_path(&new_hot_page.pack_path)?, hot_pack));
    files_to_add.push((self.to_relative_path(&new_hot_page.index_path)?, hot_index));

    Ok(SaveResult {
      files_to_add,
      files_to_remove,
    })
  }

  fn prepare_page_data(
    &self,
    page: &Page,
    pack: crate::db::pack::Pack,
  ) -> DBResult<(Vec<u8>, Vec<u8>)> {
    let entries = pack.into_entries();
    page.write_data(&entries)
  }

  fn prepare_page_data_from_pack(
    &self,
    page: &Page,
    pack: crate::db::pack::Pack,
  ) -> DBResult<(Vec<u8>, Vec<u8>)> {
    let entries = pack.into_entries();
    page.write_data(&entries)
  }

  async fn read_file(&self, path: &Utf8Path) -> DBResult<Vec<u8>> {
    let mut reader = self.fs.read_file(path).await?;
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf).await?;
    Ok(buf)
  }

  fn allocate_page_id(&self, meta: &BucketMeta) -> String {
    let max_id = meta
      .pages
      .iter()
      .filter_map(|id| id.parse::<u32>().ok())
      .max()
      .unwrap_or(0);

    (max_id + 1).to_string()
  }

  /// Convert absolute path to relative path (from DB root)
  /// Example: "/root/bucket1/0.hot.pack" -> "bucket1/0.hot.pack"
  fn to_relative_path(&self, abs_path: &Utf8Path) -> DBResult<Utf8PathBuf> {
    abs_path
      .strip_prefix(&self.root)
      .map(|p| p.to_path_buf())
      .map_err(|_| {
        DBError::InvalidFormat(format!("Path {} is not under root {}", abs_path, self.root))
      })
  }
}
