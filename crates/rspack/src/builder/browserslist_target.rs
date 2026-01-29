use std::collections::HashMap;

use super::target::TargetProperties;

// Macro for defining HashMaps with less boilerplate
macro_rules! hashmap {
    ($( $key: expr => $val: expr ),* $(,)?) => {{
         let mut map = ::std::collections::HashMap::new();
         $( map.insert($key, $val); )*
         map
    }}
}

/// Represents a version requirement, which can be a major version or a major-minor version pair.
enum VersionRequirement {
  Major(u32),
  MajorMinor(u32, u32),
}

impl VersionRequirement {
  /// Checks if the given major and minor versions meet the version requirement.
  fn matches(&self, major: u32, minor: u32) -> bool {
    match self {
      VersionRequirement::Major(r_major) => major >= *r_major,
      VersionRequirement::MajorMinor(r_major, r_minor) => {
        if major == *r_major {
          minor >= *r_minor
        } else {
          major > *r_major
        }
      }
    }
  }
}

/// Parses a version string into a (major, minor) tuple.
fn parse_version(version: &str) -> (u32, u32) {
  let parts: Vec<&str> = version.split('.').collect();
  let major = parts.first().and_then(|v| v.parse().ok()).unwrap_or(0);
  let minor = parts.get(1).and_then(|v| v.parse().ok()).unwrap_or(0);
  (major, minor)
}

/// Checks if all browsers in the list meet the version requirements.
fn raw_checker(browsers: &[String], versions: &HashMap<&str, VersionRequirement>) -> bool {
  browsers.iter().all(|b| {
    let mut parts = b.split_whitespace();
    let name = parts.next().unwrap_or("");
    let version_str = parts.next().unwrap_or("");
    if name.is_empty() {
      return false;
    }
    let required = match versions.get(name) {
      Some(r) => r,
      None => return false,
    };
    let (major, minor) = if version_str == "TP" {
      (u32::MAX, u32::MAX)
    } else if version_str.contains('-') {
      let first = version_str.split('-').next().unwrap_or("0");
      parse_version(first)
    } else {
      parse_version(version_str)
    };
    required.matches(major, minor)
  })
}

