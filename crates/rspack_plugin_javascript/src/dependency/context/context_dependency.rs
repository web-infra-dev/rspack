use rspack_core::Dependency;

pub trait ContextDependencyTrait: Dependency {
  fn chunk_name(&self) -> Option<&str>;
}
