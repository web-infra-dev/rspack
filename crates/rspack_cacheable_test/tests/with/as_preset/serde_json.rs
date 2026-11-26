use rspack_cacheable::{enable_cacheable as cacheable, from_bytes, to_bytes, with::AsPreset};
use serde_json::Value;

#[cacheable]
#[derive(Debug, PartialEq, Eq)]
struct Module {
  #[cacheable(with=AsPreset)]
  options: Value,
}

#[test]
fn test_preset_serde_json() {
  let module = Module {
    options: serde_json::from_str("{\"id\":1}").unwrap(),
  };

  let bytes = to_bytes(&module, &()).unwrap();
  let new_module: Module = from_bytes(&bytes, &()).unwrap();
  assert_eq!(module, new_module);
}

#[cacheable]
#[derive(Debug)]
struct Description {
  #[cacheable(with=AsPreset)]
  json: std::sync::Arc<Value>,
}

#[test]
fn test_preset_serde_json_arc_is_shared() {
  let first = Description {
    json: std::sync::Arc::new(serde_json::from_str("{\"name\":\"shared\"}").unwrap()),
  };
  let second = Description {
    json: std::sync::Arc::new(serde_json::from_str("{\"name\":\"shared\"}").unwrap()),
  };
  let other = Description {
    json: std::sync::Arc::new(serde_json::from_str("{\"name\":\"other\"}").unwrap()),
  };

  let first: Description = from_bytes(&to_bytes(&first, &()).unwrap(), &()).unwrap();
  let second: Description = from_bytes(&to_bytes(&second, &()).unwrap(), &()).unwrap();
  let other: Description = from_bytes(&to_bytes(&other, &()).unwrap(), &()).unwrap();

  assert_eq!(*first.json, serde_json::json!({ "name": "shared" }));
  assert!(std::sync::Arc::ptr_eq(&first.json, &second.json));
  assert!(!std::sync::Arc::ptr_eq(&first.json, &other.json));
  assert_eq!(*other.json, serde_json::json!({ "name": "other" }));
}
