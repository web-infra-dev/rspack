#[derive(Debug, Clone)]
pub struct OptimizationOptions {
  pub chunk_id_algo: ChunkIdAlgo,
  pub module_id_algo: ModuleIdAlgo,
}

impl Default for OptimizationOptions {
  fn default() -> Self {
    Self {
      chunk_id_algo: ChunkIdAlgo::Named,
      module_id_algo: ModuleIdAlgo::Named,
    }
  }
}

#[derive(Debug, Clone)]
pub enum ChunkIdAlgo {
  /// Readable ids for better debugging.
  Named,
  /// Numeric ids in order of usage.
  Numeric,
}

impl ChunkIdAlgo {
  pub fn is_named(&self) -> bool {
    matches!(self, ChunkIdAlgo::Named)
  }

  pub fn is_numeric(&self) -> bool {
    matches!(self, ChunkIdAlgo::Numeric)
  }
}

#[derive(Debug, Clone)]
pub enum ModuleIdAlgo {
  /// Readable ids for better debugging.
  Named,
  /// Numeric ids in order of usage.
  Numeric,
}

impl ModuleIdAlgo {
  pub fn is_named(&self) -> bool {
    matches!(self, Self::Named)
  }

  pub fn is_numeric(&self) -> bool {
    matches!(self, Self::Numeric)
  }
}
