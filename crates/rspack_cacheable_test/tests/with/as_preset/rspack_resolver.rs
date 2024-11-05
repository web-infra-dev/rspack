use rspack_cacheable::{
  cacheable, from_bytes, to_bytes,
  with::{AsCacheable, AsOption, AsPreset, AsTuple2, AsVec},
};
use rspack_resolver::{Alias, AliasValue};

#[cacheable]
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct ResolverOption {
  #[cacheable(with=AsOption<AsVec<AsTuple2<AsCacheable, AsVec<AsPreset>>>>)]
  alias: Option<Alias>,
}

#[test]
fn test_preset_rspack_resolver() {
  let option = ResolverOption {
    alias: Some(vec![
      (
        String::from("@"),
        vec![AliasValue::Path(String::from("./src"))],
      ),
      (String::from("ignore"), vec![AliasValue::Ignore]),
      (
        String::from("components"),
        vec![
          AliasValue::Path(String::from("./components")),
          AliasValue::Path(String::from("./src")),
          AliasValue::Ignore,
        ],
      ),
    ]),
  };

  let bytes = to_bytes(&option, &()).unwrap();
  let new_option: ResolverOption = from_bytes(&bytes, &()).unwrap();
  assert_eq!(option, new_option);
}
