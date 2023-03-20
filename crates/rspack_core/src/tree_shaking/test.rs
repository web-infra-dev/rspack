#[cfg(test)]
mod test_side_effects {
  use std::path::PathBuf;

  use crate::tree_shaking::visitor::{get_side_effects_from_package_json, SideEffects};

  fn wrap_get_side_effects_from_package_json(
    side_effects_config: Vec<&str>,
    relative_path: &str,
  ) -> bool {
    assert!(!side_effects_config.is_empty());
    let relative_path = PathBuf::from(relative_path);
    let side_effects = if side_effects_config.len() > 1 {
      SideEffects::Array(
        side_effects_config
          .into_iter()
          .map(String::from)
          .collect::<Vec<_>>(),
      )
    } else {
      SideEffects::String((&side_effects_config[0]).to_string())
    };

    get_side_effects_from_package_json(side_effects, relative_path)
  }
  #[test]
  fn cases() {
    assert!(wrap_get_side_effects_from_package_json(
      vec!["./src/**/*.js"],
      "./src/x/y/z.js"
    ));

    assert!(!wrap_get_side_effects_from_package_json(
      vec!["./src/**/*.js"],
      "./x.js"
    ));

    assert!(wrap_get_side_effects_from_package_json(
      vec!["./**/src/x/y/z.js"],
      "./src/x/y/z.js"
    ));

    // 				"./src/x/y/z.js",
    // 				"./src/**/z.js",
    assert!(wrap_get_side_effects_from_package_json(
      vec!["./src/**/z.js"],
      "./src/x/y/z.js"
    ));

    // 				"./src/x/y/z.js",
    // 				"./**/x/**/z.js",

    assert!(wrap_get_side_effects_from_package_json(
      vec!["./**/x/**/z.js"],
      "./src/x/y/z.js"
    ));

    // 				"./src/x/y/z.js",
    // 				"./**/src/**",

    assert!(wrap_get_side_effects_from_package_json(
      vec!["./**/src/**"],
      "./src/x/y/z.js"
    ));

    // 				"./src/x/y/z.js",
    // 				"./**/src/*",

    assert!(!wrap_get_side_effects_from_package_json(
      vec!["./src/x/y/z.js"],
      "./**/src/*"
    ));

    // 				"./src/x/y/z.js",
    // 				"*.js",
    assert!(wrap_get_side_effects_from_package_json(
      vec!["*.js"],
      "./src/x/y/z.js"
    ));

    // 				"./src/x/y/z.js",
    // 				"x/**/z.js",

    assert!(!wrap_get_side_effects_from_package_json(
      vec!["./src/x/y/z.js"],
      "x/**/z.js"
    ));

    // 				"./src/x/y/z.js",
    // 				"src/**/z.js",

    assert!(wrap_get_side_effects_from_package_json(
      vec!["./src/**/z.js"],
      "./src/x/y/z.js"
    ));
    // 				"./src/x/y/z.js",
    // 				"src/**/{x,y,z}.js",
    assert!(wrap_get_side_effects_from_package_json(
      vec!["src/**/{x,y,z}.js"],
      "./src/x/y/z.js"
    ));
    // 				"./src/x/y/z.js",
    // 				"src/**/[x-z].js",
    assert!(wrap_get_side_effects_from_package_json(
      vec!["./src/**/[x-z].js"],
      "./src/x/y/z.js"
    ));
    // 		const array = ["./src/**/*.js", "./dirty.js"];
    assert!(wrap_get_side_effects_from_package_json(
      vec!["./src/**/*.js", "./dirty.js"],
      "./src/x/y/z.js"
    ));
    assert!(wrap_get_side_effects_from_package_json(
      vec!["./src/**/*.js", "./dirty.js"],
      "./dirty.js"
    ));
    assert!(!wrap_get_side_effects_from_package_json(
      vec!["./src/**/*.js", "./dirty.js"],
      "./clean.js"
    ));
  }
}
