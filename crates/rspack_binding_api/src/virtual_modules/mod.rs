use rspack_fs::{AbsPath, MemoryFileSystem};

#[napi(js_name = "VirtualFileStore")]
pub struct JsVirtualFileStore(MemoryFileSystem);

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct JsVirtualFile {
  pub path: String,
  pub content: String,
}

pub(crate) fn write_virtual_file(fs: &MemoryFileSystem, path: &str, content: &[u8]) {
  if let Ok(abs) = AbsPath::new(path) {
    if let Some(parent) = abs.parent()
      && fs.create_dir_all(&parent).is_err()
    {
      return;
    }
    let _ = fs.write(&abs, content);
  }
}

#[napi]
impl JsVirtualFileStore {
  pub fn new(store: MemoryFileSystem) -> Self {
    Self(store)
  }

  #[napi]
  pub fn write_virtual_file_sync(&self, path: String, content: String) {
    write_virtual_file(&self.0, &path, content.as_bytes());
  }

  #[napi]
  pub fn batch_write_virtual_files_sync(&self, files: Vec<JsVirtualFile>) {
    for f in files {
      write_virtual_file(&self.0, &f.path, f.content.as_bytes());
    }
  }
}

mod fs;
pub use fs::VirtualFileSystem;
