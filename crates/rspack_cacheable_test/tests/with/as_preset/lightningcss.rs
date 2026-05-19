use lightningcss::targets::Browsers;
use rspack_cacheable::{enable_cacheable as cacheable, from_bytes, to_bytes, with::AsPreset};

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

#[test]
fn test_preset_lightningcss_multiple_browsers() {
  let config = Config {
    browsers: Browsers {
      chrome: Some(222),
      firefox: Some(123),
      safari: Some(17),
      edge: Some(127),
      ..Default::default()
    },
  };

  let bytes = to_bytes(&config, &()).unwrap();
  let new_config: Config = from_bytes(&bytes, &()).unwrap();

  assert_eq!(config.browsers.chrome, new_config.browsers.chrome);
  assert_eq!(config.browsers.firefox, new_config.browsers.firefox);
  assert_eq!(config.browsers.safari, new_config.browsers.safari);
  assert_eq!(config.browsers.edge, new_config.browsers.edge);
}

#[test]
fn test_preset_lightningcss_large_versions() {
  let config = Config {
    browsers: Browsers {
      chrome: Some(999_999),
      firefox: Some(123_456),
      ..Default::default()
    },
  };

  let bytes = to_bytes(&config, &()).unwrap();
  let new_config: Config = from_bytes(&bytes, &()).unwrap();

  assert_eq!(config.browsers.chrome, new_config.browsers.chrome);
  assert_eq!(config.browsers.firefox, new_config.browsers.firefox);
}

#[test]
fn test_preset_lightningcss_all_browsers() {
  let config = Config {
    browsers: Browsers {
      android: Some(108),
      chrome: Some(222),
      edge: Some(127),
      firefox: Some(123),
      ie: Some(11),
      ios_saf: Some(16),
      opera: Some(107),
      safari: Some(17),
      samsung: Some(20),
    },
  };

  let bytes = to_bytes(&config, &()).unwrap();
  let new_config: Config = from_bytes(&bytes, &()).unwrap();

  assert_eq!(config.browsers.android, new_config.browsers.android);
  assert_eq!(config.browsers.chrome, new_config.browsers.chrome);
  assert_eq!(config.browsers.edge, new_config.browsers.edge);
  assert_eq!(config.browsers.firefox, new_config.browsers.firefox);
  assert_eq!(config.browsers.ie, new_config.browsers.ie);
  assert_eq!(config.browsers.ios_saf, new_config.browsers.ios_saf);
  assert_eq!(config.browsers.opera, new_config.browsers.opera);
  assert_eq!(config.browsers.safari, new_config.browsers.safari);
  assert_eq!(config.browsers.samsung, new_config.browsers.samsung);
}

#[test]
fn test_preset_lightningcss_empty_browsers() {
  let config = Config {
    browsers: Browsers::default(),
  };

  let bytes = to_bytes(&config, &()).unwrap();
  let new_config: Config = from_bytes(&bytes, &()).unwrap();

  assert_eq!(config.browsers.chrome, new_config.browsers.chrome);
  assert_eq!(config.browsers.firefox, new_config.browsers.firefox);
  assert_eq!(config.browsers.safari, new_config.browsers.safari);
}
