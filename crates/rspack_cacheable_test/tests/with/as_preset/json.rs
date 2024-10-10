use json::JsonValue;
use rspack_cacheable::{cacheable, from_bytes, to_bytes, with::AsPreset};

#[cacheable]
#[derive(Debug, PartialEq, Eq)]
struct Module {
  #[cacheable(with=AsPreset)]
  options: JsonValue,
}

#[test]
fn test_preset_json() {
  let module = Module {
    options: json::parse("{\"id\":1}").unwrap(),
  };

  let bytes = to_bytes(&module, &()).unwrap();
  let new_module: Module = from_bytes(&bytes, &()).unwrap();
  assert_eq!(module, new_module);
}
