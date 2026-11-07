use rspack_sources::{Source, is_source_map_text, source_content_eq};

pub fn is_source_equal(a: &dyn Source, b: &dyn Source) -> bool {
  if is_source_map_text(a) || is_source_map_text(b) {
    return source_content_eq(a, b);
  }
  a.buffer() == b.buffer()
}
