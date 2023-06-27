use std::{
  collections::HashSet,
  env,
  error::Error,
  ffi::OsString,
  fmt::Display,
  fs,
  path::{self, Path, PathBuf},
  sync::{Arc, Mutex},
};

use colored::Colorize;
use derive_builder::Builder;
use glob::glob;
use serde::{Deserialize, Serialize};
use serde_json;

use crate::{
  helper::{cp, is_detail, is_mute, make_relative_from},
  record::{self, FailedCase, Record},
  terminal_inline::diff_and_print,
};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub enum Mode {
  Strict,
  #[default]
  Partial,
}

#[derive(Debug, Default)]
pub struct TestError {
  fixture: String,
  errors: Vec<TestErrorKind>,
}

impl TestError {
  pub fn new(fixture: String) -> Self {
    Self {
      fixture,
      errors: vec![],
    }
  }

  pub fn push(&mut self, e: TestErrorKind) {
    self.errors.push(e);
  }

  pub fn extend(&mut self, e: TestError) {
    self.errors.extend(e.errors);
  }

  pub fn has_err(&self) -> bool {
    !self.errors.is_empty()
  }
}

#[derive(Debug)]
pub enum TestErrorKind {
  MissingActualDir(PathBuf),
  MissingActualFile(PathBuf),
  MissingExpectedDir(PathBuf),
  MissingExpectedFile(PathBuf),
  Difference(FileDiff),
}

#[derive(Debug)]
pub struct FileDiff {
  /// expected file path
  expected_path: PathBuf,

  actual_path: PathBuf,
  /// This two fields is necessary because the expected content will be override if `UPDATE` is set
  expected_content: String,
  actual_content: String,
}

fn output(f: &mut std::fmt::Formatter<'_>, prefix: &str, msg: &str) -> std::fmt::Result {
  f.write_str(&format!(
    "{}",
    format!("- {}: {}\n", prefix.white().on_red(), msg).red()
  ))
}
impl Display for TestError {
  #[allow(clippy::unwrap_in_result)]
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(
      &format!(
        "{}",
        format!("Fixture: {}\n{} error\n\n", self.fixture, self.errors.len()).red()
      )
      .red(),
    )
    .expect("TODO:");

    for kind in self.errors.iter() {
      match &kind {
        TestErrorKind::Difference(diff) => f.write_str(&format!(
          "File difference: {}\n{}",
          diff.expected_path.to_string_lossy().red(),
          &diff_and_print(&diff.expected_content, &diff.actual_content)
        )),
        TestErrorKind::MissingExpectedDir(dir) => output(
          f,
          "Missing expected directory(maybe you create a new snapshot test case but forgot to add a expected snapshot, try to add `expected/main.js` under your new test case directory, or try to refresh the fixture.rs under that crate): ",
          dir.as_path().to_str().expect("TODO:"),
        ),
        TestErrorKind::MissingActualDir(dir) => output(
          f,
          "Missing actual directory: ",
          dir.as_path().to_str().expect("TODO:"),
        ),
        TestErrorKind::MissingActualFile(file) => output(
          f,
          "File exists in 'actual' directory, but not found in 'expected' directory: ",
          file.as_path().to_str().expect("TODO:"),
        ),
        TestErrorKind::MissingExpectedFile(file) => output(
          f,
          "File exists in 'expected' directory, but not found in 'actual' directory: ",
          file.as_path().to_str().expect("TODO:"),
        ),
      }?;
    }
    Ok(())
  }
}

impl Error for TestError {}

#[derive(Builder, Debug, Serialize, Deserialize, Clone)]
#[builder(default)]
pub struct Rst {
  pub fixture: PathBuf,
  pub actual: String,
  pub expected: String,
  pub mode: Mode,
  pub errors: Option<Vec<FailedCase>>,
}

