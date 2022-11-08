use rspack_sources::Source;

pub fn is_source_equal(a: &dyn Source, b: &dyn Source) -> bool {
  let a_source = a.buffer();
  let b_source = b.buffer();

  a_source == b_source
}
