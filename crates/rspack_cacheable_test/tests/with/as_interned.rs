use std::{
  path::Path,
  sync::{Arc, Barrier},
  thread,
};

use rspack_cacheable::{
  CacheableContext, Interner, cacheable, from_bytes, to_bytes,
  with::{AsInterned, AsPreset, AsRefStr, AsRefStrConverter},
};
use serde_json::{Value, json};

#[derive(Default)]
struct Context {
  interner: Interner,
}

impl CacheableContext for Context {
  fn project_root(&self) -> Option<&Path> {
    None
  }
  fn interner(&self) -> Option<&Interner> {
    Some(&self.interner)
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

#[cacheable]
#[derive(Debug, PartialEq, Eq)]
struct Module {
  #[cacheable(with=AsInterned<AsPreset>)]
  json: Arc<Value>,
  #[cacheable(with=AsInterned<AsRefStr>)]
  options: Arc<Options>,
}

fn module(json: Value, options: &str) -> Module {
  Module {
    json: Arc::new(json),
    options: Arc::new(Options(options.to_string())),
  }
}

#[test]
fn shares_equal_values_across_decodes() {
  let context = Context::default();
  let first = to_bytes(&module(json!({ "name": "pkg" }), "{}"), &context).unwrap();
  let second = to_bytes(&module(json!({ "name": "pkg" }), "{}"), &context).unwrap();

  let first: Module = from_bytes(&first, &context).unwrap();
  let second: Module = from_bytes(&second, &context).unwrap();

  assert_eq!(first, module(json!({ "name": "pkg" }), "{}"));
  assert!(Arc::ptr_eq(&first.json, &second.json));
  assert!(Arc::ptr_eq(&first.options, &second.options));
}

#[test]
fn keeps_different_values_apart() {
  let context = Context::default();
  let first = to_bytes(&module(json!({ "version": "1.0.0" }), "a"), &context).unwrap();
  let second = to_bytes(&module(json!({ "version": "2.0.0" }), "b"), &context).unwrap();

  let first: Module = from_bytes(&first, &context).unwrap();
  let second: Module = from_bytes(&second, &context).unwrap();

  assert_eq!(first, module(json!({ "version": "1.0.0" }), "a"));
  assert_eq!(second, module(json!({ "version": "2.0.0" }), "b"));
}

#[test]
fn keeps_equal_text_of_different_types_apart() {
  // Both fields archive to the same text, which must not resolve to one entry.
  let context = Context::default();
  let bytes = to_bytes(&module(json!({}), "{}"), &context).unwrap();

  let decoded: Module = from_bytes(&bytes, &context).unwrap();

  assert_eq!(decoded, module(json!({}), "{}"));
}

#[test]
fn does_not_share_without_an_interner() {
  let bytes = to_bytes(&module(json!({ "name": "pkg" }), "{}"), &()).unwrap();

  let first: Module = from_bytes(&bytes, &()).unwrap();
  let second: Module = from_bytes(&bytes, &()).unwrap();

  assert_eq!(first, second);
  assert!(!Arc::ptr_eq(&first.json, &second.json));
}

#[test]
fn releases_values_after_last_decoded_owner_is_dropped() {
  let context = Context::default();
  let bytes = to_bytes(&module(json!({ "name": "pkg" }), "{}"), &()).unwrap();
  let first: Module = from_bytes(&bytes, &context).unwrap();
  let second: Module = from_bytes(&bytes, &context).unwrap();
  assert!(Arc::ptr_eq(&first.json, &second.json));
  assert!(Arc::ptr_eq(&first.options, &second.options));
  let json = Arc::downgrade(&first.json);
  let options = Arc::downgrade(&first.options);

  drop(first);
  assert!(json.upgrade().is_some());
  assert!(options.upgrade().is_some());
  drop(second);

  // Observe the values while the compiler's context would still be alive.
  let retained = (json.upgrade().is_some(), options.upgrade().is_some());
  drop(context);

  assert_eq!(
    retained,
    (false, false),
    "the interner must not retain JSON or options after all decoded owners are dropped"
  );
}

#[test]
fn releases_values_when_context_is_dropped() {
  let context = Context::default();
  let bytes = to_bytes(&module(json!({ "name": "pkg" }), "{}"), &()).unwrap();
  let decoded: Module = from_bytes(&bytes, &context).unwrap();
  let json = Arc::downgrade(&decoded.json);
  let options = Arc::downgrade(&decoded.options);

  drop(decoded);
  drop(context);

  assert!(json.upgrade().is_none());
  assert!(options.upgrade().is_none());
}

#[test]
fn does_not_retain_obsolete_values_across_decodes() {
  let context = Context::default();
  let mut previous_values = Vec::new();

  // Model changing package.json contents and loader options across rebuilds.
  for version in 0..16 {
    let bytes = to_bytes(
      &module(json!({ "version": version }), &format!("options-{version}")),
      &(),
    )
    .unwrap();
    let decoded: Module = from_bytes(&bytes, &context).unwrap();
    previous_values.push((
      Arc::downgrade(&decoded.json),
      Arc::downgrade(&decoded.options),
    ));
    drop(decoded);
  }

  let retained_json = previous_values
    .iter()
    .filter(|(json, _)| json.upgrade().is_some())
    .count();
  let retained_options = previous_values
    .iter()
    .filter(|(_, options)| options.upgrade().is_some())
    .count();
  drop(context);

  assert_eq!(
    (retained_json, retained_options),
    (0, 0),
    "obsolete JSON and options must not accumulate while the context is alive"
  );
}

#[test]
fn keeps_different_contexts_isolated() {
  let first_context = Context::default();
  let second_context = Context::default();
  let bytes = to_bytes(&module(json!({ "name": "pkg" }), "{}"), &()).unwrap();
  let first: Module = from_bytes(&bytes, &first_context).unwrap();
  let second: Module = from_bytes(&bytes, &second_context).unwrap();

  assert_eq!(first, second);
  assert!(!Arc::ptr_eq(&first.json, &second.json));
  assert!(!Arc::ptr_eq(&first.options, &second.options));
}

#[test]
fn shares_equal_values_across_parallel_decodes() {
  const WORKERS: usize = 16;
  let context = Context::default();
  let bytes = to_bytes(&module(json!({ "name": "pkg" }), "{}"), &()).unwrap();
  let barrier = Barrier::new(WORKERS);

  let decoded = thread::scope(|scope| {
    let handles: Vec<_> = (0..WORKERS)
      .map(|_| {
        scope.spawn(|| {
          barrier.wait();
          from_bytes::<Module, _>(&bytes, &context).unwrap()
        })
      })
      .collect();
    handles
      .into_iter()
      .map(|handle| handle.join().unwrap())
      .collect::<Vec<_>>()
  });

  for other in &decoded[1..] {
    assert_eq!(&decoded[0], other);
    assert!(Arc::ptr_eq(&decoded[0].json, &other.json));
    assert!(Arc::ptr_eq(&decoded[0].options, &other.options));
  }
}
