use pathdiff::diff_paths;
use std::path::Path;

fn get_dir(path: &Path) -> &Path {
  match path.parent() {
    Some(p) => p,
    None => path,
  }
}

pub fn node_dynamic_url_template(a: &str, b: &str) -> String {
  let path_a = get_dir(Path::new(a));
  let path_b = get_dir(Path::new(b));

  let relative = diff_paths(path_a, path_b)
    .unwrap()
    .to_string_lossy()
    .to_string();
  format!(
    r#""{}/" + "{}""#,
    relative,
    Path::new(a).file_name().unwrap().to_string_lossy()
  )
}
