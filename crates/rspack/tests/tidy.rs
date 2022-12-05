/// copy from https://github.com/rust-lang/rust-analyzer/blob/master/crates/rust-analyzer/tests/slow-tests/tidy.rs#L1
use std::path::{Path, PathBuf};
use xshell::Shell;

#[test]
fn files_are_tidy() {
  let sh = &Shell::new().unwrap();
  let files = list_files(&project_root().join("crates"));
  for path in files {
    let extension = path
      .extension()
      .unwrap_or_default()
      .to_str()
      .unwrap_or_default();
    match extension {
      "rs" => {
        let text = sh.read_file(&path).unwrap();
        check_dbg(&path, &text);
      }
      "toml" => {}
      _ => (),
    }
  }
}

fn check_dbg(path: &Path, text: &str) {
  let need_dbg = [
    // exclude itself
    "rspack/tests/tidy.rs",
  ];
  if need_dbg.iter().any(|p| path.ends_with(p)) {
    return;
  }
  if text.contains("dbg!") {
    panic!(
      "\ndbg! macros should not be committed to the main branch,\n\
           {}\n",
      path.display(),
    )
  }
}

pub fn list_files(dir: &Path) -> Vec<PathBuf> {
  let mut res = Vec::new();
  let mut work = vec![dir.to_path_buf()];
  while let Some(dir) = work.pop() {
    for entry in dir.read_dir().unwrap() {
      let entry = entry.unwrap();
      let file_type = entry.file_type().unwrap();
      let path = entry.path();
      let is_hidden = path
        .file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
        .starts_with('.');
      if !is_hidden {
        if file_type.is_dir() {
          work.push(path);
        } else if file_type.is_file() {
          res.push(path);
        }
      }
    }
  }
  res
}

pub fn project_root() -> PathBuf {
  let dir = env!("CARGO_MANIFEST_DIR");
  let res = PathBuf::from(dir)
    .parent()
    .unwrap()
    .parent()
    .unwrap()
    .to_owned();
  res
}
