mod utils;
mod visitor;

use std::sync::Arc;

use rspack_fs::{AbsPath, ReadableFileSystem};
use rspack_javascript_compiler::JavaScriptCompiler;
use rspack_paths::{AssertUtf8, Utf8Path, Utf8PathBuf};
use rspack_resolver::ResolveError;
use rustc_hash::FxHashSet as HashSet;
use swc_core::{
  base::config::IsModule,
  common::FileName,
  ecma::{ast::EsVersion, parser::Syntax},
};
pub use utils::is_node_package_path;

use self::visitor::DependencyVisitor;
use crate::{
  CompilationLogger, Logger, Resolve as ResolveOption, ResolveInnerError, ResolveResult, Resolver,
};

/// A toolkit to recursively calculate files used by build dependencies.
///
/// The toolkit will use ast to analyze the build dependency files and resolve the requests in them,
/// treat the files associated with the requests as build dependency files,
/// and continue processing them until all dependency files are calculated.
pub struct Helper<L = CompilationLogger> {
  resolver: Resolver,
  logger: L,
}

impl<L: Logger> Helper<L> {
  pub fn new(fs: Arc<dyn ReadableFileSystem>, logger: L) -> Self {
    Helper {
      resolver: Resolver::new(
        ResolveOption {
          condition_names: Some(vec!["import".into(), "require".into(), "node".into()]),
          exports_fields: Some(vec![vec!["exports".into()]]),
          builtin_modules: true,
          extensions: Some(vec![
            ".js".into(),
            ".ts".into(),
            ".mjs".into(),
            ".cjs".into(),
            ".json".into(),
            ".node".into(),
          ]),
          ..Default::default()
        },
        fs,
      ),
      logger,
    }
  }

  /// Resolve a directory.
  ///
  /// Return the sub dir and files.
  async fn resolve_directory(&mut self, dir: &Utf8Path) -> Option<HashSet<Utf8PathBuf>> {
    let abs_dir = match AbsPath::new(dir.as_str()) {
      Ok(abs) => abs,
      Err(err) => {
        self.logger.warn(format!(
          "BuildDependencies: can't read directory {dir}.\n  - {err}"
        ));
        return None;
      }
    };
    let entries = self.resolver.inner_fs().read_dir(&abs_dir).await;
    match entries {
      Ok(entries) => Some(entries.into_iter().map(|item| dir.join(item)).collect()),
      Err(err) => {
        self.logger.warn(format!(
          "BuildDependencies: can't read directory {dir}.\n  - {err}"
        ));
        None
      }
    }
  }

  /// Resolve a file.
  ///
  /// This function will try analyze the ast and resolve the requests in them.
  /// Return the file path that the input file depends on.
  async fn resolve_file(&mut self, file: &Utf8Path) -> Option<HashSet<Utf8PathBuf>> {
    let Some(ext) = file.extension() else {
      // no extension
      return None;
    };
    let syntax = match ext {
      "ts" => Syntax::Typescript(Default::default()),
      "js" | "cjs" | "mjs" | "node" => Syntax::Es(Default::default()),
      _ => {
        // not js or ts file
        return None;
      }
    };
    let abs_file = match AbsPath::new(file.as_str()) {
      Ok(abs) => abs,
      Err(err) => {
        self.logger.warn(format!(
          "BuildDependencies: can't read file {file}.\n  - {err}"
        ));
        return None;
      }
    };
    let source = match self.resolver.inner_fs().read_to_string(&abs_file).await {
      Ok(source) => source,
      Err(err) => {
        self.logger.warn(format!(
          "BuildDependencies: can't read file {file}.\n  - {err}"
        ));
        return None;
      }
    };

    let visitor = {
      let mut visitor = DependencyVisitor::default();
      let javascript_compiler = JavaScriptCompiler::new();
      let ast = match javascript_compiler.parse(
        FileName::Custom(String::new()),
        source,
        EsVersion::EsNext,
        syntax,
        IsModule::Unknown,
        None,
      ) {
        Ok(ast) => ast,
        Err(err) => {
          self.logger.warn(format!(
            "BuildDependencies: can't parse {file}.\n  - {err:?}"
          ));
          return None;
        }
      };
      ast.visit(|program, _| {
        program.visit_with(&mut visitor);
      });
      visitor
    };

    let mut result = HashSet::default();
    let dirname = file.parent().expect("can not get parent dir");
    for req in visitor.requests {
      match self.resolver.resolve(dirname.as_std_path(), &req).await {
        Ok(ResolveResult::Resource(resource)) => {
          result.insert(resource.path);
          if let Some(data) = resource.description_data {
            let package_json_path = data.path();
            if is_node_package_path(package_json_path) {
              result.insert(package_json_path.join("package.json").assert_utf8());
            }
          }
        }
        Err(ResolveInnerError::RspackResolver(ResolveError::Builtin(_))) => {
          // builtin module ignore
        }
        Err(err) => {
          self.logger.warn(format!(
            "BuildDependencies: can't resolve {req} in {dirname}.\n  - {err}"
          ));
        }
        _ => {}
      };
    }

    Some(result)
  }

