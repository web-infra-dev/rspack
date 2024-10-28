use lightningcss::targets::Browsers;
use rspack_cacheable::{cacheable, from_bytes, to_bytes, with::AsPreset};

#[cacheable]
struct Config {
  #[cacheable(with=AsPreset)]
  browsers: Browsers,
}

#[test]
fn test_preset_lightningcss() {
  let config = Config {
    browsers: Browsers {
      chrome: Some(222),
      ..Default::default()
    },
  };

  let bytes = to_bytes(&config, &()).unwrap();
  let new_config: Config = from_bytes(&bytes, &()).unwrap();
  assert_eq!(config.browsers.chrome, new_config.browsers.chrome);
}
