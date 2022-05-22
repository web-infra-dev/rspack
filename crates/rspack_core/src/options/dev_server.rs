#[derive(Debug)]
pub struct DevServerOptions {
  pub hmr: bool,
}

impl Default for DevServerOptions {
  fn default() -> Self {
    Self { hmr: true }
  }
}
