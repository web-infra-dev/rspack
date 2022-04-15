

use swc_common::{Mark, SyntaxContext, GLOBALS};

use crate::mark_box::SWC_GLOBALS;

pub trait MarkExt {
  fn to_ctxt(&self) -> SyntaxContext;
}

impl MarkExt for Mark {
  #[inline]
  fn to_ctxt(&self) -> SyntaxContext {
    
    GLOBALS.set(&SWC_GLOBALS, || SyntaxContext::empty().apply_mark(*self))
  }
}

pub trait SyntaxContextExt {
  fn as_mark(&self) -> Mark;
}

impl SyntaxContextExt for SyntaxContext {

  #[inline]
  fn as_mark(&self) -> Mark {
    GLOBALS.set(&SWC_GLOBALS, || self.outer())
  }
}
