use super::visitor::DepdencyAnalyzeResult;
use crate::Compilation;

pub trait DependencyAnalyzer {
  fn analyze(&self, compilation: &Compilation) -> DepdencyAnalyzeResult;
}
