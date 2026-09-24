#[cfg(allocative)]
use rspack_util::allocative;

#[derive(Debug)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
pub struct NodeOption {
  pub dirname: NodeDirnameOption,
  pub global: NodeGlobalOption,
  pub filename: NodeFilenameOption,
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
pub enum NodeGlobalOption {
  True,
  False,
  Warn,
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
pub enum NodeDirnameOption {
  True,
  False,
  WarnMock,
  Mock,
  EvalOnly,
  NodeModule,
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
pub enum NodeFilenameOption {
  True,
  False,
  WarnMock,
  Mock,
  EvalOnly,
  NodeModule,
}
