use rspack_cacheable::cacheable;

#[cacheable]
#[derive(Debug, Clone)]
pub struct Meta {
  pub max_dependency_id: u32,
}
