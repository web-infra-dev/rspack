use rspack_cacheable::{Error, enable_cacheable as cacheable, with::Unsupported};

struct UnCacheable;

#[cacheable]
struct App {
  #[cacheable(with=Unsupported)]
  info: UnCacheable,
}

#[test]
fn test_unsupport() {
  let app = App { info: UnCacheable };

  assert!(matches!(
    rspack_cacheable::to_bytes(&app, &()),
    Err(Error::UnsupportedField)
  ));
}
