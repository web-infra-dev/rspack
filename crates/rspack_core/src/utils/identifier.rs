use std::path::Path;

use sugar_path::SugarPath;

pub fn contextify(context: impl AsRef<Path>, request: &str) -> String {
  let context = context.as_ref();
  request
    .split('!')
    .map(|r| absolute_to_request(context, r))
    .collect::<Vec<String>>()
    .join("!")
}

fn absolute_to_request(context: &Path, maybe_absolute_path: &str) -> String {
  if let Some((resource, query)) = maybe_absolute_path.split_once('?') {
    let resource = relative_path_to_request(&Path::new(resource).relative(context));
    resource + "?" + query
  } else {
    relative_path_to_request(&Path::new(maybe_absolute_path).relative(context))
  }
}

fn relative_path_to_request(relative: &Path) -> String {
  let r = relative.to_string_lossy();
  if r == "" {
    return "./.".to_string();
  }
  if r == ".." {
    return "../.".to_string();
  }
  if r.starts_with("../") {
    return r.to_string();
  }
  format!("./{r}")
}
