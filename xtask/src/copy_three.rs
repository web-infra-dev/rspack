use std::{fs::File, io::Write, path::PathBuf};

use xshell::{cmd, Shell};

// mkdir -p benchcases/three/src
// echo > benchcases/three/src/entry.js
// for i in 1 2 3 4 5 6 7 8 9 10; do test -d "benchcases/three/src/copy$$i" || cp -r examples/.three/src "benchcases/three/src/copy$$i"; done
// for i in 1 2 3 4 5 6 7 8 9 10; do echo "import * as copy$$i from './copy$$i/Three.js'; export {copy$$i}" >> benchcases/three/src/entry.js; done
// echo "module.exports = {mode: 'development',entry: {index: {import: ['./src/entry.js']}}};" > benchcases/three/test.config.js
// echo "module.exports = {mode: 'development',entry: {index: ['./benchcases/three/src/entry.js']},devtool: 'eval',cache: {type: 'filesystem'}}" > benchcases/three/webpack.config.js

pub fn copy_three(num: usize) {
  let sh = Shell::new().unwrap();
  println!("{:?}", sh.current_dir());
  sh.change_dir(PathBuf::from(env!("CARGO_WORKSPACE_DIR")));

  cmd!(sh, "mkdir -p benchcases/three/src").run().unwrap();
  for i in 1..=num {
    let ii = i.to_string();
    // let res = format!("");
    cmd!(
      sh,
      "cp -r examples/.three/src benchcases/three/src/copy{ii}"
    )
    .run()
    .unwrap();
  }

  // entry.js

  let mut entry = String::new();
  for i in 1..=num {
    entry += &format!(
      "import * as copy{} from './copy{}/Three.js';export {{copy{}}};\n",
      i, i, i
    );
  }
  let root_dir = PathBuf::from(env!("CARGO_WORKSPACE_DIR"));
  let entry_file = root_dir.join("benchcases/three/src/entry.js");
  let mut file = File::create(entry_file).unwrap();
  file.write_all(entry.as_bytes()).unwrap();

  // test.config.js
  let test_config_file = r#"
module.exports = {
    mode: 'production',
    devtools: 'source-map', 
    builtins: {
      minify: true
    },
    entry: {
        index: {
            import: ['./src/entry.js']
        }
    }
};
  "#;
  let test_config_path = root_dir.join("benchcases/three/test.config.js");
  let mut file = File::create(test_config_path).unwrap();
  file.write_all(test_config_file.as_bytes()).unwrap();

  // webpack.config.js
  let webpack_config_file = r#"
module.exports = {
    mode: 'development',
    entry: {
        index: ['./benchcases/three/src/entry.js']
    },
    devtool: 'eval',
    cache: {type: 'filesystem'}
}
    "#;
  let webpack_config_path = root_dir.join("benchcases/three/webpack.config.js");
  let mut file = File::create(webpack_config_path).unwrap();
  file.write_all(webpack_config_file.as_bytes()).unwrap();
}
