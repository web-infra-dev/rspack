#[derive(Debug, PartialEq, Eq, Default, Clone)]
pub enum SourceMapKind {
  #[default]
  None,
  SourceMap,
  SimpleSourceMap,
}

pub trait SourceMapGenConfig {
  fn get_source_map_kind(&self) -> &SourceMapKind;
  fn set_source_map_kind(&mut self, source_map: SourceMapKind);
}
