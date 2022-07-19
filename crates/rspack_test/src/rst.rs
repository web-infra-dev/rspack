use std::{
  collections::HashSet,
  env,
  error::Error,
  ffi::OsString,
  fmt::Display,
  fs::{self},
  io,
  path::{Path, PathBuf},
};

use colored::Colorize;

use serde::{Deserialize, Serialize};

use bincode;
use similar::ChangeTag;

use crate::helper::{clear_console, is_mute, user_prompt};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Mode {
  Strict,
  Partial,
}

impl Default for Mode {
  fn default() -> Self {
    Mode::Partial
  }
}

#[derive(Debug, Default)]
pub struct TestError(pub Vec<TestErrorKind>);

impl TestError {
  pub fn new() -> Self {
    Self(vec![])
  }

  pub fn push(&mut self, e: TestErrorKind) {
    self.0.push(e);
  }

  pub fn extend(&mut self, e: TestError) {
    self.0.extend(e.0);
  }

  pub fn has_err(&self) -> bool {
    !self.0.is_empty()
  }
}

#[derive(Debug)]
pub enum TestErrorKind {
  MissingActualDir(PathBuf),
  MissingActualFile(PathBuf),
  MissingExpectedDir(PathBuf),
  MissingExpectedFile(PathBuf),
  Difference(PathBuf, PathBuf),
}

impl Display for TestError {
  #[allow(clippy::unwrap_in_result)]
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    for kind in self.0.iter() {
      if let Err(e) = match &kind {
        TestErrorKind::Difference(file, _) => f.write_str(&format!(
          "File is different from expecting: {}",
          file.as_path().to_str().unwrap()
        )),
        TestErrorKind::MissingExpectedDir(dir) => f.write_str(&format!(
          "Can not find 'expected' directory:\nExpected: {}",
          dir.as_path().to_str().unwrap()
        )),
        TestErrorKind::MissingActualDir(dir) => f.write_str(&format!(
          "Can not find 'actual' directory:\nActual: {}",
          dir.as_path().to_str().unwrap()
        )),
        TestErrorKind::MissingActualFile(file) => f.write_str(&format!(
          "There is a file missing: {}",
          file.as_path().to_str().unwrap()
        )),
        TestErrorKind::MissingExpectedFile(file) => f.write_str(&format!(
          "There is a expected file missing: {}",
          file.as_path().to_str().unwrap()
        )),
      } {
        return Err(e);
      }
    }
    Ok(())
  }
}

impl Error for TestError {}

#[derive(Builder, Debug, Serialize, Deserialize, Clone)]
#[builder(default)]
pub struct Rst {
  fixture: PathBuf,
  actual: String,
  expected: String,
  mode: Mode,
}

impl Default for Rst {
  fn default() -> Self {
    Rst {
      fixture: PathBuf::from("fixtures"),
      actual: String::from("actual"),
      expected: String::from("expected"),
      mode: Mode::Partial,
    }
  }
}

impl From<Vec<u8>> for Rst {
  fn from(v: Vec<u8>) -> Self {
    bincode::deserialize(&v[..]).unwrap()
  }
}

#[macro_export]
macro_rules! prt {
  ($($arg:tt)*) => {{
    if !$crate::helper::is_mute() {
      print!($($arg)*);
    }
}};
}

#[macro_export]
macro_rules! prtln {
  ($($arg:tt)*) => {{
    if !is_mute() {
      println!($($arg)*);
    }
}};
}

impl Rst {
  #[inline(always)]
  fn get_expected_path(&self) -> PathBuf {
    let mut base = self.fixture.clone();
    base.push(self.expected.as_str());

    base
  }

  #[inline(always)]
  fn get_actual_path(&self) -> PathBuf {
    let mut base = self.fixture.clone();
    base.push(self.actual.as_str());

    base
  }

  #[inline(always)]
  fn get_record_path(&self) -> PathBuf {
    let mut path_buf = env::current_dir().expect("No permission to access current working dir");
    path_buf.push(".rst_records/records");
    path_buf.push(&self.fixture.as_path().to_str().unwrap().replace('/', "_"));

    path_buf
  }

  #[inline(always)]
  fn get_record_dir() -> PathBuf {
    let mut path_buf = env::current_dir().expect("No permission to access current working dir");
    path_buf.push(".rst_records");
    path_buf
  }

