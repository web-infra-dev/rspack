// use crate::{helper::make_relative_from, rst::RstBuilder};
// use node_binding::{normalize_bundle_options, RawOptions};
use std::path::Path;

/// Helper only for rspack
#[allow(unused)]
#[tokio::main]
pub async fn test_fixture(fixture_path: &Path) {
  // let options = TestOptions::from_fixture(cur_dir).into();
  // let options = normalize_bundle_options(RawOptions::from_fixture(fixture_path))
  //   .unwrap_or_else(|_| panic!("failed to normalize in fixtrue {:?}", fixture_path));
  // let output_path = options.output.path.clone();

  // let mut compiler = rspack::rspack(options, Default::default());

  // compiler
  //   .run()
  //   .await
  //   .unwrap_or_else(|_| panic!("failed to compile in fixtrue {:?}", fixture_path));

  // let output_name = make_relative_from(Path::new(&output_path), fixture_path);
  // let rst = RstBuilder::default()
  //   .fixture(PathBuf::from(fixture_path))
  //   .actual(output_name)
  //   .build()
  //   .unwrap();

  // rst.assert();
}

// pub trait RawOptionsTestExt {
//   fn from_fixture(fixture_path: &Path) -> Self;
// }

// impl RawOptionsTestExt for RawOptions {
//   fn from_fixture(fixture_path: &Path) -> Self {
//     let pkg_path = fixture_path.join("rspack.config.json");
//     let mut options = {
//       if pkg_path.exists() {
//         let pkg_content = std::fs::read_to_string(pkg_path).unwrap();
//         let options: RawOptions = serde_json::from_str(&pkg_content).unwrap();
//         options
//       } else {
//         RawOptions {
//           entries: HashMap::from([(
//             "main".to_string(),
//             fixture_path.join("index.js").to_str().unwrap().to_string(),
//           )]),
//           ..Default::default()
//         }
//       }
//     };
//     assert!(
//       options.root.is_none(),
//       "You should not specify `root` in config. It would probably resolve to a wrong path"
//     );
//     options.root = Some(fixture_path.to_str().unwrap().to_string());
//     options
//   }
// }
