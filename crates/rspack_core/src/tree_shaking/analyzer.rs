use super::visitor::OptimizeAnalyzeResult;
use crate::Compilation;

pub trait OptimizeAnalyzer {
  fn analyze(&self) -> OptimizeAnalyzeResult;
}