  #[inline(always)]
  fn get_failed_path(&self) -> PathBuf {
    let mut path_buf = env::current_dir().expect("No permission to access current working dir");
    path_buf.push(".rst_records/failed");
    path_buf.push(&self.fixture.as_path().to_str().unwrap().replace('/', "_"));

    path_buf
  }

  // If is called using CLI, need_read_disk is true
  fn init(&mut self, need_read_disk: bool) {
    if self.fixture.to_str().unwrap() == "" {
      panic!("Fixture path must be specified, maybe you forget to call RstBuilder::default().fixture(\"...\")");
    } else if need_read_disk {
      let record_path = self.get_record_path();
      if record_path.exists() {
        let rst = fs::read(record_path.as_path()).unwrap();
        let rst = bincode::deserialize::<Rst>(rst.as_slice()).unwrap();
        self.expected = rst.expected;
        self.actual = rst.actual;
        self.mode = rst.mode;
      }
    }
  }

  fn finalize(&self, res: &Result<(), TestError>) {
    if let Err(e) = res {
      if !is_mute() {
        self.prt_err(e);
      }
      let failed_path = self.get_failed_path();
      if !failed_path.exists() {
        fs::create_dir_all(failed_path.parent().unwrap()).unwrap();
      }
      fs::write(failed_path, bincode::serialize(self).unwrap()).unwrap();
    }

    let record_path = self.get_record_path();
    if !record_path.exists() {
      fs::create_dir_all(record_path.parent().unwrap()).unwrap();
    }
    let rst = bincode::serialize::<Rst>(self).unwrap();
    fs::write(record_path.as_path(), rst).unwrap();
  }

  // Convenient for unit test
  pub fn assert(&mut self) {
    let msg = format!(
      "{}{}",
      "Test failed: ".red(),
      self.fixture.to_str().unwrap().red()
    );
    self.test().expect(&msg);
  }

  pub fn test(&mut self) -> Result<(), TestError> {
    self.test_internal(false)
  }

  fn test_internal(&mut self, need_read_disk: bool) -> Result<(), TestError> {
    self.init(need_read_disk);

    let res = Self::compare(
      &self.mode,
      &self.get_actual_path(),
      &self.get_expected_path(),
    );

    self.finalize(&res);

    res
  }

  fn prt_err(&self, err: &TestError) {
    prtln!(
      "{}",
      format!("{} error:\n\n{}", err.0.len(), err)
        .white()
        .on_red()
    );
  }

  #[allow(clippy::unwrap_in_result)]
  fn compare(mode: &Mode, actual: &Path, expected: &Path) -> Result<(), TestError> {
    let mut err = TestError::new();

    if !actual.exists() || !actual.is_dir() {
      err.push(TestErrorKind::MissingActualDir(PathBuf::from(actual)));
      return Err(err);
    }

    if !expected.exists() || !expected.is_dir() {
      err.push(TestErrorKind::MissingExpectedDir(PathBuf::from(expected)));
      return Err(err);
    }

    let actual_dirs: Vec<OsString> = actual
      .read_dir()
      .unwrap()
      .map(|p| p.unwrap().file_name())
      .collect();

    let actual_dirs: HashSet<OsString> = HashSet::from_iter(actual_dirs);

    let expected_dirs: Vec<OsString> = expected
      .read_dir()
      .unwrap()
      .map(|p| p.unwrap().file_name())
      .collect();
    let expected_dirs: HashSet<OsString> = HashSet::from_iter(expected_dirs);

    for actual_dir_str in actual_dirs.iter() {
      let mut actual_dir = PathBuf::from(actual);
      actual_dir.push(actual_dir_str);

      if let Some(expected_dir_str) = expected_dirs.get(actual_dir_str) {
        let mut expected_dir = PathBuf::from(expected);
        expected_dir.push(expected_dir_str);

        let is_expect_file = expected_dir.is_file();
        let is_actual_file = actual_dir.is_file();

        if is_expect_file && is_actual_file {
          // file diff
          let expected_buf = fs::read(expected_dir.as_path()).unwrap();
          let actual_buf = fs::read(actual_dir.as_path()).unwrap();

          if expected_buf != actual_buf {
            err.push(TestErrorKind::Difference(
              actual_dir.clone(),
              expected_dir.clone(),
            ));
          }
        } else if !is_expect_file && !is_actual_file {
          // directory diff
          if let Err(e) = Self::compare(mode, actual_dir.as_path(), expected_dir.as_path()) {
            err.extend(e);
          }
        } else if is_actual_file {
          // actual is file, but expect is dir
          err.push(TestErrorKind::MissingActualFile(actual_dir.clone()));
        } else {
          // actual is dir, but expect is file
          err.push(TestErrorKind::MissingActualDir(actual_dir.clone()));
        }
      } else if matches!(mode, Mode::Strict) {
        err.push(TestErrorKind::MissingActualFile(actual_dir.clone()));
      }
    }

    for expected_dir_str in expected_dirs.iter() {
      if !actual_dirs.contains(expected_dir_str) {
        let mut expected_dir = PathBuf::from(expected);
        expected_dir.push(expected_dir_str);

        if expected_dir.is_file() {
          err.push(TestErrorKind::MissingExpectedFile(expected_dir.clone()));
        } else {
          err.push(TestErrorKind::MissingExpectedDir(expected_dir.clone()));
        }
      }
    }

    if err.has_err() {
      Err(err)
    } else {
      Ok(())
    }
  }

