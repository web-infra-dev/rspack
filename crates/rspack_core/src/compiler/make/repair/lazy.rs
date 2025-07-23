use crate::LazyDependenciesInfo;

#[derive(Debug)]
pub enum HasLazyDependencies {
  Maybe,
  Has(LazyDependenciesInfo),
}
