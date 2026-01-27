#[derive(Debug, Default, Clone)]
pub enum CacheOptions {
  #[default]
  Disabled,
  Memory {
    /// The maximum number of generations to keep in memory.
    ///
    /// For example, if `max_generations` is set to 1,
    /// the cache will be removed if it's not accessed for 1 compilation generation.
    max_generations: Option<u32>,
  },
}