  /// Resolve a path.
  ///
  /// Use the corresponding resolve method according to the type of input file.
  pub async fn resolve(&mut self, path: &Utf8Path) -> Option<HashSet<Utf8PathBuf>> {
    let abs_path = match AbsPath::new(path.as_str()) {
      Ok(abs) => abs,
      Err(err) => {
        self.logger.warn(format!(
          "BuildDependencies: can't resolve {path}.\n  - {err}"
        ));
        return None;
      }
    };
    let metadata = match self.resolver.inner_fs().metadata(&abs_path).await {
      Ok(metadata) => metadata,
      Err(err) => {
        self.logger.warn(format!(
          "BuildDependencies: can't resolve {path}.\n  - {err}"
        ));
        return None;
      }
    };
    if metadata.is_directory {
      self.resolve_directory(path).await
    } else {
      self.resolve_file(path).await
    }
  }
}

#[cfg(test)]
mod test {
  use std::sync::Arc;

  use cow_utils::CowUtils;
  use rspack_fs::{AbsPath, MemoryFileSystem};
  use rspack_paths::Utf8PathBuf;

  use super::Helper;
  use crate::{CompilationLogger, CompilationLogging, LogType};

  fn test_str(posix: &str) -> String {
    if cfg!(windows) {
      let rest = posix.trim_start_matches('/').cow_replace('/', "\\");
      format!("C:\\{rest}")
    } else {
      posix.to_string()
    }
  }

  fn test_abs(posix: &str) -> AbsPath {
    AbsPath::new(&test_str(posix)).expect("valid test AbsPath")
  }

  fn test_logger(name: &str) -> (CompilationLogger, CompilationLogging) {
    let logging = CompilationLogging::default();
    (
      CompilationLogger::new(name.to_string(), logging.clone()),
      logging,
    )
  }

  fn warn_count(logging: &CompilationLogging, name: &str) -> usize {
    logging.get(name).map_or_default(|entries| {
      entries
        .iter()
        .filter(|entry| matches!(entry, LogType::Warn { .. }))
        .count()
    })
  }

  #[tokio::test]
  async fn helper_file_test() {
    let fs = Arc::new(MemoryFileSystem::default());
    fs.create_dir_all(&test_abs("/")).unwrap();
    fs.write(&test_abs("/a.js"), br#"console.log("a")"#)
      .unwrap();
    fs.write(&test_abs("/a1.jsx"), br#"console.log('a1')"#)
      .unwrap();
    fs.write(&test_abs("/b.js"), br#"console.log('b')"#)
      .unwrap();
    fs.write(&test_abs("/c.txt"), b"123").unwrap();
    fs.write(&test_abs("/e.ts"), br#"console.log("e")"#)
      .unwrap();
    fs.write(&test_abs("/e1.tsx"), br#"console.log("e1")"#)
      .unwrap();
    fs.write(&test_abs("/f.json"), br#"{"name":"f"}"#).unwrap();
    fs.write(&test_abs("/g.cjs"), br#"console.log("g")"#)
      .unwrap();
    fs.write(&test_abs("/h.mjs"), br#"console.log("h")"#)
      .unwrap();
    fs.write(&test_abs("/i.node"), b"").unwrap();
    fs.create_dir_all(&test_abs("/node_modules/lib1/")).unwrap();
    fs.write(
      &test_abs("/node_modules/lib1/package.json"),
      br#"{"name":"lib1","version": "1.0.0","main":"./index.js"}"#,
    )
    .unwrap();
    fs.write(&test_abs("/node_modules/lib1/index.js"), b"")
      .unwrap();
    fs.write(
      &test_abs("/package.json"),
      br#"{"name":"project","version": "1.0.0","main":"./index.js"}"#,
    )
    .unwrap();
    fs.write(
      &test_abs("/index.js"),
      r#"
import "./a";
import "./a1";
import "./b";

require("./c.txt");
require("./d.md");

require("./e");
require("./e1");
require("./f");
require("./g");
require("./h");
require("./i");
require("lib1");
"#
      .as_bytes(),
    )
    .unwrap();

    let (logger, logging) = test_logger("test");
    let mut helper = Helper::new(fs, logger);
    let entry = Utf8PathBuf::from(test_str("/index.js"));
    let deps = helper.resolve(&entry).await.expect("should have deps");
    assert_eq!(deps.len(), 10);
    assert_eq!(warn_count(&logging, "test"), 3);
  }

  #[tokio::test]
  async fn helper_dir_test() {
    let fs = Arc::new(MemoryFileSystem::default());
    fs.create_dir_all(&test_abs("/configs/test")).unwrap();
    fs.write(&test_abs("/configs/a.js"), br#"console.log('a')"#)
      .unwrap();
    fs.write(&test_abs("/configs/test/b.js"), br#"console.log('b')"#)
      .unwrap();
    fs.write(&test_abs("/configs/test/b1.js"), br#"console.log('b1')"#)
      .unwrap();
    fs.write(&test_abs("/configs/c.txt"), b"123").unwrap();
    fs.write(&test_abs("/index.js"), br#"console.log('index')"#)
      .unwrap();

    let (logger, logging) = test_logger("test");
    let mut helper = Helper::new(fs, logger);
    let dir = Utf8PathBuf::from(test_str("/configs/"));
    let deps = helper.resolve(&dir).await.expect("should have deps");
    assert_eq!(deps.len(), 3);
    assert_eq!(warn_count(&logging, "test"), 0);
  }
}
