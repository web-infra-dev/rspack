mod mutations;

use std::fmt;

use bitflags::bitflags;
pub use mutations::{Mutation, Mutations};
use rspack_error::{miette, miette::Diagnostic, thiserror, thiserror::Error};

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
  pub fn can_write_mutations(&self) -> bool {
    !self.is_empty()
  }

  pub fn can_read_mutations(&self, pass: IncrementalPasses) -> bool {
    self.contains(pass)
  }
}

#[derive(Debug)]
pub enum Incremental {
  Build,
  Rebuild {
    passes: IncrementalPasses,
    mutations: Mutations,
  },
}

impl Incremental {
  pub fn new_build() -> Self {
    Self::Build
  }

  pub fn new_rebuild(passes: IncrementalPasses) -> Self {
    Self::Rebuild {
      passes,
      mutations: Mutations::default(),
    }
  }

  pub fn disable_passes(&mut self, passes: IncrementalPasses) -> bool {
    if let Self::Rebuild { passes: p, .. } = self
      && p.contains(passes)
    {
      p.remove(passes);
      return true;
    }
    false
  }

  pub fn can_write_mutations(&self) -> bool {
    if let Self::Rebuild { passes, .. } = self {
      return passes.can_write_mutations();
    }
    false
  }

  pub fn can_read_mutations(&self, passes: IncrementalPasses) -> bool {
    if let Self::Rebuild { passes: p, .. } = self {
      return p.can_read_mutations(passes);
    }
    false
  }

  pub fn mutations_write(&mut self) -> Option<&mut Mutations> {
    if let Self::Rebuild { passes, mutations } = self {
      return passes.can_write_mutations().then_some(mutations);
    }
    None
  }

  pub fn mutations_read(&self, passes: IncrementalPasses) -> Option<&Mutations> {
    if let Self::Rebuild {
      passes: p,
      mutations,
    } = self
    {
      return p.can_read_mutations(passes).then_some(mutations);
    }
    None
  }
}

#[derive(Debug, Error, Diagnostic)]
#[diagnostic(code(NotFriendlyForIncremental))]
#[diagnostic(severity(Warning))]
#[error(
  r#"{thing} is not friendly for incremental, {reason}. For the last compilation {passes} is fallback to non-incremental."#
)]
pub struct NotFriendlyForIncremental {
  pub thing: &'static str,
  pub reason: &'static str,
  pub passes: IncrementalPasses,
}
