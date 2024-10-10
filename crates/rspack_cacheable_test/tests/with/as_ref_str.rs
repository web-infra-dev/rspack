use rspack_cacheable::cacheable;

#[cacheable]
#[derive(Debug, PartialEq, Eq)]
enum ModuleType {
  NormalModule,
  ContextModule,
}

#[cacheable]
#[derive(Debug, PartialEq, Eq)]
struct Module {
  module_type: ModuleType,
}

#[test]
fn test_as_ref_string() {
  let module = Module {
    module_type: ModuleType::NormalModule,
  };

  let bytes = rspack_cacheable::to_bytes(&module, &()).unwrap();
  let new_module: Module = rspack_cacheable::from_bytes(&bytes, &()).unwrap();
  assert_eq!(module, new_module);
}
