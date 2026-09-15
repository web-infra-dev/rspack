use super::{Dependency, DependencyContext, ImportAttributes};

#[test]
#[cfg(target_pointer_width = "64")]
fn dependency_keeps_rare_import_payload_out_of_hot_storage() {
  assert_eq!(std::mem::size_of::<DependencyContext<'static>>(), 144);
  assert_eq!(std::mem::size_of::<Dependency<'static>>(), 48);
  assert_eq!(std::mem::size_of::<ImportAttributes<'static>>(), 48);
}
