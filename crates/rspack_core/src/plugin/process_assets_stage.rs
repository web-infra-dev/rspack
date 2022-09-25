#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProcessAssetStage {
  /**
   * Add additional assets to the compilation.
   */
  Additional = -2000,

  /**
   * Basic preprocessing of assets.
   */
  PreProcess = -1000,

  /**
   * Derive new assets from existing assets.
   * Existing assets should not be treated as complete.
   */
  Derived = -200,

  /**
   * Add additional sections to existing assets, like a banner or initialization code.
   */
  Additions = -100,

  /**
   * Optimize existing assets in a general way.
   */
  Optimize = 100,

  /**
   * Optimize the count of existing assets, e. g. by merging them.
   * Only assets of the same type should be merged.
   * For assets of different types see PROCESS_ASSETS_STAGE_OPTIMIZE_INLINE.
   */
  OptimizeCount = 200,

  /**
   * Optimize the compatibility of existing assets, e. g. add polyfills or vendor-prefixes.
   */
  OptimizeCompatibility = 300,

  /**
   * Optimize the size of existing assets, e. g. by minimizing or omitting whitespace.
   */
  OptimizeSize = 400,

  /**
   * Add development tooling to assets, e. g. by extracting a SourceMap.
   */
  DevTooling = 500,

  /**
   * Optimize the count of existing assets, e. g. by inlining assets of into other assets.
   * Only assets of different types should be inlined.
   * For assets of the same type see PROCESS_ASSETS_STAGE_OPTIMIZE_COUNT.
   */
  OptimizeInline = 700,

  /**
   * Summarize the list of existing assets
   * e. g. creating an assets manifest of Service Workers.
   */
  Summarize = 1000,

  /**
   * Optimize the hashes of the assets, e. g. by generating real hashes of the asset content.
   */
  OptimizeHash = 2500,

  /**
   * Optimize the transfer of existing assets, e. g. by preparing a compressed (gzip) file as separate asset.
   */
  OptimizeTransfer = 3000,

  /**
   * Analyse existing assets.
   */
  Analyse = 4000,

  /**
   * Creating assets for reporting purposes.
   */
  Report = 5000,

  /// If a plugin doesn't specify a stage, it will be put in this stage.
  Default = 0,
}
impl ProcessAssetStage {
  pub fn is_additional(&self) -> bool {
    matches!(self, Self::Additional)
  }
  pub fn is_pre_process(&self) -> bool {
    matches!(self, Self::PreProcess)
  }
  pub fn is_derived(&self) -> bool {
    matches!(self, Self::Derived)
  }
  pub fn is_additions(&self) -> bool {
    matches!(self, Self::Additions)
  }
  pub fn is_optimize(&self) -> bool {
    matches!(self, Self::Optimize)
  }
  pub fn is_optimize_count(&self) -> bool {
    matches!(self, Self::OptimizeCount)
  }
  pub fn is_optimize_compatibility(&self) -> bool {
    matches!(self, Self::OptimizeCompatibility)
  }
  pub fn is_optimize_size(&self) -> bool {
    matches!(self, Self::OptimizeSize)
  }
  pub fn is_dev_tooling(&self) -> bool {
    matches!(self, Self::DevTooling)
  }
  pub fn is_optimize_inline(&self) -> bool {
    matches!(self, Self::OptimizeInline)
  }
  pub fn is_summarize(&self) -> bool {
    matches!(self, Self::Summarize)
  }
  pub fn is_optimize_hash(&self) -> bool {
    matches!(self, Self::OptimizeHash)
  }
  pub fn is_optimize_transfer(&self) -> bool {
    matches!(self, Self::OptimizeTransfer)
  }
  pub fn is_analyse(&self) -> bool {
    matches!(self, Self::Analyse)
  }
  pub fn is_report(&self) -> bool {
    matches!(self, Self::Report)
  }
  pub fn is_default(&self) -> bool {
    matches!(self, Self::Default)
  }
}
