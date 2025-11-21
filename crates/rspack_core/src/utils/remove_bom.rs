use std::sync::Arc;

use rspack_sources::{ReplaceSource, Source, SourceExt, SourceValue};

pub fn remove_bom(source: Arc<dyn Source>) -> Arc<dyn Source> {
  let has_bom = match source.source() {
    SourceValue::String(content) => content.starts_with('\u{feff}'),
    SourceValue::Buffer(buffer) => buffer.starts_with("\u{feff}".as_bytes()),
  };

  if !has_bom {
    return source;
  }

  let mut replace_source = ReplaceSource::new(source);
  replace_source.replace(0, 3, "", None);
  replace_source.boxed()
}
