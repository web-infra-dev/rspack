#![cfg(allocative)]

use std::{
  cell::UnsafeCell,
  collections::HashMap,
  mem::{ManuallyDrop, MaybeUninit, size_of},
  sync::{
    Arc, LazyLock, Mutex,
    atomic::{AtomicBool, Ordering},
  },
};

use allocative::{Allocative, FlameGraph, FlameGraphBuilder, Key, Visit, Visitor};
use indexmap::{IndexMap, IndexSet};
use smallvec::SmallVec;

#[path = "support/driver_enums.rs"]
mod driver_enums;

// Custom field policies can deliberately exclude an otherwise supported payload.
fn exclude_payload<T: ?Sized>(value: &T, visitor: &mut Visitor<'_>) {
  visitor.visit_opaque(value);
}

fn capture(value: &dyn Visit) -> (usize, String, String) {
  capture_with(|visitor| value.visit_memory(visitor))
}

fn capture_with(visit: impl FnOnce(&mut Visitor<'_>)) -> (usize, String, String) {
  capture_in(FlameGraphBuilder::with_shared_ownership(), visit)
}

fn capture_in(
  mut builder: FlameGraphBuilder,
  visit: impl FnOnce(&mut Visitor<'_>),
) -> (usize, String, String) {
  let mut visitor = builder.root_visitor();
  visit(&mut visitor);
  visitor.exit();
  let output = builder.finish();
  let folded = output.flamegraph().write();
  let total = folded
    .lines()
    .map(|line| {
      line
        .rsplit_once(' ')
        .expect("folded row has a weight")
        .1
        .parse::<usize>()
        .expect("weight is an integer")
    })
    .sum();
  (total, folded, output.warnings())
}

#[test]
fn reserved_vector_capacity_is_counted_without_elements() {
  let value: Vec<u64> = Vec::with_capacity(128);
  let (total, _, warnings) = capture(&value);
  assert_eq!(
    total,
    size_of::<Vec<u64>>() + value.capacity() * size_of::<u64>()
  );
  assert!(warnings.is_empty(), "{warnings}");
}

#[test]
fn hash_map_accounts_for_capacity_and_nested_allocations() {
  let mut value = HashMap::with_capacity(64);
  let text = String::with_capacity(4096);
  let text_capacity = text.capacity();
  value.insert(1u64, text);
  let (total, _, warnings) = capture(&value);
  // Standard HashMap exposes usable capacity, not its private control-byte layout.
  assert_eq!(
    total,
    size_of_val(&value) + value.capacity() * size_of::<(u64, String)>() + text_capacity
  );
  assert!(warnings.is_empty(), "{warnings}");
}

// Deliberately test both the Arc allocation and its separately owned String buffer.
#[allow(clippy::rc_buffer)]
struct Owners {
  first: Arc<String>,
  second: Arc<String>,
}

#[test]
fn shared_allocations_stay_below_the_first_owner_and_are_counted_once() {
  let text = Arc::new("x".repeat(4096));
  let single = capture(&text).0;
  let value = Owners {
    first: text.clone(),
    second: text,
  };
  assert!(Arc::ptr_eq(&value.first, &value.second));
  let (total, folded, warnings) = capture(&value);
  assert_eq!(total, single + size_of::<Arc<String>>());
  assert!(
    folded
      .lines()
      .any(|line| line.contains("Owners;first;") && line.contains("capacity")),
    "{folded}"
  );
  assert!(
    !folded
      .lines()
      .any(|line| line.contains(";second;") && line.contains("capacity")),
    "{folded}"
  );
  assert!(warnings.is_empty(), "{warnings}");
}

struct Node {
  next: Mutex<Option<Arc<Node>>>,
  bytes: Vec<u8>,
}

#[test]
fn strong_cycles_terminate_and_deduplicate() {
  let first = Arc::new(Node {
    next: Mutex::new(None),
    bytes: vec![0; 4096],
  });
  let second = Arc::new(Node {
    next: Mutex::new(Some(first.clone())),
    bytes: vec![0; 8192],
  });
  *first.next.lock().unwrap() = Some(second.clone());
  assert_eq!(first.bytes.len() + second.bytes.len(), 4096 + 8192);
  let (total, _, warnings) = capture(&first);
  *first.next.lock().unwrap() = None;
  *second.next.lock().unwrap() = None;
  assert!(total >= 4096 + 8192);
  assert!(total < 4096 + 8192 + 1024);
  assert!(warnings.is_empty(), "{warnings}");
}

#[test]
fn profiling_does_not_initialize_lazy_values() {
  static INITIALIZED: AtomicBool = AtomicBool::new(false);
  let value: LazyLock<Vec<u8>> = LazyLock::new(|| {
    INITIALIZED.store(true, Ordering::Relaxed);
    vec![0; 4096]
  });
  let (total, _, warnings) = capture(&value);
  assert!(!INITIALIZED.load(Ordering::Relaxed));
  assert_eq!(total, size_of_val(&value));
  assert!(warnings.is_empty(), "{warnings}");
}

#[test]
fn locked_values_report_missing_coverage_without_blocking() {
  let value = Mutex::new(vec![0u8; 4096]);
  let _guard = value.lock().expect("test lock");
  let (total, _, warnings) = capture(&value);
  assert_eq!(total, size_of_val(&value));
  assert!(
    warnings.contains("Unobserved nested allocations"),
    "{warnings}"
  );
}

#[test]
fn ordinary_nested_fields_arrays_and_static_references_need_no_derive() {
  struct Leaf {
    text: String,
  }
  struct Root<'a> {
    borrowed: &'a str,
    encoder: fn(&str) -> usize,
    values: Vec<(Leaf, [Leaf; 2])>,
  }
  let borrowed = "borrowed storage is counted by its owner";
  let value = Root {
    borrowed,
    encoder: str::len,
    values: vec![(
      Leaf {
        text: String::with_capacity(31),
      },
      [
        Leaf {
          text: String::with_capacity(47),
        },
        Leaf {
          text: String::with_capacity(73),
        },
      ],
    )],
  };
  assert_eq!(value.borrowed, borrowed);
  assert_eq!((value.encoder)(borrowed), borrowed.len());
  let expected = size_of_val(&value)
    + value.values.capacity() * size_of::<(Leaf, [Leaf; 2])>()
    + value.values[0].0.text.capacity()
    + value.values[0]
      .1
      .iter()
      .map(|leaf| leaf.text.capacity())
      .sum::<usize>();
  let (total, folded, warnings) = capture(&value);
  assert_eq!(total, expected);
  assert!(folded.contains(";text;"), "{folded}");
  assert!(warnings.is_empty(), "{warnings}");
}

#[test]
fn trait_object_bridge_visits_the_unannotated_concrete_type() {
  trait Module: Visit {
    fn name(&self) -> &str;
  }
  struct TextModule {
    text: String,
  }
  impl Module for TextModule {
    fn name(&self) -> &str {
      &self.text
    }
  }
  let text = String::with_capacity(4096);
  let expected = size_of::<Box<dyn Module>>() + size_of::<TextModule>() + text.capacity();
  let value: Box<dyn Module> = Box::new(TextModule { text });
  assert_eq!(value.name(), "");
  let (total, folded, warnings) = capture(&value);
  assert_eq!(total, expected);
  assert!(folded.contains("TextModule;text;"), "{folded}");
  assert!(warnings.is_empty(), "{warnings}");
}

#[test]
fn generic_enum_adapter_accepts_unannotated_payloads() {
  enum Value<T> {
    Empty,
    Data(T),
  }
  struct Payload {
    text: String,
  }
  let payload = Payload {
    text: String::with_capacity(4096),
  };
  let value = Value::Data(payload);
  let Value::Data(payload) = &value else {
    unreachable!()
  };
  let expected = size_of::<Value<Payload>>() + payload.text.capacity();
  let (total, _, warnings) = capture(&value);
  assert_eq!(total, expected);
  assert!(warnings.is_empty(), "{warnings}");
  assert_eq!(
    capture(&Value::<Payload>::Empty).0,
    size_of::<Value<Payload>>()
  );
}

#[test]
fn generic_reflected_enum_does_not_require_erased_payloads_to_implement_visit() {
  enum Value<T: ?Sized> {
    Empty,
    Data(Box<T>),
  }
  let value: Value<dyn std::any::Any> = Value::Data(Box::new(String::with_capacity(4096)));
  let Value::Data(payload) = &value else {
    unreachable!()
  };
  let (total, _, warnings) = capture(&value);
  assert_eq!(total, size_of_val(&value) + size_of_val(&**payload));
  assert_eq!(warnings.lines().count(), 1, "{warnings}");
  assert!(
    warnings.contains(std::any::type_name::<dyn std::any::Any>()),
    "{warnings}"
  );
  assert_eq!(
    capture(&Value::<dyn std::any::Any>::Empty).0,
    size_of::<Value<dyn std::any::Any>>()
  );
}

#[test]
fn reflected_niche_enum_visits_only_the_active_recursive_payload() {
  enum Value {
    Empty,
    Next { text: String, next: Box<Value> },
  }
  let value = Value::Next {
    text: String::with_capacity(4096),
    next: Box::new(Value::Next {
      text: String::with_capacity(1024),
      next: Box::new(Value::Empty),
    }),
  };
  let Value::Next { text, next } = &value else {
    unreachable!()
  };
  let Value::Next { text: nested, .. } = &**next else {
    unreachable!()
  };
  let expected = 3 * size_of::<Value>() + text.capacity() + nested.capacity();
  let (total, folded, warnings) = capture(&value);
  assert_eq!(total, expected);
  assert!(folded.contains(";Next;text;"), "{folded}");
  assert!(warnings.is_empty(), "{warnings}");
  let (total, _, warnings) = capture(&Value::Empty);
  assert_eq!(total, size_of::<Value>());
  assert!(warnings.is_empty(), "{warnings}");
}

#[test]
fn const_generic_enum_keeps_typed_array_traversal() {
  #[derive(Allocative)]
  enum Value<const N: usize = 2> {
    Empty,
    Strings([String; N]),
  }
  let value = Value::Strings([String::with_capacity(4096), String::with_capacity(1024)]);
  let Value::Strings(texts) = &value else {
    unreachable!()
  };
  let expected = size_of_val(&value) + texts.iter().map(String::capacity).sum::<usize>();
  let (total, _, warnings) = capture(&value);
  assert_eq!(total, expected);
  assert!(warnings.is_empty(), "{warnings}");
  assert_eq!(capture(&Value::<0>::Empty).0, size_of::<Value<0>>());
}

#[test]
fn reflected_enum_variants_match_typed_traversal_with_custom_discriminants() {
  #[derive(Allocative)]
  #[repr(i16)]
  enum Value {
    Empty = -9,
    Text(String) = 42,
    Named {
      marker: u8,
      bytes: Vec<u8>,
      text: Option<Box<(String, u64)>>,
    } = 90,
  }
  for value in [
    Value::Empty,
    Value::Text(String::with_capacity(4096)),
    Value::Named {
      marker: 7,
      bytes: Vec::with_capacity(8192),
      text: Some(Box::new((String::with_capacity(1024), 23))),
    },
  ] {
    let expected = capture_with(|visitor| {
      let mut visitor = visitor.enter_self(&value);
      match &value {
        Value::Empty => {}
        Value::Text(text) => {
          let mut variant = visitor.enter(allocative::Key::new("Text"), size_of::<Value>());
          variant.visit_field(allocative::Key::new("0"), text);
          variant.exit();
        }
        Value::Named {
          marker,
          bytes,
          text,
        } => {
          let mut variant = visitor.enter(allocative::Key::new("Named"), size_of::<Value>());
          variant.visit_field(allocative::Key::new("marker"), marker);
          variant.visit_field(allocative::Key::new("bytes"), bytes);
          variant.visit_field(allocative::Key::new("text"), text);
          variant.exit();
        }
      }
      visitor.exit();
    });
    assert_eq!(capture(&value), expected);
  }
}

#[test]
fn enum_field_and_variant_policies_keep_typed_traversal() {
  #[derive(Allocative)]
  enum Value {
    Data {
      text: String,
      #[allocative(skip)]
      ignored: Vec<u8>,
      #[allocative(visit = exclude_payload)]
      external: Vec<u8>,
    },
    #[allocative(skip)]
    Skipped(String),
  }
  let value = Value::Data {
    text: String::with_capacity(4096),
    ignored: Vec::with_capacity(8192),
    external: Vec::with_capacity(16384),
  };
  let (total, folded, warnings) = capture(&value);
  assert_eq!(total, size_of_val(&value) + 4096);
  assert!(!folded.contains(";ignored;"), "{folded}");
  assert_eq!(warnings.lines().count(), 1, "{warnings}");
  assert!(warnings.contains(";Data;external;opaque;"), "{warnings}");
  let (total, _, warnings) = capture(&Value::Skipped(String::with_capacity(4096)));
  assert_eq!(total, size_of::<Value>());
  assert!(warnings.is_empty(), "{warnings}");
}

#[test]
fn explicit_field_policy_takes_precedence_over_reflection() {
  #[derive(Allocative)]
  struct Value {
    text: String,
    #[allocative(visit = exclude_payload)]
    external: Vec<u8>,
  }
  let value = Value {
    text: String::with_capacity(1024),
    external: Vec::with_capacity(8192),
  };
  let (total, _, warnings) = capture(&value);
  assert_eq!(total, size_of_val(&value) + value.text.capacity());
  assert!(
    warnings.contains("Unobserved nested allocations"),
    "{warnings}"
  );
}

#[test]
fn unsupported_enum_reports_omitted_heap_instead_of_reading_inactive_fields() {
  macro_rules! opaque_enum {
    () => {
      enum Value {
        Empty,
        Data(String),
      }
    };
  }
  opaque_enum!();
  let value = Value::Data(String::with_capacity(4096));
  assert!(matches!(&value, Value::Data(text) if text.capacity() >= 4096));
  for value in [Value::Empty, value] {
    let (total, _, warnings) = capture(&value);
    assert_eq!(total, size_of_val(&value));
    assert!(
      warnings.contains("Unobserved nested allocations"),
      "{warnings}"
    );
  }
}

#[test]
fn driver_instruments_external_modules_after_cfg_without_confusing_discriminants() {
  use driver_enums::{Conditional, Generated};
  let values = [
    Conditional::Empty,
    Conditional::Text(String::with_capacity(4096)),
    Conditional::Named {
      bytes: Vec::with_capacity(8192),
    },
  ];
  for value in values {
    let heap = match &value {
      Conditional::Empty => 0,
      Conditional::Text(text) => text.capacity(),
      Conditional::Named { bytes } => bytes.capacity(),
    };
    let (total, _, warnings) = capture(&value);
    assert_eq!(total, size_of_val(&value) + heap);
    assert!(warnings.is_empty(), "{warnings}");
  }
  let generated = Generated::Text(String::with_capacity(1024));
  let Generated::Text(text) = &generated;
  let (total, _, warnings) = capture(&generated);
  assert_eq!(total, size_of_val(&generated) + text.capacity());
  assert!(warnings.is_empty(), "{warnings}");
}

#[test]
fn unsynchronized_uninitialized_and_dropped_storage_is_not_read() {
  let pointer = std::ptr::dangling::<String>();
  let (total, folded, warnings) = capture(&pointer);
  assert_eq!(total, size_of_val(&pointer));
  assert!(folded.starts_with("opaque;*const"), "{folded}");
  assert!(warnings.contains("*const"), "{warnings}");

  let cell = UnsafeCell::new(String::with_capacity(4096));
  let uninit = MaybeUninit::<String>::uninit();
  let mut dropped = ManuallyDrop::new(String::with_capacity(4096));
  // SAFETY: Drop once, without moving or reading the payload afterwards.
  unsafe { ManuallyDrop::drop(&mut dropped) };
  for (total, _, warnings) in [capture(&cell), capture(&uninit), capture(&dropped)] {
    assert_eq!(total, size_of::<String>());
    assert!(
      warnings.contains("Unobserved nested allocations"),
      "{warnings}"
    );
  }
}

#[test]
fn packed_fields_are_not_borrowed_at_an_invalid_alignment() {
  #[repr(C, packed)]
  struct Packed {
    byte: u8,
    text: String,
  }
  #[repr(C, align(16))]
  struct Root {
    byte: u8,
    packed: Packed,
  }
  let value = Root {
    byte: 0,
    packed: Packed {
      byte: 0,
      text: String::with_capacity(4096),
    },
  };
  assert_eq!(
    std::mem::offset_of!(Root, packed) + std::mem::offset_of!(Packed, text),
    2
  );
  let (total, _, warnings) = capture(&value);
  assert_eq!(total, size_of_val(&value));
  assert!(
    warnings.contains("Unobserved nested allocations"),
    "{warnings}"
  );
}

#[test]
fn array_lengths_with_const_expressions_keep_typed_field_adapters() {
  const COUNT: usize = 2;
  #[derive(Allocative)]
  struct Value {
    items: [String; COUNT + 1],
  }
  let value = Value {
    items: std::array::from_fn(|i| String::with_capacity(127 + i)),
  };
  let expected = size_of_val(&value) + value.items.iter().map(String::capacity).sum::<usize>();
  let (total, _, warnings) = capture(&value);
  assert_eq!(total, expected);
  assert!(warnings.is_empty(), "{warnings}");
}

#[test]
fn opaque_callbacks_keep_their_allocation_without_executing_or_following_captures() {
  let bytes = vec![0u8; 4096];
  let called = Arc::new(AtomicBool::new(false));
  let callback_called = Arc::clone(&called);
  let callback: Box<dyn Fn() -> usize + Send + Sync> = Box::new(move || {
    callback_called.store(true, Ordering::Relaxed);
    bytes.len()
  });
  let expected = size_of_val(&callback) + size_of_val(&*callback);
  let (total, _, warnings) = capture(&callback);
  assert_eq!(total, expected);
  assert!(
    warnings.contains("Unobserved nested allocations: dyn"),
    "{warnings}"
  );
  assert!(
    !warnings.contains("*const"),
    "callback payload was lost: {warnings}"
  );

  let callback: Arc<dyn Fn() -> usize + Send + Sync> = Arc::from(callback);
  let allocation = std::alloc::Layout::new::<[usize; 2]>()
    .extend(std::alloc::Layout::for_value(&*callback))
    .expect("callback layout")
    .0
    .pad_to_align()
    .size();
  let value = (callback.clone(), callback);
  assert_eq!(capture(&value).0, size_of_val(&value) + allocation);
  assert!(!called.load(Ordering::Relaxed));
}

#[test]
fn opaque_diagnostics_identify_each_field_without_changing_byte_weights() {
  #[derive(Allocative)]
  struct Fields {
    #[allocative(visit = exclude_payload)]
    first: Vec<u8>,
    #[allocative(visit = exclude_payload)]
    second: Vec<u8>,
  }
  let value = Fields {
    first: Vec::with_capacity(4096),
    second: Vec::with_capacity(8192),
  };
  let (total, folded, warnings) = capture(&value);
  assert_eq!(total, size_of_val(&value));
  let root = std::any::type_name::<Fields>();
  let name = std::any::type_name::<Vec<u8>>();
  assert_eq!(warnings.lines().count(), 2, "{warnings}");
  for field in ["first", "second"] {
    assert!(
      folded.contains(&format!(";{field};opaque;{name} {}", size_of::<Vec<u8>>())),
      "{folded}"
    );
    assert!(
      warnings.contains(&format!(
        "Unobserved nested allocations: {name} (path: {root};{field};opaque;{name})"
      )),
      "{warnings}"
    );
  }
}

#[test]
fn opaque_shared_callbacks_keep_the_owner_path_and_are_counted_once() {
  type Callback = dyn Fn(&str) -> usize + Send + Sync;
  struct Owners {
    first: Arc<Callback>,
    second: Arc<Callback>,
  }
  let bytes = vec![0u8; 4096];
  let callback: Arc<Callback> = Arc::new(move |text| bytes.len() + text.len());
  let allocation = std::alloc::Layout::new::<[usize; 2]>()
    .extend(std::alloc::Layout::for_value(&*callback))
    .expect("callback layout")
    .0
    .pad_to_align()
    .size();
  let value = Owners {
    first: callback.clone(),
    second: callback,
  };
  assert!(Arc::ptr_eq(&value.first, &value.second));
  for builder in [
    FlameGraphBuilder::with_shared_ownership(),
    FlameGraphBuilder::default(),
  ] {
    let (total, folded, warnings) = capture_in(builder, |visitor| value.visit_memory(visitor));
    assert_eq!(total, size_of_val(&value) + allocation);
    let name = std::any::type_name::<Callback>();
    assert!(folded.contains(&format!(";opaque;{name} ")), "{folded}");
    assert_eq!(warnings.lines().count(), 1, "{warnings}");
    assert!(
      warnings.contains(&format!(
        "(path: {};first;",
        std::any::type_name::<Owners>()
      )),
      "{warnings}"
    );
    assert!(!warnings.contains(";second;"), "{warnings}");
  }
}

#[test]
fn nested_optional_callbacks_count_slice_storage_and_deduplicate_opaque_payloads() {
  type Callback = dyn Fn(&str) -> usize + Send + Sync;
  let called = Arc::new(AtomicBool::new(false));
  let callback_called = called.clone();
  let bytes = vec![0u8; 4096];
  let callback: Arc<Callback> = Arc::new(move |text| {
    callback_called.store(true, Ordering::Relaxed);
    bytes.len() + text.len()
  });
  let allocation = std::alloc::Layout::new::<[usize; 2]>()
    .extend(std::alloc::Layout::for_value(&*callback))
    .expect("callback layout")
    .0
    .pad_to_align()
    .size();
  let value = Some(vec![None, Some(callback.clone()), Some(callback)].into_boxed_slice());
  let (total, folded, warnings) = capture(&value);
  assert_eq!(
    total,
    size_of_val(&value) + 3 * size_of::<Option<Arc<Callback>>>() + allocation
  );
  assert_eq!(warnings.lines().count(), 1, "{warnings}");
  assert!(folded.contains(";Some;"), "{folded}");
  assert!(
    warnings.contains(std::any::type_name::<Callback>()),
    "{warnings}"
  );
  assert!(!called.load(Ordering::Relaxed));
  let absent: Option<Box<[Option<Arc<Callback>>]>> = None;
  let (total, _, warnings) = capture(&absent);
  assert_eq!(total, size_of_val(&absent));
  assert!(warnings.is_empty(), "{warnings}");
}

#[test]
fn unknown_trait_objects_in_box_rc_and_arc_use_the_generic_opaque_fallback() {
  use std::{any::Any, rc::Rc};

  let payload = String::with_capacity(4096);
  let boxed: Box<dyn Any> = Box::new(payload);
  let (total, _, warnings) = capture(&boxed);
  assert_eq!(total, size_of_val(&boxed) + size_of::<String>());
  assert!(
    warnings.contains(std::any::type_name::<dyn Any>()),
    "{warnings}"
  );

  let shared: Rc<dyn Any> = Rc::from(boxed);
  let allocation = std::alloc::Layout::new::<[usize; 2]>()
    .extend(std::alloc::Layout::for_value(&*shared))
    .expect("shared payload layout")
    .0
    .pad_to_align()
    .size();
  let owners = (shared.clone(), shared);
  let (total, _, warnings) = capture(&owners);
  assert_eq!(total, size_of_val(&owners) + allocation);
  assert_eq!(warnings.lines().count(), 1, "{warnings}");

  let shared: Arc<dyn Any + Send + Sync> = Arc::new(String::with_capacity(4096));
  let allocation = std::alloc::Layout::new::<[usize; 2]>()
    .extend(std::alloc::Layout::for_value(&*shared))
    .expect("shared payload layout")
    .0
    .pad_to_align()
    .size();
  let owners = (shared.clone(), shared);
  let (total, folded, warnings) = capture(&owners);
  assert_eq!(total, size_of_val(&owners) + allocation);
  assert!(folded.contains(";0;"), "{folded}");
  assert!(
    warnings.contains(std::any::type_name::<dyn Any + Send + Sync>()),
    "{warnings}"
  );
  assert_eq!(warnings.lines().count(), 1, "{warnings}");
}

#[test]
fn ordered_collections_retain_index_allocations_when_empty_or_sparse() {
  let mut map: IndexMap<u64, String> = IndexMap::with_capacity(100_000);
  let (empty, folded, warnings) = capture(&map);
  assert!(empty > size_of_val(&map) + map.capacity() * size_of::<(u64, String)>());
  assert!(folded.contains(";raw_table;alloc "), "{folded}");
  assert!(warnings.is_empty(), "{warnings}");
  let text = String::with_capacity(4096);
  let heap = text.capacity();
  map.insert(1, text);
  assert_eq!(capture(&map).0, empty + heap);
  map.clear();
  assert_eq!(capture(&map).0, empty);

  let mut set: IndexSet<String> = IndexSet::with_capacity(100_000);
  let (empty, folded, warnings) = capture(&set);
  assert!(empty > size_of_val(&set) + set.capacity() * size_of::<String>());
  assert!(folded.contains(";raw_table;alloc "), "{folded}");
  assert!(warnings.is_empty(), "{warnings}");
  let text = String::with_capacity(8192);
  let heap = text.capacity();
  set.insert(text);
  assert_eq!(capture(&set).0, empty + heap);
  set.clear();
  assert_eq!(capture(&set).0, empty);

  let (total, folded, warnings) = capture(&IndexMap::<u64, String>::new());
  assert_eq!(total, size_of::<IndexMap<u64, String>>());
  assert!(!folded.contains(";raw_table;"), "{folded}");
  assert!(warnings.is_empty(), "{warnings}");
  let (total, folded, warnings) = capture(&IndexSet::<String>::new());
  assert_eq!(total, size_of::<IndexSet<String>>());
  assert!(!folded.contains(";raw_table;"), "{folded}");
  assert!(warnings.is_empty(), "{warnings}");
}

#[test]
fn spilled_small_vectors_keep_capacity_and_owned_payload_accounting() {
  let mut value: SmallVec<[String; 2]> = SmallVec::with_capacity(64);
  assert!(value.spilled());
  let expected = size_of_val(&value) + value.capacity() * size_of::<String>();
  let (total, folded, warnings) = capture(&value);
  assert_eq!(total, expected);
  assert!(folded.contains(";unused_capacity "), "{folded}");
  assert!(warnings.is_empty(), "{warnings}");

  let text = String::with_capacity(4096);
  let heap = text.capacity();
  value.push(text);
  assert_eq!(capture(&value).0, expected + heap);
  value.clear();
  assert!(value.spilled());
  assert_eq!(capture(&value).0, expected);

  let text = String::with_capacity(1024);
  let heap = text.capacity();
  value.push(text);
  value.shrink_to_fit();
  assert!(!value.spilled());
  let (total, _, warnings) = capture(&value);
  assert_eq!(total, size_of_val(&value) + heap);
  assert!(warnings.is_empty(), "{warnings}");

  let empty: SmallVec<[String; 2]> = SmallVec::new();
  assert_eq!(capture(&empty).0, size_of_val(&empty));
  let zeros: SmallVec<[(); 2]> = SmallVec::from_vec(vec![(); 4]);
  assert_eq!(capture(&zeros).0, size_of_val(&zeros));
}

#[test]
fn folded_labels_preserve_frame_boundaries_and_distinct_keys() {
  let mut parent = FlameGraph::default();
  parent.add_self(8);
  for (name, size) in [
    ("leaf;name", 1),
    ("leaf%3Bname", 2),
    ("child\nline\rreturn\ttab", 4),
    ("ordinary type::中文 with spaces", 16),
  ] {
    let mut child = FlameGraph::default();
    child.add_self(size);
    parent.add_child(Key::new(name), child);
  }
  let mut graph = FlameGraph::default();
  graph.add_child(Key::new("parent;[T; 2]"), parent);
  let folded = graph.write();
  let rows: std::collections::BTreeMap<_, _> = folded
    .lines()
    .map(|line| {
      let (path, weight) = line.rsplit_once(' ').expect("folded sample");
      assert!(!path.chars().any(|ch| ch.is_ascii_control()), "{path:?}");
      (path, weight.parse::<usize>().expect("byte weight"))
    })
    .collect();
  assert_eq!(rows.len(), 5, "{folded}");
  assert_eq!(rows["parent%3B[T%3B 2]"], 8);
  assert_eq!(rows["parent%3B[T%3B 2];leaf%3Bname"], 1);
  assert_eq!(rows["parent%3B[T%3B 2];leaf%253Bname"], 2);
  assert_eq!(rows["parent%3B[T%3B 2];child%0Aline%0Dreturn%09tab"], 4);
  assert_eq!(
    rows["parent%3B[T%3B 2];ordinary type::中文 with spaces"],
    16
  );
  assert_eq!(rows.values().sum::<usize>(), graph.total_size());
  assert!(rows.keys().all(|path| path.split(';').count() <= 2));
}

#[test]
fn small_vector_array_type_names_stay_in_one_folded_frame() {
  use cow_utils::CowUtils;

  let mut value: SmallVec<[String; 2]> = SmallVec::with_capacity(64);
  let text = String::with_capacity(4096);
  let heap = text.capacity();
  value.push(text);
  let (total, folded, warnings) = capture(&value);
  assert_eq!(
    total,
    size_of_val(&value) + value.capacity() * size_of::<String>() + heap
  );
  let type_name = std::any::type_name::<SmallVec<[String; 2]>>();
  assert!(type_name.contains(';'));
  for row in folded.lines() {
    let (path, _) = row.rsplit_once(' ').expect("folded sample");
    let first_frame = path.split(';').next().expect("root frame");
    assert_eq!(first_frame, type_name.cow_replace(';', "%3B"));
    assert!(
      first_frame.ends_with(">"),
      "incomplete type frame: {first_frame}"
    );
  }
  assert!(warnings.is_empty(), "{warnings}");
}

fn assert_retained_weak(value: &dyn Visit, expected: usize) {
  for builder in [
    FlameGraphBuilder::default(),
    FlameGraphBuilder::with_shared_ownership(),
  ] {
    let (total, folded, warnings) = capture_in(builder, |visitor| value.visit_memory(visitor));
    assert_eq!(total, expected, "{folded}");
    assert!(folded.contains(";retained "), "{folded}");
    assert!(
      !folded.contains("capacity"),
      "dropped payload was visited: {folded}"
    );
    assert!(warnings.is_empty(), "{warnings}");
  }
}

#[test]
#[allow(clippy::rc_buffer)]
fn weak_only_owners_count_the_backing_allocation_once_without_dropped_payloads() {
  let arc = Arc::new(String::with_capacity(8192));
  let weak = Arc::downgrade(&arc);
  let live = (weak.clone(), arc);
  let (total, _, warnings) = capture(&live);
  assert_eq!(
    total,
    size_of_val(&live) + 2 * size_of::<usize>() + size_of::<String>() + 8192
  );
  assert!(warnings.is_empty(), "{warnings}");
  drop(live);
  assert!(weak.upgrade().is_none());
  let pair = (weak.clone(), weak);
  assert_retained_weak(
    &pair,
    size_of_val(&pair) + 2 * size_of::<usize>() + size_of::<String>(),
  );

  let rc = std::rc::Rc::new(String::with_capacity(8192));
  let weak = std::rc::Rc::downgrade(&rc);
  let live = (rc, weak.clone());
  let (total, _, warnings) = capture(&live);
  assert_eq!(
    total,
    size_of_val(&live) + 2 * size_of::<usize>() + size_of::<String>() + 8192
  );
  assert!(warnings.is_empty(), "{warnings}");
  drop(live);
  assert!(weak.upgrade().is_none());
  let pair = (weak.clone(), weak);
  assert_retained_weak(
    &pair,
    size_of_val(&pair) + 2 * size_of::<usize>() + size_of::<String>(),
  );
}

#[test]
fn weak_only_dst_allocations_preserve_length_alignment_and_vtable_metadata() {
  #[repr(align(64))]
  struct Aligned(u8);

  let arc: Arc<dyn std::any::Any + Send + Sync> = Arc::new(Aligned(1));
  assert_eq!(arc.downcast_ref::<Aligned>().expect("aligned payload").0, 1);
  let weak = Arc::downgrade(&arc);
  drop(arc);
  assert_retained_weak(&weak, size_of_val(&weak) + 128);
  let rc: std::rc::Rc<dyn std::any::Any> = std::rc::Rc::new(Aligned(1));
  let weak = std::rc::Rc::downgrade(&rc);
  drop(rc);
  assert_retained_weak(&weak, size_of_val(&weak) + 128);

  let arc: Arc<[u64]> = Arc::from(vec![0; 128]);
  let weak = Arc::downgrade(&arc);
  drop(arc);
  assert_retained_weak(
    &weak,
    size_of_val(&weak) + 2 * size_of::<usize>() + 128 * size_of::<u64>(),
  );
  let rc: std::rc::Rc<str> = std::rc::Rc::from("x".repeat(4096));
  let weak = std::rc::Rc::downgrade(&rc);
  drop(rc);
  assert_retained_weak(&weak, size_of_val(&weak) + 2 * size_of::<usize>() + 4096);

  let arc: Arc<[u8]> = Arc::from([]);
  let weak = Arc::downgrade(&arc);
  drop(arc);
  assert_retained_weak(&weak, size_of_val(&weak) + 2 * size_of::<usize>());
  let rc = std::rc::Rc::new(());
  let weak = std::rc::Rc::downgrade(&rc);
  drop(rc);
  assert_retained_weak(&weak, size_of_val(&weak) + 2 * size_of::<usize>());
}

#[test]
fn empty_weak_pointers_do_not_invent_backing_allocations() {
  let arc = std::sync::Weak::<u64>::new();
  let rc = std::rc::Weak::<u64>::new();
  let empty_arc: std::sync::Weak<dyn std::any::Any + Send + Sync> = std::sync::Weak::<u64>::new();
  let empty_rc: std::rc::Weak<dyn std::any::Any> = std::rc::Weak::<u64>::new();
  for value in [&arc as &dyn Visit, &rc, &empty_arc, &empty_rc] {
    let (total, folded, warnings) = capture(value);
    assert_eq!(total, size_of_val(value));
    assert!(!folded.contains(";retained "), "{folded}");
    assert!(warnings.is_empty(), "{warnings}");
  }
}
