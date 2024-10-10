use rspack_cacheable::{cacheable, from_bytes, to_bytes, with::AsPreset};
use swc_core::ecma::atoms::Atom;

#[cacheable]
#[derive(Debug, PartialEq, Eq)]
struct Module {
  #[cacheable(with=AsPreset)]
  id: Atom,
}

#[test]
fn test_preset_swc() {
  let module = Module {
    id: Atom::new("abc"),
  };

  let bytes = to_bytes(&module, &()).unwrap();
  let new_module: Module = from_bytes(&bytes, &()).unwrap();
  assert_eq!(module, new_module);
}
