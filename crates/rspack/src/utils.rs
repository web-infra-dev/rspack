use std::sync::Arc;

use once_cell::sync::Lazy;
use swc::{Compiler, common::{FilePathMapping, SourceMap}};

static COMPILER: Lazy<Arc<Compiler>> = Lazy::new(|| {
  let cm = Arc::new(SourceMap::new(FilePathMapping::empty()));

  Arc::new(Compiler::new(cm))
});

pub fn get_compiler() -> Arc<Compiler> {
  COMPILER.clone()
}