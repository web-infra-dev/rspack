use std::{
  env,
  path::{self, Path, PathBuf},
};

use rspack_test::rst::Rst;

fn get_record_dir() -> PathBuf {
  let mut p = env::current_dir().unwrap();
  p.push(".temp");
  p
}

pub fn update(fixture: Option<String>) {
  if let Some(fixture) = fixture {
    let path: &Path = Path::new(fixture.as_str());

    let mut records_dir = get_record_dir();

    if path.is_relative() {
      let mut p = env::current_dir().unwrap();

      p.push(path);

      if !p.exists() {
        println!("Fixture not found");
        return;
      }

      records_dir
        .push({ path.to_str().unwrap().to_string() + ".json" }.replace(path::MAIN_SEPARATOR, "&"));
    } else {
      if !path.exists() {
        println!("Fixture not found");
        return;
      }
      records_dir
        .push({ path.to_str().unwrap().to_string() + ".json" }.replace(path::MAIN_SEPARATOR, "&"));
    };

    let rst = Rst::from_path(path);
    rst.update_fixture();
    println!("Updated 1 fixture");
  } else {
    Rst::update_all_cases();
  }
}
