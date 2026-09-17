use std::sync::Arc;

use rspack_cacheable::{
  CacheableContext, Error, enable_cacheable as cacheable, enable_cacheable_dyn as cacheable_dyn,
  rkyv::{Archived, Deserialize, access, de::Pool, rancor::Strategy},
  to_bytes,
};

struct Context;

impl CacheableContext for Context {
  fn project_root(&self) -> Option<&std::path::Path> {
    None
  }
}

#[cacheable_dyn]
trait Value: Send + Sync {
  fn value(&self) -> u64;
  fn shared(&self) -> Option<&Arc<u64>> {
    None
  }
}

#[cacheable]
struct SharedValue(Arc<dyn Value>);

#[cacheable]
#[repr(align(64))]
struct AlignedValue {
  data: [u64; 32],
}

#[cacheable_dyn]
impl Value for AlignedValue {
  fn value(&self) -> u64 {
    self.data[0]
  }
}

#[test]
fn shared_deserialization_preserves_pool_ownership() {
  let value = SharedValue(Arc::new(AlignedValue { data: [42; 32] }));
  let values = (SharedValue(Arc::clone(&value.0)), value);
  let bytes = to_bytes(&values, &Context).expect("serialize shared values");
  let archived =
    access::<Archived<(SharedValue, SharedValue)>, Error>(&bytes).expect("validate shared values");
  let mut pool = Pool::with_capacity(1);

  let restored: Result<(SharedValue, SharedValue), Error> =
    archived.deserialize(Strategy::wrap(&mut pool));

  let (first, second) = restored.expect("restore shared values");
  assert!(Arc::ptr_eq(&first.0, &second.0));
  assert_eq!(Arc::as_ptr(&first.0).cast::<()>() as usize % 64, 0);
  assert_eq!(
    Arc::strong_count(&first.0),
    3,
    "the pool owns one reference"
  );
  drop(pool);
  assert_eq!(Arc::strong_count(&first.0), 2);
  assert_eq!(second.0.value(), 42);
  let weak = Arc::downgrade(&first.0);
  drop(first);
  drop(second);
  assert!(weak.upgrade().is_none());
}

#[cacheable]
struct NestedValue {
  shared: Arc<u64>,
}

#[cacheable_dyn]
impl Value for NestedValue {
  fn value(&self) -> u64 {
    *self.shared
  }

  fn shared(&self) -> Option<&Arc<u64>> {
    Some(&self.shared)
  }
}

#[test]
fn shared_objects_preserve_sharing_with_siblings_and_external_references() {
  let shared = Arc::new(42);
  let first = SharedValue(Arc::new(NestedValue {
    shared: Arc::clone(&shared),
  }));
  let second = SharedValue(Arc::new(NestedValue {
    shared: Arc::clone(&shared),
  }));
  let bytes = to_bytes(&(first, second, shared), &Context).expect("serialize nested sharing");
  let (first, second, shared): (SharedValue, SharedValue, Arc<u64>) =
    rspack_cacheable::from_bytes(&bytes, &Context).expect("restore nested sharing");
  assert!(Arc::ptr_eq(first.0.shared().expect("nested arc"), &shared));
  assert!(Arc::ptr_eq(second.0.shared().expect("nested arc"), &shared));
  assert_eq!(Arc::strong_count(&shared), 3);
  assert!(!Arc::ptr_eq(&first.0, &second.0));
  drop(first);
  assert_eq!(second.0.value(), 42);
  assert_eq!(Arc::strong_count(&shared), 2);
}
