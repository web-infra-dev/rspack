use std::{
  env, fs,
  path::{self, Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{helper::make_relative_from, rst::Rst};

#[derive(Debug, Serialize, Deserialize)]
pub struct Record {
  pub config: Rst,
  pub causes: Vec<FailedCase>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FailedCase {
  MissingActualDir(PathBuf),
  MissingActualFile(PathBuf),
  MissingExpectedDir(PathBuf),
  MissingExpectedFile(PathBuf),
  Difference {
    expected_file_path: PathBuf,
    added: Vec<usize>,
    removed: Vec<usize>,
  },
}

impl Record {
  pub fn new(rst: &Rst, causes: Vec<FailedCase>) -> Self {
    Self {
      config: rst.clone(),
      causes,
    }
  }

  pub fn save_to_disk(&self) {
    let cwd = env::current_dir().unwrap();
    let mut p = cwd.clone();
    p.push(".temp");

    let relative = make_relative_from(self.config.fixture.as_path(), cwd.as_path());
    let record_path = { relative + ".json" }.replace(path::MAIN_SEPARATOR, "&");

    p.push(record_path);

    fs::write(p, self.serialize()).unwrap();
  }

  pub fn serialize(&self) -> String {
    serde_json::to_string_pretty(self).unwrap()
  }
}

impl<T> From<T> for Record
where
  T: AsRef<Path>,
{
  /// from relative path
  fn from(p: T) -> Self {
    let relative_fixture = p.as_ref();
    let relative_fixture = { relative_fixture.to_str().unwrap().to_string() + ".json" }
      .replace(path::MAIN_SEPARATOR, "&");

    let mut record_path = env::current_dir().unwrap();
    record_path.push(".temp");
    record_path.push(&relative_fixture);

    if !record_path.exists() {
      println!("Record {} is not exist", record_path.as_path().display());
      panic!();
    }

    serde_json::from_slice(fs::read(record_path).unwrap().as_slice()).unwrap()
  }
}

impl From<Record> for Rst {
  fn from(record: Record) -> Self {
    let mut rst = record.config;

    rst.errors = Some(record.causes);

    rst
  }
}
