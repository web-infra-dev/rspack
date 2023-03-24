#[derive(Debug, Clone, Default)]
pub struct MemoryCacheOptions {
  /// Define the lifespan of unused cache entries in the memory cache.
  pub max_generations: u32,
}

#[derive(Debug, Clone, Default)]
pub struct FileSystemCacheOptions {
  // Collect unused memory allocated during deserialization
  // allow_collecting_memory: bool,
  // Define the lifespan of unused cache entries in the memory cache.
  // max_memory_generations: u32,
  /// The amount of time in milliseconds that unused cache entries are allowed to stay in the filesystem cache; defaults to one month.
  pub max_age: u32,
  /// Track and log detailed timing information for individual cache items of type 'filesystem'.
  pub profile: bool,
  /// An object of arrays of additional code dependencies for the build.
  pub build_dependencies: Vec<String>,
  /// Base directory for the cache. Defaults to node_modules/.cache/rspack.
  pub cache_directory: String,
  /// Locations for the cache. Defaults to path.resolve(cache.cacheDirectory, cache.name)
  pub cache_location: String,
  // Algorithm used the hash generation.
  // hash_algorithm: "md4" | "md5" | ...,
  // Compression type used for the cache files.
  // compression: false | "gzip" | "brotli",
  // Dnotes the time period after which the cache storing should happen.
  // idle_timeout: number,
  // idle_timeout_for_initial_store: number,
  // idle_timeout_after_large_changes": number,
  /// Name for the cache
  pub name: String,
  // Store strategy
  // store: "pack",
  /// Version of the cache data.
  pub version: String,
}

#[derive(Debug, Default, Clone)]
pub enum CacheOptions {
  #[default]
  Disabled,
  Memory(MemoryCacheOptions),
  FileSystem(FileSystemCacheOptions),
}