  /// Remove origin expected directory, copy the actual directory to expected directory.
  pub fn update_fixture(&self) {
    let actual_dir = self.get_actual_path();
    let expected_dir = self.get_expected_path();

    // remove old expected directory
    if expected_dir.exists() {
      for dir in fs::read_dir(&expected_dir)
        .unwrap()
        .map(|d| d.unwrap().path())
      {
        if dir.is_dir() {
          fs::remove_dir_all(dir).unwrap();
        } else {
          fs::remove_file(dir).unwrap();
        }
      }
    }

    // write into new expected directory
    fn cp(orig: &Path, target: &Path) {
      if orig.is_dir() {
        fs::create_dir_all(target).unwrap();
        for dir in fs::read_dir(orig).unwrap() {
          let dir = dir.unwrap().path();
          cp(&dir, &target.join(dir.file_name().unwrap()));
        }
      } else {
        fs::copy(orig, target).unwrap();
      }
    }

    cp(&actual_dir, &expected_dir);

    // update record
    let failed_path = self.get_failed_path();
    if failed_path.exists() {
      fs::remove_file(failed_path).unwrap();
    }
  }

  /// Update all the failed records in the current working directory.
  pub fn update_all_cases(mute: bool) {
    let dir = Self::get_record_dir();
    let mut updates: Vec<Rst> = vec![];

    if !dir.exists() {
      return;
    }

    let failed_files = fs::read_dir(dir).unwrap().map(|dir| dir.unwrap().path());
    for failed_path in failed_files {
      let rst = bincode::deserialize::<Rst>(&fs::read(&failed_path).unwrap()).unwrap();

      rst.update_fixture();

      updates.push(rst);
    }

    if !mute {
      prtln!(
        "Updates {} fixture{}:\n{}",
        updates.len(),
        if updates.len() > 1 { "s" } else { "" },
        updates.iter().fold(String::new(), |str, update| {
          format!("{}\n{}", str, update.get_expected_path().display())
        })
      );
    }
  }

  // CLI only
  // @deprecated
  // pub fn review() {
  //   let failed_dir = Self::get_failed_dir();
  //   let failed_dirs = fs::read_dir(failed_dir).unwrap().map(|d| d.unwrap().path());

  //   for failed_case in failed_dirs {
  //     let mut rst: Rst = fs::read(failed_case).unwrap().into();
  //     if let Err(err) = rst.test() {
  //       let err = err.0;

  //       for e in err {
  //         match e {
  //           TestErrorKind::Difference(actual_dir, expected_dir) => {
  //             if let Some(
  //               "js" | "ts" | "jsx" | "tsx" | "html" | "vue" | "txt" | "json" | "yml" | "yaml"
  //               | "toml" | "css" | "less" | "sass" | "scss" | "md" | "markdown" | "mdx" | "xml",
  //             ) = actual_dir.extension().as_ref().map(|e| e.to_str().unwrap())
  //             {
  //               // Text File diff

  //               let expected = String::from_utf8(fs::read(&expected_dir).unwrap()).unwrap();
  //               let actual = String::from_utf8(fs::read(&actual_dir).unwrap()).unwrap();

  //               let diff = similar::TextDiff::from_lines(expected.as_str(), actual.as_str());