impl Default for Rst {
  fn default() -> Self {
    Rst {
      fixture: PathBuf::from("fixtures"),
      actual: String::from("actual"),
      expected: String::from("expected"),
      mode: Mode::Partial,
      errors: None,
    }
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
macro_rules! println_if_not_mute {
    ($($arg:tt)*) => {{
      if !is_mute() {
        println!($($arg)*);
      }
  }};
  }

impl Rst {
  /// Generate Rst using **relative** path.
  pub fn from_path(path: &Path) -> Self {
    record::Record::from(path).into()
  }

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
    let root = env::current_dir().expect("No permission to access current working dir");
    let mut path_buf = root.clone();
    path_buf.push(".temp");

    let json_path = self.fixture.to_str().expect("TODO:").to_string() + ".json";
    let json_path = Path::new(&json_path);

    path_buf.push(make_relative_from(json_path, &root).replace(path::MAIN_SEPARATOR, "&"));

    path_buf
  }

  #[inline(always)]
  fn get_record_dir() -> PathBuf {
    let mut path_buf = env::current_dir().expect("No permission to access current working dir");
    path_buf.push(".temp");
    path_buf
  }

  /// /fixture/expected/a.js -> /fixture/actual/a.js
  fn expected_2_actual(&self, expected: &Path) -> PathBuf {
    expected
      .to_str()
      .expect("TODO:")
      .replace(&self.expected, &self.actual)
      .into()
  }

  fn validate(&self) {
    if self.fixture.to_str().expect("TODO:") == "" {
      panic!("Fixture path must be specified, maybe you forget to call RstBuilder::default().fixture(\"...\")");
    }
  }

  fn need_update() -> bool {
    std::env::var("UPDATE").is_ok()
  }

  fn finalize(&self, res: &Result<(), TestError>) {
    // if env `UPDATE` is set, the snapshot will automatically
    // update
    let need_update = Self::need_update();

    let record_path = self.get_record_path();

    if let Err(err) = res {
      // Test failed, we should save the failed record.
      if !need_update && !record_path.exists() {
        fs::create_dir_all(record_path.parent().expect("TODO:")).expect("TODO:");
      }

      let record = record::Record::new(
        self,
        err
          .errors
          .iter()
          .map(|err| match err {
            TestErrorKind::MissingActualDir(p) => FailedCase::MissingActualDir(p.clone()),
            TestErrorKind::MissingActualFile(p) => FailedCase::MissingActualFile(p.clone()),
            TestErrorKind::MissingExpectedDir(p) => FailedCase::MissingExpectedDir(p.clone()),
            TestErrorKind::MissingExpectedFile(p) => FailedCase::MissingExpectedFile(p.clone()),
            TestErrorKind::Difference(diff) => FailedCase::Difference {
              expected_file_path: diff.expected_path.clone(),
              actual_file_path: diff.actual_path.clone(),
            },
          })
          .collect(),
      );

      if Self::need_update() {
        let rst: Rst = record.into();
        rst.update_fixture();
      } else {
        record.save_to_disk();
      }
    }
  }

  /// Main test method
  /// This will write failed records in the disk
  #[allow(clippy::unwrap_in_result)]
  pub fn test(&self) -> Result<(), TestError> {
    self.validate();

    let res = Self::compare(
      self.fixture.to_str().expect("TODO:").into(),
      &self.mode,
      &self.get_actual_path(),
      &self.get_expected_path(),
      is_detail(),
    );

    self.finalize(&res);
    res
  }

  pub fn assert(&self) {
    let res = self.test();
    if let Err(e) = res {
      println_if_not_mute!("{}", e);
      panic!("Fixture test failed");
    }
  }

  #[allow(clippy::unwrap_in_result, clippy::only_used_in_recursion)]
  fn compare(
    fixture: String,
    mode: &Mode,
    actual_base: &Path,
    expected_base: &Path,
    verbose: bool,
  ) -> Result<(), TestError> {
    let mut err = TestError::new(fixture.clone());

    if !actual_base.exists() || !actual_base.is_dir() {
      err.push(TestErrorKind::MissingActualDir(PathBuf::from(actual_base)));
      return Err(err);
    }

    if !expected_base.exists() || !expected_base.is_dir() {
      err.push(TestErrorKind::MissingExpectedDir(PathBuf::from(
        expected_base,
      )));
      return Err(err);
    }

    let actual_dirs: Vec<OsString> = actual_base
      .read_dir()
      .expect("TODO:")
      .map(|p| p.expect("TODO:").file_name())
      .collect();

    let actual_dirs: HashSet<OsString> = HashSet::from_iter(actual_dirs);

    let expected_dirs: Vec<OsString> = expected_base
      .read_dir()
      .expect("TODO:")
      .map(|p| p.expect("TODO:").file_name())
      .collect();
    let expected_dirs: HashSet<OsString> = HashSet::from_iter(expected_dirs);

    for actual_dir_str in actual_dirs.iter() {
      let mut actual_dir = PathBuf::from(actual_base);
      actual_dir.push(actual_dir_str);

      if let Some(expected_dir_str) = expected_dirs.get(actual_dir_str) {
        let mut expected_dir = PathBuf::from(expected_base);
        expected_dir.push(expected_dir_str);

        let is_expect_file = expected_dir.is_file();
        let is_actual_file = actual_dir.is_file();

        if is_expect_file && is_actual_file {
          let expected_buf = fs::read(expected_dir.as_path()).expect("TODO:");
          let expected_str = String::from_utf8_lossy(&expected_buf);

          let actual_buf = fs::read(actual_dir.as_path()).expect("TODO:");
          let actual_str = String::from_utf8_lossy(&actual_buf);
          // ignore newline diff for ci
          if expected_str.replace("\r\n", "\n") != actual_str.replace("\r\n", "\n") {
            let diff = FileDiff {
              expected_path: expected_dir.clone(),
              actual_path: actual_dir.clone(),
              expected_content: expected_str.to_string(),
              actual_content: actual_str.to_string(),
            };

            err.push(TestErrorKind::Difference(diff))
          }
        } else if !is_expect_file && !is_actual_file {
          // directory diff
          if let Err(e) = Self::compare(
            fixture.clone(),
            mode,
            actual_dir.as_path(),
            expected_dir.as_path(),
            verbose,
          ) {
            err.extend(e);
          }
        } else if actual_dir.is_dir() {
          // actual is dir, but expected is file
          err.push(TestErrorKind::MissingExpectedFile(expected_dir.clone()));
        } else {
          // actual is file, but expected is dir
          err.push(TestErrorKind::MissingExpectedDir(expected_dir.clone()));
        }
      } else if matches!(mode, Mode::Strict) {
        // strict check, expected must exist
        if actual_dir.is_dir() {
          err.push(TestErrorKind::MissingActualDir(actual_dir.clone()));
        } else {
          err.push(TestErrorKind::MissingActualFile(actual_dir.clone()));
        }
      }
    }

    for expected_dir_str in expected_dirs.iter() {
      if !actual_dirs.contains(expected_dir_str) {
        let mut expected_dir = PathBuf::from(expected_base);
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
  /// If mode is Partial, only update files in expected directory, and should pass errors
  /// argument, so it can only update failed cases.
  pub fn update_fixture(&self) {
    let actual_dir = self.get_actual_path();
    let expected_dir = self.get_expected_path();

    if !actual_dir.exists() {
      fs::create_dir_all(actual_dir.as_path()).expect("TODO:");
    }

    match (&self.errors, &self.mode) {
      (Some(errors), &Mode::Partial) => {
        for err in errors {
          match err {
            // If mode is Partial, those 2 errors are impossible to occur.
            FailedCase::MissingActualDir(_) => unreachable!(),
            FailedCase::MissingActualFile(_) => unreachable!(),
            FailedCase::MissingExpectedDir(dir) => {
              // Expected dir should not exist
              fs::remove_dir_all(dir).unwrap_or_else(|_| panic!("Remove {dir:?} dir failed"));
            }
            FailedCase::MissingExpectedFile(file) => {
              // Expected file should not exist
              fs::remove_file(file).expect("Remove file failed");
            }
            FailedCase::Difference {
              expected_file_path, ..
            } => {
              let actual_content =
                fs::read(self.expected_2_actual(expected_file_path)).expect("TODO:");
              fs::write(expected_file_path, actual_content).unwrap_or_else(|_| {
                panic!(
                  "Copy file from actual into expected file failed\n{}\n",
                  self.expected_2_actual(expected_file_path).display()
                )
              });
            }
          }
        }
      }
      _ => {
        println!("Not here");
        // Fully update, just copy actual dir to expected dir
        // Remove old expected directory
        if expected_dir.exists() {
          for dir in fs::read_dir(&expected_dir)
            .expect("TODO:")
            .map(|d| d.expect("TODO:").path())
          {
            if dir.is_dir() {
              fs::remove_dir_all(dir).expect("TODO:");
            } else {
              fs::remove_file(dir).expect("TODO:");
            }
          }
        }

        cp(&actual_dir, &expected_dir);

        // update record
        let failed_path = self.get_record_path();
        if failed_path.exists() {
          fs::remove_file(failed_path.as_path()).expect("TODO:");
          // Remove when fix all records
          let record_dir = Self::get_record_dir();

          let failed_count = fs::read_dir(record_dir.as_path()).expect("TODO:").count();
          if failed_count == 0 {
            match fs::remove_dir(record_dir.as_path()) {
              Ok(_) => {}
              Err(e) => {
                println!("{e}");
                panic!("Unable to delete record dir (.temp)");
              }
            }
          }
        }
      }
    };
  }

  /// Update all the failed records in the current working directory.
  pub fn update_all_cases() {
    let workspace_dir =
      std::env::var("CARGO_WORKSPACE_DIR").expect("Can't get CARGO_WORKSPACE_DIR");
    // dbg!(&minifest);
    for entry in
      glob(&format!("{workspace_dir}crates/*/.temp")).expect("Failed to read glob pattern")
    {
      match entry {
        Ok(entry) => {
          update_single_case(&entry);
          // remove temp dir that store diff info
          std::fs::remove_dir_all(entry).expect("TODO:");
        }
        Err(e) => println!("{e:?}"),
      }
    }
  }
}

fn update_single_case(dir: &PathBuf) {
  let updates: Arc<Mutex<Vec<PathBuf>>> = Default::default();
  if !dir.exists() {
    println_if_not_mute!("No records found, nothing updated");
  }
  let failed_files = fs::read_dir(dir)
    .expect("TODO:")
    .map(|dir| dir.expect("TODO:").path())
    .collect::<Vec<_>>();
  failed_files.iter().for_each(|failed_path| {
    let record =
      serde_json::from_slice::<Record>(&fs::read(failed_path).expect("TODO:")).expect("TODO:");
    let rst: Rst = record.into();
    rst.update_fixture();

    updates.clone().lock().expect("TODO:").push(rst.fixture);
  });
  let updates = updates.lock().expect("TODO:");
  let count = updates.len();
  println_if_not_mute!(
    "Updated {} fixture{}:\n{}",
    count.to_string().green(),
    if count > 1 { "s" } else { "" },
    updates.iter().fold(String::new(), |str, update| {
      format!("{}\n{}", str, update.display())
    })
  );
}

pub fn test(p: PathBuf) -> Result<(), TestError> {
  let rst = RstBuilder::default().fixture(p).build().expect("TODO:");
  rst.test()
}

pub fn assert(p: PathBuf) {
  let rst = RstBuilder::default().fixture(p).build().expect("TODO:");
  let res = rst.test();

  if let Err(e) = res {
    println_if_not_mute!("{}", e);
    panic!("Fixture test failed");
  }
}

// #[cfg(test)]
// mod test {
//   use crate::{
//     helper::for_each_dir,
//     rst::{self, Mode, RstBuilder, TestErrorKind},
//   };
//   use std::{env, fs, path::PathBuf};
//   use testing_macros::fixture;

//   #[fixture("fixtures/same/*")]
//   fn same(p: PathBuf) {
//     rst::assert(p);
//   }

//   #[test]
//   #[ignore = "reason"]
//   fn different() {
//     env::set_var("RST_MUTE", "1");

//     let cwd = env::current_dir().expect("TODO:");

//     /*
//      * A file in the expect dir, but not in the actual dir
//      */
//     let mut p = cwd.clone();
//     p.push("fixtures/diff/dirs");

//     let test_res = RstBuilder::default()
//       .fixture(p.clone())
//       .build()
//       .expect("TODO:")
//       .test();

//     assert!(test_res.is_err());
//     let err = test_res.unwrap_err().errors;
//     assert!(err.len() == 1);

//     p.push("expected/a");
//     match &err[0] {
//       TestErrorKind::MissingExpectedDir(expect) => {
//         assert_eq!(expect.as_path(), &p);
//       }
//       _ => {
//         println!("{:?}", err[0]);
//         panic!("Expected error is missing expected dir");
//       }
//     };

//     /*
//      * two files are different in content
//      */
//     let mut p = cwd.clone();
//     p.push("fixtures/diff/files");
//     let test_res = RstBuilder::default()
//       .fixture(p.clone())
//       .build()
//       .expect("TODO:")
//       .test();
//     assert!(test_res.is_err());

//     let err = test_res.unwrap_err().errors;
//     assert!(err.len() == 1);
//     match &err[0] {
//       TestErrorKind::Difference(diff) => {
//         p.push("expected/a.js");
//         assert_eq!(diff.path.as_path(), &p);
//       }
//       _ => panic!("Test Fail"),
//     };

//     /*
//      * there is a missing in the actual dir
//      */
//     let mut p = cwd;
//     p.push("fixtures/diff/missing");
//     let test_res = RstBuilder::default()
//       .fixture(p.clone())
//       .build()
//       .expect("TODO:")
//       .test();
//     assert!(test_res.is_err());

//     let err = test_res.unwrap_err().errors;
//     assert!(err.len() == 1);
//     match &err[0] {
//       TestErrorKind::MissingExpectedFile(missing) => {
//         p.push("expected/b.js");
//         assert_eq!(missing, &p);
//       }
//       _ => panic!("Test Fail"),
//     };
//   }

//   /*
//    * disable update test in CI, because update will remove
//    * record file, which is forbidden in CI env.
//    * We can check env var in the runtime but it is an invasion to
//    * the library.
//    */
//   fn is_in_ci() -> bool {
//     env::var("CI").is_ok()
//   }

//   #[test]
//   fn update() {
//     if is_in_ci() {
//       return;
//     }

//     let cwd = env::current_dir().expect("TODO:");
//     let mut p = cwd.clone();
//     p.push("fixtures/update/a");
//     let rst = RstBuilder::default().fixture(p.clone()).build().expect("TODO:");

//     // fail because the expected dir is missing
//     assert!(rst.test().is_err());

//     rst.update_fixture();
//     assert!(rst.test().is_ok());

//     // recover for next time testing
//     fs::remove_dir_all(p.as_path().join("expected")).expect("TODO:");

//     let mut p = cwd;
//     p.push("fixtures/update/update_all");

//     for_each_dir(p.as_path(), |dir| {
//       let rst = RstBuilder::default()
//         .fixture(PathBuf::from(dir))
//         .mode(Mode::Strict)
//         .build()
//         .expect("TODO:");

//       assert!(rst.test().is_err());
//     });
//   }
// }