/// Resolves target properties based on the provided browser list.
pub fn resolve(browsers: &[String]) -> TargetProperties {
  let any_node = browsers.iter().any(|b| b.starts_with("node "));
  let any_browser = browsers.iter().any(|b| !b.starts_with("node "));

  let browser_property = if !any_browser {
    Some(false)
  } else if any_node {
    None
  } else {
    Some(true)
  };
  let node_property = if !any_node {
    Some(false)
  } else if any_browser {
    None
  } else {
    Some(true)
  };

  let es6_dynamic_import_versions = hashmap! {
      "chrome" => VersionRequirement::Major(63),
      "and_chr" => VersionRequirement::Major(63),
      "edge" => VersionRequirement::Major(79),
      "firefox" => VersionRequirement::Major(67),
      "and_ff" => VersionRequirement::Major(67),
      "opera" => VersionRequirement::Major(50),
      "op_mob" => VersionRequirement::Major(46),
      "safari" => VersionRequirement::MajorMinor(11, 1),
      "ios_saf" => VersionRequirement::MajorMinor(11, 3),
      "samsung" => VersionRequirement::MajorMinor(8, 2),
      "android" => VersionRequirement::Major(63),
      "and_qq" => VersionRequirement::MajorMinor(10, 4),
      "baidu" => VersionRequirement::MajorMinor(13, 18),
      "and_uc" => VersionRequirement::MajorMinor(15, 5),
      "kaios" => VersionRequirement::MajorMinor(3, 0),
      "node" => VersionRequirement::MajorMinor(12, 17),
  };

  let es6_dynamic_import = raw_checker(browsers, &es6_dynamic_import_versions);

  TargetProperties {
    r#const: Some(raw_checker(
      browsers,
      &hashmap! {
          "chrome" => VersionRequirement::Major(49),
          "and_chr" => VersionRequirement::Major(49),
          "edge" => VersionRequirement::Major(12),
          "firefox" => VersionRequirement::Major(36),
          "and_ff" => VersionRequirement::Major(36),
          "opera" => VersionRequirement::Major(36),
          "op_mob" => VersionRequirement::Major(36),
          "safari" => VersionRequirement::MajorMinor(10, 0),
          "ios_saf" => VersionRequirement::MajorMinor(10, 0),
          "samsung" => VersionRequirement::MajorMinor(5, 0),
          "android" => VersionRequirement::Major(37),
          "and_qq" => VersionRequirement::MajorMinor(10, 4),
          "baidu" => VersionRequirement::MajorMinor(13, 18),
          "and_uc" => VersionRequirement::MajorMinor(12, 12),
          "kaios" => VersionRequirement::MajorMinor(2, 5),
          "node" => VersionRequirement::MajorMinor(6, 0),
      },
    )),
    arrow_function: Some(raw_checker(
      browsers,
      &hashmap! {
          "chrome" => VersionRequirement::Major(45),
          "and_chr" => VersionRequirement::Major(45),
          "edge" => VersionRequirement::Major(12),
          "firefox" => VersionRequirement::Major(39),
          "and_ff" => VersionRequirement::Major(39),
          "opera" => VersionRequirement::Major(32),
          "op_mob" => VersionRequirement::Major(32),
          "safari" => VersionRequirement::MajorMinor(10, 0),
          "ios_saf" => VersionRequirement::MajorMinor(10, 0),
          "samsung" => VersionRequirement::MajorMinor(5, 0),
          "android" => VersionRequirement::Major(45),
          "and_qq" => VersionRequirement::MajorMinor(10, 4),
          "baidu" => VersionRequirement::MajorMinor(7, 12),
          "and_uc" => VersionRequirement::MajorMinor(12, 12),
          "kaios" => VersionRequirement::MajorMinor(2, 5),
          "node" => VersionRequirement::MajorMinor(6, 0),
      },
    )),
    for_of: Some(raw_checker(
      browsers,
      &hashmap! {
          "chrome" => VersionRequirement::Major(38),
          "and_chr" => VersionRequirement::Major(38),
          "edge" => VersionRequirement::Major(12),
          "firefox" => VersionRequirement::Major(51),
          "and_ff" => VersionRequirement::Major(51),
          "opera" => VersionRequirement::Major(25),
          "op_mob" => VersionRequirement::Major(25),
          "safari" => VersionRequirement::MajorMinor(7, 0),
          "ios_saf" => VersionRequirement::MajorMinor(7, 0),
          "samsung" => VersionRequirement::MajorMinor(3, 0),
          "android" => VersionRequirement::Major(38),
          "kaios" => VersionRequirement::MajorMinor(3, 0),
          "node" => VersionRequirement::MajorMinor(0, 12),
      },
    )),
    destructuring: Some(raw_checker(
      browsers,
      &hashmap! {
          "chrome" => VersionRequirement::Major(49),
          "and_chr" => VersionRequirement::Major(49),
          "edge" => VersionRequirement::Major(14),
          "firefox" => VersionRequirement::Major(41),
          "and_ff" => VersionRequirement::Major(41),
          "opera" => VersionRequirement::Major(36),
          "op_mob" => VersionRequirement::Major(36),
          "safari" => VersionRequirement::MajorMinor(8, 0),
          "ios_saf" => VersionRequirement::MajorMinor(8, 0),
          "samsung" => VersionRequirement::MajorMinor(5, 0),
          "android" => VersionRequirement::Major(49),
          "kaios" => VersionRequirement::MajorMinor(2, 5),
          "node" => VersionRequirement::MajorMinor(6, 0),
      },
    )),
    big_int_literal: Some(raw_checker(
      browsers,
      &hashmap! {
          "chrome" => VersionRequirement::Major(67),
          "and_chr" => VersionRequirement::Major(67),
          "edge" => VersionRequirement::Major(79),
          "firefox" => VersionRequirement::Major(68),
          "and_ff" => VersionRequirement::Major(68),
          "opera" => VersionRequirement::Major(54),
          "op_mob" => VersionRequirement::Major(48),
          "safari" => VersionRequirement::MajorMinor(14, 0),
          "ios_saf" => VersionRequirement::MajorMinor(14, 0),
          "samsung" => VersionRequirement::MajorMinor(9, 2),
          "android" => VersionRequirement::Major(67),
          "and_qq" => VersionRequirement::MajorMinor(13, 1),
          "baidu" => VersionRequirement::MajorMinor(13, 18),
          "and_uc" => VersionRequirement::MajorMinor(15, 5),
          "kaios" => VersionRequirement::MajorMinor(3, 0),
          "node" => VersionRequirement::MajorMinor(10, 4),
      },
    )),
    module: Some(raw_checker(
      browsers,
      &hashmap! {
          "chrome" => VersionRequirement::Major(61),
          "and_chr" => VersionRequirement::Major(61),
          "edge" => VersionRequirement::Major(16),
          "firefox" => VersionRequirement::Major(60),
          "and_ff" => VersionRequirement::Major(60),
          "opera" => VersionRequirement::Major(48),
          "op_mob" => VersionRequirement::Major(45),
          "safari" => VersionRequirement::MajorMinor(10, 1),
          "ios_saf" => VersionRequirement::MajorMinor(10, 3),
          "samsung" => VersionRequirement::MajorMinor(8, 0),
          "android" => VersionRequirement::Major(61),
          "and_qq" => VersionRequirement::MajorMinor(10, 4),
          "baidu" => VersionRequirement::MajorMinor(13, 18),
          "and_uc" => VersionRequirement::MajorMinor(15, 5),
          "kaios" => VersionRequirement::MajorMinor(3, 0),
          "node" => VersionRequirement::MajorMinor(12, 17),
      },
    )),
    dynamic_import: Some(es6_dynamic_import),
    dynamic_import_in_worker: Some(es6_dynamic_import && !any_node),
    global_this: Some(raw_checker(
      browsers,
      &hashmap! {
          "chrome" => VersionRequirement::Major(71),
          "and_chr" => VersionRequirement::Major(71),
          "edge" => VersionRequirement::Major(79),
          "firefox" => VersionRequirement::Major(65),
          "and_ff" => VersionRequirement::Major(65),
          "opera" => VersionRequirement::Major(58),
          "op_mob" => VersionRequirement::Major(50),
          "safari" => VersionRequirement::MajorMinor(12, 1),
          "ios_saf" => VersionRequirement::MajorMinor(12, 2),
          "samsung" => VersionRequirement::MajorMinor(10, 1),
          "android" => VersionRequirement::Major(71),
          "kaios" => VersionRequirement::MajorMinor(3, 0),
          "node" => VersionRequirement::Major(12),
      },
    )),
    optional_chaining: Some(raw_checker(
      browsers,
      &hashmap! {
          "chrome" => VersionRequirement::Major(80),
          "and_chr" => VersionRequirement::Major(80),
          "edge" => VersionRequirement::Major(80),
          "firefox" => VersionRequirement::Major(74),
          "and_ff" => VersionRequirement::Major(79),
          "opera" => VersionRequirement::Major(67),
          "op_mob" => VersionRequirement::Major(64),
          "safari" => VersionRequirement::MajorMinor(13, 1),
          "ios_saf" => VersionRequirement::MajorMinor(13, 4),
          "samsung" => VersionRequirement::Major(13),
          "android" => VersionRequirement::Major(80),
          "kaios" => VersionRequirement::MajorMinor(3, 0),
          "node" => VersionRequirement::Major(14),
      },
    )),
    template_literal: Some(raw_checker(
      browsers,
      &hashmap! {
          "chrome" => VersionRequirement::Major(41),
          "and_chr" => VersionRequirement::Major(41),
          "edge" => VersionRequirement::Major(13),
          "firefox" => VersionRequirement::Major(34),
          "and_ff" => VersionRequirement::Major(34),
          "opera" => VersionRequirement::Major(29),
          "op_mob" => VersionRequirement::Major(64),
          "safari" => VersionRequirement::MajorMinor(9, 1),
          "ios_saf" => VersionRequirement::Major(9),
          "samsung" => VersionRequirement::Major(4),
          "android" => VersionRequirement::Major(41),
          "and_qq" => VersionRequirement::MajorMinor(10, 4),
          "baidu" => VersionRequirement::MajorMinor(7, 12),
          "and_uc" => VersionRequirement::MajorMinor(12, 12),
          "kaios" => VersionRequirement::MajorMinor(2, 5),
          "node" => VersionRequirement::Major(4),
      },
    )),
    async_function: Some(raw_checker(
      browsers,
      &hashmap! {
          "chrome" => VersionRequirement::Major(55),
          "and_chr" => VersionRequirement::Major(55),
          "edge" => VersionRequirement::Major(15),
          "firefox" => VersionRequirement::Major(52),
          "and_ff" => VersionRequirement::Major(52),
          "opera" => VersionRequirement::Major(42),
          "op_mob" => VersionRequirement::Major(42),
          "safari" => VersionRequirement::Major(11),
          "ios_saf" => VersionRequirement::Major(11),
          "samsung" => VersionRequirement::MajorMinor(6, 2),
          "android" => VersionRequirement::Major(55),
          "and_qq" => VersionRequirement::MajorMinor(13, 1),
          "baidu" => VersionRequirement::MajorMinor(13, 18),
          "and_uc" => VersionRequirement::MajorMinor(15, 5),
          "kaios" => VersionRequirement::Major(3),
          "node" => VersionRequirement::MajorMinor(7, 6),
      },
    )),
    browser: browser_property,
    electron: Some(false),
    node: node_property,
    nwjs: Some(false),
    web: browser_property,
    webworker: Some(false),

    document: browser_property,
    fetch_wasm: browser_property,
    global: node_property,
    import_scripts: Some(false),
    import_scripts_in_worker: Some(true),
    node_builtins: node_property,
    node_prefix_for_core_modules: Some(
      node_property.unwrap_or(false)
        && !browsers.iter().any(|b| b.starts_with("node 15"))
        && raw_checker(
          browsers,
          &hashmap! { "node" => VersionRequirement::MajorMinor(14, 18) },
        ),
    ),
    require: node_property,
    ..Default::default()
  }
}

