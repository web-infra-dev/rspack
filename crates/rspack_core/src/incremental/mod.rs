mod mutations;

use std::fmt;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::{Mutex, MutexGuard};

use bitflags::bitflags;
pub use mutations::{Mutation, Mutations};
use rspack_error::{Diagnostic, Error};

pub const TRACING_TARGET: &str = "rspack_incremental";

bitflags! {
  #[derive(Debug, Clone, Copy, Eq, PartialEq)]
  pub struct IncrementalPasses: u16 {
    const MAKE = 1 << 0;
    const INFER_ASYNC_MODULES = 1 << 1;
    const PROVIDED_EXPORTS = 1 << 2;
    const DEPENDENCIES_DIAGNOSTICS = 1 << 3;
    const SIDE_EFFECTS = 1 << 4;
    const BUILD_CHUNK_GRAPH = 1 << 5;
    const MODULE_IDS = 1 << 6;
    const CHUNK_IDS = 1 << 7;
    const MODULES_HASHES = 1 << 8;
    const MODULES_CODEGEN = 1 << 9;
    const MODULES_RUNTIME_REQUIREMENTS = 1 << 10;
    const CHUNKS_RUNTIME_REQUIREMENTS = 1 << 11;
    const CHUNKS_HASHES = 1 << 12;
    const CHUNKS_RENDER = 1 << 13;
    const EMIT_ASSETS = 1 << 14;
  }
}

impl IncrementalPasses {
  pub fn pass_name(&self) -> &str {
    match *self {
      Self::MAKE => "make",
      Self::INFER_ASYNC_MODULES => "inferAsyncModules",
      Self::PROVIDED_EXPORTS => "providedExports",
      Self::DEPENDENCIES_DIAGNOSTICS => "dependenciesDiagnostics",
      Self::SIDE_EFFECTS => "sideEffects",
      Self::BUILD_CHUNK_GRAPH => "buildChunkGraph",
      Self::MODULE_IDS => "moduleIds",
      Self::CHUNK_IDS => "chunkIds",
      Self::MODULES_HASHES => "modulesHashes",
      Self::MODULES_CODEGEN => "modulesCodegen",
      Self::MODULES_RUNTIME_REQUIREMENTS => "modulesRuntimeRequirements",
      Self::CHUNKS_RUNTIME_REQUIREMENTS => "chunksRuntimeRequirements",
      Self::CHUNKS_HASHES => "chunksHashes",
      Self::CHUNKS_RENDER => "chunksRender",
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
        let mutations = mutations.lock().expect("failed to lock mutations");
        f.debug_struct("Hot").field("mutations", &*mutations).finish()
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
    IncrementalPasses::from_bits_retain(self.passes.load(Ordering::Relaxed))
  }

  pub fn disable_passes(
    &self,
    passes: IncrementalPasses,
    thing: &'static str,
    reason: &'static str,
  ) -> Option<Option<Diagnostic>> {
    if matches!(self.state, IncrementalState::Hot { .. }) {
      let current = IncrementalPasses::from_bits_retain(self.passes.load(Ordering::Relaxed));
      let passes_to_disable = current.intersection(passes);
      if !passes_to_disable.is_empty() {
        // Atomically remove the passes using fetch_and with the negated bits
        self
          .passes
          .fetch_and(!passes_to_disable.bits(), Ordering::Relaxed);
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
    if let IncrementalState::Hot { mutations } = &self.state {
      if self.passes().allow_write() {
        return Some(mutations.lock().expect("failed to lock mutations"));
      }
    }
    None
  }

  pub fn mutations_read(&self, passes: IncrementalPasses) -> Option<MutexGuard<'_, Mutations>> {
    if let IncrementalState::Hot { mutations } = &self.state {
      if self.passes().allow_read(passes) {
        return Some(mutations.lock().expect("failed to lock mutations"));
      }
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
