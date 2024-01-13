#[derive(Debug, PartialEq, Eq, Default, Clone)]
pub enum SourceMapKind {
  #[default]
  None,
  SourceMap,
  SimpleSourceMap,
}
