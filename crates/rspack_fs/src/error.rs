pub type Error = std::io::Error;

pub type Result<T> = std::result::Result<T, Error>;

pub trait RspackResultToFsResultExt<T> {
  fn to_fs_result(self) -> Result<T>;
}

impl<T> RspackResultToFsResultExt<T> for rspack_error::Result<T> {
  fn to_fs_result(self) -> Result<T> {
    self.map_err(|e| Error::other(e.to_string()))
  }
}
