use rspack_paths::Utf8PathBuf;

#[derive(Debug)]
pub struct PackOptions {
  pub bucket_size: usize,
  pub pack_size: usize,
}

#[derive(Debug)]
pub struct RootOptions {
  pub root: Utf8PathBuf,
  pub expire: u64,
  pub clean: bool,
}
