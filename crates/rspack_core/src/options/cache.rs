#[derive(Debug, Default, Clone)]
pub enum CacheOptions {
  #[default]
  Disabled,
  Memory,
}
