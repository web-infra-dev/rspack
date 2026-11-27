use std::{
  sync::{Arc, Barrier, LazyLock},
  thread,
};

use rspack_cacheable::{
  enable_cacheable as cacheable, from_bytes, to_bytes,
  with::{AsInterned, AsInternedConverter, AsRefStr, AsRefStrConverter},
};

#[cacheable(with=AsRefStr)]
#[derive(Debug, PartialEq, Eq)]
struct Name(String);

impl AsRefStrConverter for Name {
  fn as_str(&self) -> &str {
    &self.0
  }
  fn from_str(s: &str) -> Self {
    Self(s.to_string())
  }
}

#[derive(Debug, PartialEq, Eq)]
struct Options(String);

impl AsRefStrConverter for Options {
  fn as_str(&self) -> &str {
    &self.0
  }
  fn from_str(s: &str) -> Self {
    Self(s.to_string())
  }
}

#[derive(Debug)]
enum Source {
  Borrowed(&'static Options),
  Owned(Arc<Options>),
}

impl AsInternedConverter for Source {
  type Inner = Options;
  fn to_inner(&self) -> &Self::Inner {
    match self {
      Self::Borrowed(options) => options,
      Self::Owned(options) => options,
    }
  }
  fn from_interned(inner: Arc<Self::Inner>) -> Self {
    Self::Owned(inner)
  }
}

#[cacheable]
#[derive(Debug)]
struct Module {
  #[cacheable(with=AsInterned)]
  name: Arc<Name>,
  #[cacheable(with=AsInterned<AsRefStr>)]
  options: Source,
}

fn module(name: &str, options: &'static Options) -> Module {
  Module {
    name: Arc::new(Name(name.to_string())),
    options: Source::Borrowed(options),
  }
}

fn restore(module: &Module) -> Module {
  let bytes = to_bytes(module, &()).expect("serialize module");
  from_bytes(&bytes, &()).expect("restore module")
}

fn owned(source: &Source) -> &Arc<Options> {
  match source {
    Source::Owned(options) => options,
    Source::Borrowed(_) => panic!("restored source should own its options"),
  }
}

#[test]
fn test_as_interned() {
  static OPTIONS: LazyLock<Options> = LazyLock::new(|| Options(r#"{"jsc":{}}"#.into()));
  static OTHER_OPTIONS: LazyLock<Options> = LazyLock::new(|| Options(r#"{"jsc":{}}"#.into()));

  let first = restore(&module("as-interned-a", &OPTIONS));
  assert_eq!(first.name.0, "as-interned-a");
  assert_eq!(Arc::strong_count(&first.name), 1);
  assert_eq!(Arc::strong_count(owned(&first.options)), 1);

  let second = restore(&module("as-interned-a", &OTHER_OPTIONS));
  assert!(Arc::ptr_eq(&first.name, &second.name));
  assert!(Arc::ptr_eq(owned(&first.options), owned(&second.options)));

  let third = restore(&module("as-interned-b", &OPTIONS));
  assert_eq!(third.name.0, "as-interned-b");
  assert!(!Arc::ptr_eq(&first.name, &third.name));

  let options = Arc::downgrade(owned(&first.options));
  drop((first, second, third));
  assert!(options.upgrade().is_none());

  let restored = restore(&module("as-interned-a", &OPTIONS));
  assert_eq!(Arc::strong_count(&restored.name), 1);
  assert_eq!(**owned(&restored.options), *OPTIONS);
}

#[test]
fn test_as_interned_keeps_equal_text_of_different_types_apart() {
  static OPTIONS: LazyLock<Options> = LazyLock::new(|| Options("as-interned-same-text".into()));

  let first = restore(&module("as-interned-same-text", &OPTIONS));
  let second = restore(&module("as-interned-same-text", &OPTIONS));
  assert_eq!(first.name.0, "as-interned-same-text");
  assert_eq!(owned(&first.options).0, "as-interned-same-text");
  assert!(Arc::ptr_eq(&first.name, &second.name));
  assert!(Arc::ptr_eq(owned(&first.options), owned(&second.options)));
}

#[test]
fn test_as_interned_shares_across_parallel_decodes() {
  static OPTIONS: LazyLock<Options> = LazyLock::new(|| Options(r#"{"parallel":true}"#.into()));
  const WORKERS: usize = 8;

  let bytes = to_bytes(&module("as-interned-parallel", &OPTIONS), &()).expect("serialize module");
  let barrier = Barrier::new(WORKERS);
  let decoded = thread::scope(|scope| {
    let handles = (0..WORKERS)
      .map(|_| {
        scope.spawn(|| {
          barrier.wait();
          from_bytes::<Module, _>(&bytes, &()).expect("restore module")
        })
      })
      .collect::<Vec<_>>();
    handles
      .into_iter()
      .map(|handle| handle.join().expect("decode thread"))
      .collect::<Vec<_>>()
  });

  for other in &decoded[1..] {
    assert!(Arc::ptr_eq(&decoded[0].name, &other.name));
    assert!(Arc::ptr_eq(
      owned(&decoded[0].options),
      owned(&other.options)
    ));
  }
}

#[test]
fn test_as_interned_keeps_live_values_after_pruning() {
  static OPTIONS: LazyLock<Options> = LazyLock::new(|| Options(r#"{"prune":true}"#.into()));
  let name = |i: usize| format!("as-interned-prune-{i}");

  let restored = (0..2048)
    .map(|i| restore(&module(&name(i), &OPTIONS)))
    .collect::<Vec<_>>();
  let kept = restored
    .iter()
    .step_by(2)
    .map(|module| module.name.clone())
    .collect::<Vec<_>>();
  drop(restored);

  let extra = (2048..6144)
    .map(|i| restore(&module(&name(i), &OPTIONS)))
    .collect::<Vec<_>>();
  assert_eq!(extra.len(), 4096);

  for (index, kept_name) in kept.iter().enumerate() {
    let again = restore(&module(&name(index * 2), &OPTIONS));
    assert!(Arc::ptr_eq(kept_name, &again.name));
  }
  let released = restore(&module(&name(1), &OPTIONS));
  assert_eq!(released.name.0, name(1));
  assert_eq!(Arc::strong_count(&released.name), 1);
}
