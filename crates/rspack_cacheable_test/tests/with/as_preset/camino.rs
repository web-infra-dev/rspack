use camino::Utf8PathBuf;
use rspack_cacheable::{cacheable, from_bytes, to_bytes, with::AsPreset};

#[cacheable]
#[derive(Debug, PartialEq, Eq)]
struct Module {
  #[cacheable(with=AsPreset)]
  path: Utf8PathBuf,
}

#[test]
fn test_preset_camino() {
  let module = Module {
    path: Utf8PathBuf::from("/home/user"),
  };

  let bytes = to_bytes(&module, &()).unwrap();
  let new_module: Module = from_bytes(&bytes, &()).unwrap();
  assert_eq!(module, new_module);
}
