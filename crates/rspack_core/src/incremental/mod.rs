mod mutations;

use std::{
  fmt,
  sync::{
    Mutex, MutexGuard,
    atomic::{AtomicU16, Ordering},
  },
};

use bitflags::bitflags;
pub use mutations::{Mutation, Mutations};
use rspack_error::{Diagnostic, Error};

pub const TRACING_TARGET: &str = "rspack_incremental";

bitflags! {
  #[derive(Debug, Clone, Copy, Eq, PartialEq)]
  pub struct IncrementalPasses: u16 {
    /// Webpack stage: compiler.hooks.make (build module graph)
    /// https://webpack.js.org/api/compiler-hooks/#make
    const BUILD_MODULE_GRAPH = 1 << 0;
    /// Webpack stage: compilation.hooks.finishModules
    /// https://webpack.js.org/api/compilation-hooks/#finishmodules
    const FINISH_MODULES = 1 << 1;
    /// Webpack stage: compilation.hooks.optimizeDependencies
    /// https://webpack.js.org/api/compilation-hooks/#optimizedependencies
    const OPTIMIZE_DEPENDENCIES = 1 << 2;
    /// Webpack stage: compilation.hooks.seal
    /// https://webpack.js.org/api/compilation-hooks/#seal
    const BUILD_CHUNK_GRAPH = 1 << 3;
    /// Webpack stage: compilation.hooks.optimizeChunkModules
    /// https://webpack.js.org/api/compilation-hooks/#optimizechunkmodules
    const OPTIMIZE_CHUNK_MODULES = 1 << 4;
    /// Webpack stage: compilation.hooks.moduleIds
    /// https://webpack.js.org/api/compilation-hooks/#moduleids
    const MODULE_IDS = 1 << 5;
    /// Webpack stage: compilation.hooks.chunkIds
    /// https://webpack.js.org/api/compilation-hooks/#chunkids
    const CHUNK_IDS = 1 << 6;
    /// Webpack stage: compilation.hooks.moduleHash beforeModuleHash / afterModuleHash
    /// https://webpack.js.org/api/compilation-hooks/#beforemodulehash
    /// https://webpack.js.org/api/compilation-hooks/#aftermodulehash
    const MODULES_HASHES = 1 << 7;
    /// Webpack stage: compilation.hooks.codeGeneration / afterCodeGeneration
    /// https://github.com/webpack/webpack/blob/d2a124db548cad6e84dffd93b502a4e74bfe2b6a/lib/Compilation.js#L902
    const MODULES_CODEGEN = 1 << 8;
    /// Webpack stage: compilation.hooks.beforeRuntimeRequirement / afterRuntimeRequirement
    /// https://github.com/webpack/webpack/blob/d2a124db548cad6e84dffd93b502a4e74bfe2b6a/lib/Compilation.js#L907
    const MODULES_RUNTIME_REQUIREMENTS = 1 << 9;
    const CHUNKS_RUNTIME_REQUIREMENTS = 1 << 10;
    /// Webpack stage: compilation.hooks.chunkHash / contentHash
    /// https://webpack.js.org/api/compilation-hooks/#contenthash
    const CHUNKS_HASHES = 1 << 11;
    /// Webpack stage: compilation.hooks.chunkAsset
    /// https://webpack.js.org/api/compilation-hooks/#chunkasset
    const CHUNK_ASSET = 1 << 12;
    /// Webpack stage: compiler.hooks.emit / afterEmit / assetEmitted
    /// https://webpack.js.org/api/compiler-hooks/#emit
    /// https://webpack.js.org/api/compiler-hooks/#afteremit
    /// https://webpack.js.org/api/compiler-hooks/#assetemitted
    const EMIT_ASSETS = 1 << 13;
  }
}

impl IncrementalPasses {
  pub fn pass_name(&self) -> &str {
    match *self {
      Self::BUILD_MODULE_GRAPH => "buildModuleGraph",
      Self::FINISH_MODULES => "finishModules",
      Self::OPTIMIZE_DEPENDENCIES => "optimizeDependencies",
      Self::BUILD_CHUNK_GRAPH => "buildChunkGraph",
      Self::OPTIMIZE_CHUNK_MODULES => "optimizeChunkModules",
      Self::MODULE_IDS => "moduleIds",
      Self::CHUNK_IDS => "chunkIds",
      Self::MODULES_HASHES => "modulesHashes",
      Self::MODULES_CODEGEN => "modulesCodegen",
      Self::MODULES_RUNTIME_REQUIREMENTS => "modulesRuntimeRequirements",
      Self::CHUNKS_RUNTIME_REQUIREMENTS => "chunksRuntimeRequirements",
      Self::CHUNKS_HASHES => "chunksHashes",
      Self::CHUNK_ASSET => "chunkAsset",
      Self::EMIT_ASSETS => "emitAssets",
      _ => unreachable!(),
    }
  }
}

impl fmt::Display for IncrementalPasses {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut first = true;
    for pass in IncrementalPasses::all().iter() {
      if self.contains(pass) {
        if !first {
          write!(f, ", ")?;
        }
        first = false;
        write!(f, "incremental.{}", pass.pass_name())?;
      }
    }
    Ok(())
  }
}

