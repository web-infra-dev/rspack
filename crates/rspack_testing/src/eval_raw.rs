use std::{
  path::{Path, PathBuf},
  process::{Command, Stdio},
};

pub fn evaluate_to_json(config_path: &Path) -> Vec<u8> {
  let r = Command::new("node")
    .args(["-p", &get_evaluate_code(config_path)])
    .stdout(Stdio::piped())
    .spawn()
    .expect("ok");
  let out = r.wait_with_output().expect("ok");
  out.stdout
}

fn get_evaluate_code(config_path: &Path) -> String {
  let workspace_dir = PathBuf::from(env!("CARGO_WORKSPACE_DIR"));
  let rspack_path = workspace_dir.join("packages").join("rspack");
  let rspack_path = rspack_path.to_string_lossy();
  let test_dir = config_path.parent().expect("TODO:").to_string_lossy();
  let config_path = config_path.to_string_lossy();
  format!(
    r#"
const rspack = require("{rspack_path}");
const config = require("{config_path}");
const normalized = rspack.getNormalizedRspackOptions(config);
normalized.context="{test_dir}";
normalized.output.path = `{test_dir}/dist`; //TODO:
rspack.applyRspackOptionsDefaults(normalized);
const raw = rspack.getRawOptions(normalized);
JSON.stringify(raw)
"#
  )
}
