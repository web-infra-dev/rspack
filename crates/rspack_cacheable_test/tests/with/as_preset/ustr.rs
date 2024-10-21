use rspack_cacheable::{cacheable, from_bytes, to_bytes, with::AsPreset};
use ustr::Ustr;

#[cacheable]
#[derive(Debug, PartialEq, Eq)]
struct Module {
  #[cacheable(with=AsPreset)]
  id: Ustr,
}

#[test]
fn test_preset_ustr() {
  let module = Module {
    id: Ustr::from("abc"),
  };

  let bytes = to_bytes(&module, &()).unwrap();
  let new_module: Module = from_bytes(&bytes, &()).unwrap();
  assert_eq!(module, new_module);
}