impl IncrementalPasses {
  pub fn allow_write(&self) -> bool {
    !self.is_empty()
  }

  pub fn allow_read(&self, pass: IncrementalPasses) -> bool {
    self.contains(pass)
  }
}

#[derive(Debug, Clone, Copy)]
pub struct IncrementalOptions {
  pub silent: bool,
  pub passes: IncrementalPasses,
}

impl IncrementalOptions {
  pub fn empty_passes() -> Self {
    Self {
      silent: true,
      passes: IncrementalPasses::empty(),
    }
  }

  pub fn advanced_silent() -> Self {
    Self {
      silent: true,
      passes: IncrementalPasses::all() - IncrementalPasses::BUILD_CHUNK_GRAPH,
    }
  }
}

enum IncrementalState {
  /// For cold build and cold start
  Cold,
  /// For hot build, hot start, and rebuild
  Hot { mutations: Mutex<Mutations> },
}

impl fmt::Debug for IncrementalState {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Cold => write!(f, "Cold"),
      Self::Hot { mutations } => {
        let mutations = mutations
          .lock()
          .expect("Mutex poisoned: failed to acquire lock on incremental mutations for debug");
        f.debug_struct("Hot")
          .field("mutations", &*mutations)
          .finish()
      }
    }
  }
}

pub struct Incremental {
  silent: bool,
  passes: AtomicU16,
  state: IncrementalState,
}

impl fmt::Debug for Incremental {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Incremental")
      .field("silent", &self.silent)
      .field("passes", &self.passes())
      .field("state", &self.state)
      .finish()
  }
}

impl Incremental {
  pub fn new_cold(options: IncrementalOptions) -> Self {
    Self {
      silent: options.silent,
      passes: AtomicU16::new(options.passes.bits()),
      state: IncrementalState::Cold,
    }
  }

  pub fn new_hot(options: IncrementalOptions) -> Self {
    Self {
      silent: options.silent,
      passes: AtomicU16::new(options.passes.bits()),
      state: IncrementalState::Hot {
        mutations: Mutex::new(Mutations::default()),
      },
    }
  }

  /// Get the current passes value
  fn passes(&self) -> IncrementalPasses {
    IncrementalPasses::from_bits_retain(self.passes.load(Ordering::SeqCst))
  }

  pub fn disable_passes(
    &self,
    passes: IncrementalPasses,
    thing: &'static str,
    reason: &'static str,
  ) -> Option<Option<Diagnostic>> {
    if matches!(self.state, IncrementalState::Hot { .. }) {
      let current = IncrementalPasses::from_bits_retain(self.passes.load(Ordering::SeqCst));
      let passes_to_disable = current.intersection(passes);
      if !passes_to_disable.is_empty() {
        // Atomically remove the passes using fetch_and with the negated bits
        self
          .passes
          .fetch_and(!passes_to_disable.bits(), Ordering::SeqCst);
        if self.silent {
          return Some(None);
        }
        return Some(Some(
          Error::from(NotFriendlyForIncremental {
            thing,
            reason,
            passes: passes_to_disable,
          })
          .into(),
        ));
      }
    }
    None
  }

  pub fn enabled(&self) -> bool {
    self.passes().allow_write()
  }

  pub fn passes_enabled(&self, passes: IncrementalPasses) -> bool {
    self.passes().allow_read(passes)
  }

  pub fn mutations_writeable(&self) -> bool {
    if matches!(self.state, IncrementalState::Hot { .. }) {
      return self.passes().allow_write();
    }
    false
  }

  pub fn mutations_readable(&self, passes: IncrementalPasses) -> bool {
    if matches!(self.state, IncrementalState::Hot { .. }) {
      return self.passes().allow_read(passes);
    }
    false
  }

  pub fn mutations_write(&self) -> Option<MutexGuard<'_, Mutations>> {
    if let IncrementalState::Hot { mutations } = &self.state
      && self.passes().allow_write()
    {
      return Some(
        mutations
          .lock()
          .expect("Mutex poisoned: failed to acquire write lock on incremental mutations"),
      );
    }
    None
  }

  pub fn mutations_read(&self, passes: IncrementalPasses) -> Option<MutexGuard<'_, Mutations>> {
    if let IncrementalState::Hot { mutations } = &self.state
      && self.passes().allow_read(passes)
    {
      return Some(
        mutations
          .lock()
          .expect("Mutex poisoned: failed to acquire read lock on incremental mutations"),
      );
    }
    None
  }
}

#[derive(Debug)]
pub struct NotFriendlyForIncremental {
  pub thing: &'static str,
  pub reason: &'static str,
  pub passes: IncrementalPasses,
}

impl From<NotFriendlyForIncremental> for rspack_error::Error {
  fn from(value: NotFriendlyForIncremental) -> rspack_error::Error {
    let mut error = rspack_error::Error::warning(format!(
      "{} is not friendly for incremental, {}. For this rebuild {} are fallback to non-incremental.",
      value.thing, value.reason, value.passes
    ));
    error.code = Some("NotFriendlyForIncremental".into());
    error
  }
}
