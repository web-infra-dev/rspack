use super::visitor::OptimizeAnalyzeResult;

pub trait OptimizeAnalyzer {
  fn analyze(&self) -> OptimizeAnalyzeResult;
}
