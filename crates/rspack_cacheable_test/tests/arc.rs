use std::{
  alloc::{GlobalAlloc, Layout, System},
  cell::Cell,
  sync::Arc,
};

use rspack_cacheable::{
  CacheableContext, Deserializer, Error, enable_cacheable as cacheable,
  enable_cacheable_dyn as cacheable_dyn,
  rkyv::{
    Archived, Deserialize, access,
    de::{Pool, Pooling, PoolingState},
    rancor::Strategy,
  },
  to_bytes,
  with::AsArc,
};

struct CountingAllocator;

thread_local! {
  static ALLOCATIONS: Cell<Option<usize>> = const { Cell::new(None) };
}

unsafe impl GlobalAlloc for CountingAllocator {
  unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    ALLOCATIONS.with(|count| count.set(count.get().map(|n| n + 1)));
    unsafe { System.alloc(layout) }
  }

  unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
    unsafe { System.dealloc(ptr, layout) }
  }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

struct Context;

impl CacheableContext for Context {
  fn project_root(&self) -> Option<&std::path::Path> {
    None
  }
}

#[cacheable_dyn(arc)]
trait Value: Send + Sync {
  fn value(&self) -> u64;
  fn shared(&self) -> Option<&Arc<u64>> {
    None
  }
}

#[cacheable]
struct SharedValue(#[cacheable(with=AsArc)] Arc<dyn Value>);

#[cacheable]
#[repr(align(64))]
struct AlignedValue {
  data: [u64; 32],
}

#[cacheable_dyn(arc)]
impl Value for AlignedValue {
  fn value(&self) -> u64 {
    self.data[0]
  }
}

#[test]
fn shared_deserialization_allocates_once_and_preserves_pool_ownership() {
  let value = SharedValue(Arc::new(AlignedValue { data: [42; 32] }));
  let values = (SharedValue(Arc::clone(&value.0)), value);
  let bytes = to_bytes(&values, &Context).expect("serialize shared values");
  let archived =
    access::<Archived<(SharedValue, SharedValue)>, Error>(&bytes).expect("validate shared values");
  let mut pool = Pool::with_capacity(1);

  ALLOCATIONS.with(|count| count.set(Some(0)));
  let restored: Result<(SharedValue, SharedValue), Error> =
    archived.deserialize(Strategy::wrap(&mut pool));
  let allocations = ALLOCATIONS.with(|count| count.replace(None));

  let (first, second) = restored.expect("restore shared values directly into Arc");
  assert_eq!(allocations, Some(1), "aliases share one Arc allocation");
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

#[test]
fn pending_shared_value_is_rejected() {
  let value = SharedValue(Arc::new(AlignedValue { data: [42; 32] }));
  let bytes = to_bytes(&value, &Context).expect("serialize shared value");
  let archived = access::<Archived<SharedValue>, Error>(&bytes).expect("validate shared value");
  let mut pool = Pool::default();
  let deserializer: &mut Deserializer = Strategy::wrap(&mut pool);
  let address = archived.0.get() as *const _ as *const () as usize;
  assert!(matches!(
    deserializer.start_pooling(address),
    PoolingState::Started
  ));
  let restored: Result<SharedValue, Error> = archived.deserialize(deserializer);
  assert!(matches!(
    restored,
    Err(Error::MessageError("cyclic shared trait object"))
  ));
}

#[cacheable]
struct NestedValue {
  shared: Arc<u64>,
}

#[cacheable_dyn(arc)]
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
