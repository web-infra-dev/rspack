use std::{
  borrow::Cow,
  path::{Path, PathBuf},
};

use swc_common::{Mark, SyntaxContext, GLOBALS};

use crate::symbol_box::SWC_GLOBALS;

pub trait PathExt {
  fn as_str(&self) -> Cow<'_, str>;
}

impl PathExt for Path {
  #[inline]
  fn as_str(&self) -> Cow<'_, str> {
    self.to_string_lossy()
  }
}

pub trait StrExt {
  fn as_path(&self) -> PathBuf;
}

impl StrExt for str {
  #[inline]
  fn as_path(&self) -> PathBuf {
    Path::new(&self).to_owned()
  }
}

pub trait MarkExt {
  fn as_ctxt(&self) -> SyntaxContext;
}

impl MarkExt for Mark {
  #[inline]
  fn as_ctxt(&self) -> SyntaxContext {
    
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