#[cfg(test)]
mod tests {
  use insta::assert_debug_snapshot;
  use rspack_browserslist::load_browserslist;

  use super::*;
  #[test]
  fn test() {
    let context = std::env::current_dir()
      .unwrap()
      .to_str()
      .unwrap()
      .to_string();

    // Example: Load browsers list, pass query string or None for default config
    let browsers =
      load_browserslist(Some("last 2 versions, not dead"), &context).unwrap_or_default();

    println!("browsers: {browsers:?}");

    // Resolve target properties
    let properties = resolve(&browsers);

    println!("prop: {properties:#?}")
  }

  #[test]
  fn test_browserslist_targets_snapshots() {
    let tests = vec![
      vec!["ie 11"],
      vec!["ie_mob 11"],
      vec!["edge 79"],
      vec!["android 4"],
      vec!["android 4.1"],
      vec!["android 4.4.3-4.4.4"],
      vec!["android 81"],
      vec!["chrome 80"],
      vec!["and_chr 80"],
      vec!["firefox 68"],
      vec!["and_ff 68"],
      vec!["opera 54"],
      vec!["op_mob 54"],
      vec!["safari 10"],
      vec!["safari TP"],
      vec!["safari 11"],
      vec!["safari 12.0"],
      vec!["safari 12.1"],
      vec!["safari 13"],
      vec!["ios_saf 12.0-12.1"],
      vec!["samsung 4"],
      vec!["samsung 9.2"],
      vec!["samsung 11.1-11.2"],
      vec!["op_mini all"],
      vec!["bb 10"],
      vec!["node 0.10.0"],
      vec!["node 0.12.0"],
      vec!["node 10.0.0"],
      vec!["node 10.17.0"],
      vec!["node 12.19.0"],
      vec!["and_uc 12.12"],
      vec!["and_qq 10.4"],
      vec!["kaios 2.5"],
      vec!["baidu 7.12"],
      vec!["firefox 80", "chrome 80"],
      vec!["chrome 80", "node 12.19.0"],
      vec!["unknown 50"],
    ];

    let mut results = vec![];

    for test_case in tests {
      let input: Vec<String> = test_case.iter().map(|s| (*s).to_string()).collect();

      let targets = resolve(&input);

      results.push((test_case, targets));
    }

    assert_debug_snapshot!(results);
  }
}
