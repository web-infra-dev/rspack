// use std::{
//   env,
//   ffi::OsString,
//   fs, io,
//   path::{Path, PathBuf},
// };
//
// use cargo_rst::helper::for_each_dir;
// use cargo_rst::setup;
//
// /*
//  * disable update test in CI, because update will remove
//  * record file, which is forbidden in CI env.
//  * We can check env var in the runtime but it is an invasion to
//  * the library.
//  */
// fn is_in_ci() -> bool {
//   env::var("CI").is_ok()
// }
//
// // #[test]
// fn main() {
//   if is_in_ci() {
//     return;
//   }
//
//   prepare();
//
//   // Integration test, current_dir is project root dir
//   let cwd = env::current_dir().expect("TODO:");
//
//   // Test all fixtures, all should failed
//   let mut all = cwd.clone();
//   all.push("tests/fixtures");
//
//   for_each_dir(&all, |dir| {
//     assert!(cargo_rst::test(PathBuf::from(dir)).is_err());
//   });
//
//   // update one fixture
//   let mut file_fixture = cwd;
//   file_fixture.push("tests/fixtures/files");
//
//   let cmd: Vec<OsString> = vec![
//     "rst".into(),
//     "update".into(),
//     "-p".into(),
//     "tests/fixtures/files".into(),
//   ];
//   setup(&cmd);
//
//   assert!(cargo_rst::test(file_fixture.clone()).is_ok());
//
//   // update all fixtures
//   let cmd: Vec<OsString> = vec!["rst".into(), "update".into()];
//   setup(&cmd);
//
//   for_each_dir(&all, |dir| {
//     assert!(cargo_rst::test(PathBuf::from(dir)).is_ok());
//   });
// }
//
// fn prepare() {
//   let mut template = env::current_dir().expect("TODO:");
//
//   let mut target = template.clone();
//   template.push("tests/templates");
//   target.push("tests/fixtures");
//   let res = copy(&template, &target);
//   println!("cwd: {}", env::current_dir().expect("TODO:").display());
//   res.expect("TODO:");
// }
//
// fn copy(orig: &Path, dest: &Path) -> io::Result<()> {
//   if orig.is_file() {
//     fs::copy(orig, dest).map(|_| ())
//   } else {
//     if !dest.exists() {
//       fs::create_dir(dest)?;
//     }
//
//     let next = PathBuf::from(orig);
//     let dest = PathBuf::from(dest);
//     for dir in fs::read_dir(orig).expect("TODO:").flatten() {
//       let mut next = next.clone();
//       let name = dir.file_name();
//       next.push(name.clone());
//
//       let mut dest = dest.clone();
//       dest.push(name);
//       copy(&next, &dest)?
//     }
//
//     Ok(())
//   }
// }
