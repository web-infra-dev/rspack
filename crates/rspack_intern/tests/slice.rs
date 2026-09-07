use rspack_intern::{InternSliceStorage, InternedSlice, SliceInternable};

struct Bytes;

impl SliceInternable for Bytes {
  type Header = u64;
  type Item = u8;

  fn hash(header: &u64, _items: &[u8]) -> u64 {
    *header
  }

  fn eq(a: &[u8], b: &[u8]) -> bool {
    a == b
  }

  fn storage() -> &'static InternSliceStorage<Self> {
    static STORAGE: InternSliceStorage<Bytes> = InternSliceStorage::new();
    &STORAGE
  }
}

fn intern(items: &[u8]) -> InternedSlice<Bytes> {
  InternedSlice::new(items.len() as u64, items)
}

fn map_len() -> usize {
  Bytes::storage().len()
}

#[test]
fn smoke_test() {
  let base = map_len();

  let a = intern(b"aa");
  let same_a = intern(b"aa");
  let cloned_a = a.clone();
  let b = intern(b"bb");

  assert_eq!(map_len(), base + 2, "equal values share one entry");
  assert_eq!(a, same_a);
  assert_eq!(a.items(), b"aa");
  assert_eq!(*a.header(), 2);
  assert_ne!(a, b);

  drop(same_a);
  drop(cloned_a);
  assert_eq!(map_len(), base + 2, "still held by `a`");

  drop(a);
  assert_eq!(map_len(), base + 1);
  drop(b);
  assert_eq!(map_len(), base);
}

#[test]
fn interning_races_with_dropping() {
  // A hash that ignores the length, so unequal values collide and exercise the item comparison.
  struct Colliding;

  impl SliceInternable for Colliding {
    type Header = u64;
    type Item = u8;

    fn hash(_header: &u64, _items: &[u8]) -> u64 {
      0
    }

    fn eq(a: &[u8], b: &[u8]) -> bool {
      a == b
    }

    fn storage() -> &'static InternSliceStorage<Self> {
      static STORAGE: InternSliceStorage<Colliding> = InternSliceStorage::new();
      &STORAGE
    }
  }

  let storage = Colliding::storage();
  assert_eq!(storage.len(), 0);

  std::thread::scope(|scope| {
    for _ in 0..8 {
      scope.spawn(|| {
        for i in 0..2000u32 {
          let items = [(i % 16) as u8, 7];
          let value = InternedSlice::<Colliding>::new(0, &items);
          assert_eq!(value.items(), items);
        }
      });
    }
  });

  assert_eq!(storage.len(), 0, "every value is freed once unreferenced");
}
