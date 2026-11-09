use rspack_sources::{Source, is_devtool_text_source, source_content_eq};

pub fn is_source_equal(a: &dyn Source, b: &dyn Source) -> bool {
  if is_devtool_text_source(a) || is_devtool_text_source(b) {
    return source_content_eq(a, b);
  }
  a.buffer() == b.buffer()
}
