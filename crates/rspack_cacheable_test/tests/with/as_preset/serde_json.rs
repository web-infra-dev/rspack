use rspack_cacheable::{cacheable, from_bytes, to_bytes, with::AsPreset};
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
