use rspack_sources::{MappingSource, Source};

pub fn is_source_equal(a: &dyn Source, b: &dyn Source) -> bool {
  if let Some(mapping) = a.as_any().downcast_ref::<MappingSource>() {
    return mapping.content_eq(b);
  }
  if let Some(mapping) = b.as_any().downcast_ref::<MappingSource>() {
    return mapping.content_eq(a);
  }
  let a_source = a.buffer();
  let b_source = b.buffer();

  a_source == b_source
}
