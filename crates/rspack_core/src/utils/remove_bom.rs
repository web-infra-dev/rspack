use std::sync::Arc;

use rspack_sources::{ReplaceSource, Source, SourceExt};

pub fn remove_bom(s: Arc<dyn Source>) -> Arc<dyn Source> {
  if s.source().starts_with('\u{feff}') {
    let mut s = ReplaceSource::new(s);
    s.replace(0, 3, "", None);
    s.boxed()
  } else {
    s
  }
}
