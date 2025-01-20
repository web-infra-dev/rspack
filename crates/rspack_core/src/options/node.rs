#[derive(Debug, Clone)]
pub struct NodeOption {
  pub dirname: NodeDirnameOption,
  pub global: NodeGlobalOption,
  pub filename: NodeFilenameOption,
}

#[derive(Debug, Clone, Copy)]
pub enum NodeGlobalOption {
  True,
  False,
  Warn,
}

#[derive(Debug, Clone, Copy)]
pub enum NodeDirnameOption {
  True,
  False,
  WarnMock,
  Mock,
  EvalOnly,
  NodeModule,
}

#[derive(Debug, Clone, Copy)]
pub enum NodeFilenameOption {
  True,
  False,
  WarnMock,
  Mock,
  EvalOnly,
  NodeModule,
}
