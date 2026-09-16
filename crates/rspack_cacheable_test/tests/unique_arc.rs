#![feature(unique_rc_arc)]

use std::{
  alloc::{GlobalAlloc, Layout, System},
  cell::Cell,
  sync::{Arc, UniqueArc},
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
  with::AsUniqueArc,
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

#[cacheable_dyn(unique_arc)]
trait Value: Send + Sync {
  fn value(&self) -> u64;
  fn set_value(&mut self, value: u64);
}

#[cacheable]
struct SharedValue(#[cacheable(with=AsUniqueArc)] Arc<dyn Value>);

#[cacheable]
#[repr(align(64))]
struct AlignedValue {
  data: [u64; 32],
}

#[cacheable_dyn(unique_arc)]
impl Value for AlignedValue {
  fn value(&self) -> u64 {
    self.data[0]
  }

  fn set_value(&mut self, value: u64) {
    self.data[0] = value;
  }
}

#[test]
fn deserialize_unique_allocates_once_and_preserves_allocation_when_published() {
  let value: Box<dyn Value> = Box::new(AlignedValue { data: [42; 32] });
  let bytes = to_bytes(&value, &Context).expect("serialize value");
  let archived = access::<Archived<Box<dyn Value>>, Error>(&bytes).expect("validate value");
  let mut pool = Pool::default();
  let deserializer: &mut Deserializer = Strategy::wrap(&mut pool);

  ALLOCATIONS.with(|count| count.set(Some(0)));
  let restored = archived.get().deserialize_unique(deserializer);
  let allocations = ALLOCATIONS.with(|count| count.replace(None));

  let mut restored = restored.expect("deserialize value directly into UniqueArc");
  assert_eq!(
    allocations,
    Some(1),
    "only the UniqueArc allocation is needed"
  );
  assert_eq!(restored.value(), 42);
  restored.set_value(7);
  let ptr = &*restored as *const dyn Value;
  assert_eq!(ptr.cast::<()>() as usize % 64, 0);
  let shared = UniqueArc::into_arc(restored);
  assert!(std::ptr::eq(ptr, Arc::as_ptr(&shared)));
  assert_eq!(shared.value(), 7);
  assert_eq!(Arc::strong_count(&shared), 1);
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

  let (first, second) = restored.expect("restore shared values through UniqueArc");
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
