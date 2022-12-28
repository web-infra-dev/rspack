use std::{
  env, fs,
  path::{Path, PathBuf},
};

pub fn for_each_dir<F>(p: &Path, f: F)
where
  F: Fn(&Path),
{
  for dir in fs::read_dir(p).expect("TODO:") {
    let dir = dir.expect("TODO:");
    let mut p = PathBuf::from(p);
    p.push(dir.path());

    f(p.as_path())
  }
}

#[inline(always)]
pub fn is_mute() -> bool {
  env::var("RST_MUTE").is_ok()
}

pub fn is_detail() -> bool {
  env::var("RST_DETAIL").is_ok()
}

pub fn no_write() -> bool {
  env::var("RST_NO_WRITE").is_ok()
}

pub fn make_relative_from(path: &Path, base: &Path) -> String {
  let mut path_iter = path.iter();

  for curr_base in base.iter() {
    match path_iter.next() {
      Some(curr_path) => {
        if curr_path != curr_base {
          panic!("Second path is not the root path of the first one");
        }
        continue;
      }
      None => break,
    };
  }

  path_iter
    .collect::<PathBuf>()
    .to_str()
    .expect("TODO:")
    .into()
}

// Recursively copy orig dir to target dir
pub fn cp(orig: &Path, target: &Path) {
  if orig.is_dir() {
    fs::create_dir_all(target).expect("TODO:");
    for dir in fs::read_dir(orig).expect("TODO:") {
      let dir = dir.expect("TODO:").path();
      cp(&dir, &target.join(dir.file_name().expect("TODO:")));
    }
  } else {
    fs::copy(orig, target).expect("TODO:");
  }
}
