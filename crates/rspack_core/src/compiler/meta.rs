use rspack_cacheable::cacheable;

/// Compiler-owned state needed to reuse modules with stored dependency IDs.
#[cacheable]
#[derive(Debug)]
pub(crate) struct Meta {
  pub max_dependency_id: u32,
}