  //               prtln!(
  //                 "> {}\n{}",
  //                 expected_dir.display(),
  //                 "'+' means file in the actual directory has this line but expected file does not"
  //                   .white()
  //                   .on_green()
  //               );

  //               prtln!("------------------------------------------------------");

  //               for change in diff.iter_all_changes() {
  //                 let line = match change.tag() {
  //                   ChangeTag::Delete => format!("+{}", &change).green(),
  //                   ChangeTag::Insert => format!("-{}", &change).red(),
  //                   _ => format!(" {}", &change).bright_black(),
  //                 };
  //                 prt!("{}", line);
  //               }

  //               prt!("------------------------------------------------------\n");

  //               if user_prompt("Update expected file?") {
  //                 rst.update_fixture();
  //               }
  //             }
  //           }
  //           TestErrorKind::MissingActualFile(actual_file) => {
  //             prtln!(
  //               "{}",
  //               format!(
  //                 "Missing {}\nDo you want to add that file?",
  //                 actual_file.display()
  //               )
  //               .green()
  //             );
  //             prtln!(
  //               "{}",
  //               "Type 'a' to add that file\nType 'c' to continue reviewing".green()
  //             );

  //             if user_prompt("Create the missing file?") {}
  //           }
  //           TestErrorKind::MissingActualDir(path) => {}
  //           TestErrorKind::MissingExpectedFile(path) => {}
  //           TestErrorKind::MissingExpectedDir(path) => {}
  //         };
  //         clear_console();
  //       }
  //     }
  //   }
  // }
}

pub fn assert(p: PathBuf) {
  let mut rst = RstBuilder::default().fixture(p).build().unwrap();
  rst.assert();
}

#[cfg(test)]
mod test {
  use std::{env, fs, path::PathBuf};

  use crate::rst::{RstBuilder, TestErrorKind};
  use testing_macros::fixture;

  use super::Rst;

  #[fixture("fixtures/same/*")]
  fn same(p: PathBuf) {
    assert!(RstBuilder::default()
      .fixture(p)
      .build()
      .unwrap()
      .test()
      .is_ok());
  }

  #[test]
  fn different() {
    env::set_var("MUTE", "true");
    let cwd = env::current_dir().unwrap();

    /*
     * A file in the expect dir, but not in the actual dir
     */
    let mut p = cwd.clone();
    p.push("fixtures/diff/dirs");

    let test_res = RstBuilder::default()
      .fixture(p.clone())
      .build()
      .unwrap()
      .test();

    assert!(test_res.is_err());
    let err = test_res.unwrap_err().0;
    assert!(err.len() == 1);

    p.push("expected/a");
    match &err[0] {
      TestErrorKind::MissingExpectedDir(expect) => {
        assert_eq!(expect.as_path(), &p);
      }
      _ => panic!("Test Fail"),
    };

    /*
     * two files are different in content
     */
    let mut p = cwd.clone();
    p.push("fixtures/diff/files");
    let test_res = RstBuilder::default()
      .fixture(p.clone())
      .build()
      .unwrap()
      .test();
    assert!(test_res.is_err());

    let err = test_res.unwrap_err().0;
    assert!(err.len() == 1);
    match &err[0] {
      TestErrorKind::Difference(actual, _) => {
        p.push("actual/a.js");
        assert_eq!(actual, &p);
      }
      _ => panic!("Test Fail"),
    };

    /*
     * there is a misssing in the actual dir
     */
    let mut p = cwd;
    p.push("fixtures/diff/missing");
    let test_res = RstBuilder::default()
      .fixture(p.clone())
      .build()
      .unwrap()
      .test();
    assert!(test_res.is_err());

    let err = test_res.unwrap_err().0;
    assert!(err.len() == 1);
    match &err[0] {
      TestErrorKind::MissingExpectedFile(missing) => {
        p.push("expected/b.js");
        assert_eq!(missing, &p);
      }
      _ => panic!("Test Fail"),
    };
  }

  #[test]
  fn update() {
    let cwd = env::current_dir().unwrap();
    let mut p = cwd;
    p.push("fixtures/update/a");
    let mut rst = RstBuilder::default().fixture(p.clone()).build().unwrap();

    // fail because the expected dir is missing
    assert!(rst.test().is_err());

    rst.update_fixture();
    assert!(rst.test().is_ok());

    // recover for next time test call
    fs::remove_dir_all(p.as_path().join("expected")).unwrap();
  }
}
