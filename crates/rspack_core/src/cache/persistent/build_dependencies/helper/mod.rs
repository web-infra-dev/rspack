mod utils;
mod visitor;

use std::sync::Arc;

use rspack_fs::ReadableFileSystem;
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
pub struct Helper {
  resolver: Resolver,
  logger: CompilationLogger,
}

impl Helper {
  pub fn new(fs: Arc<dyn ReadableFileSystem>, logger: CompilationLogger) -> Self {
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
    let entries = self.resolver.inner_fs().read_dir(dir).await;
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
    let source = match self.resolver.inner_fs().read_to_string(file).await {
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
    let metadata = match self.resolver.inner_fs().metadata(path).await {
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

  use rspack_fs::{MemoryFileSystem, WritableFileSystem};

  use super::Helper;
  use crate::{CompilationLogger, CompilationLogging, LogType};

  fn test_logger(name: &str) -> (CompilationLogger, CompilationLogging) {
    let logging = CompilationLogging::default();
    (
      CompilationLogger::new(name.to_string(), logging.clone()),
      logging,
    )
  }

  fn warn_count(logging: &CompilationLogging, name: &str) -> usize {
    logging
      .get(name)
      .map(|entries| {
        entries
          .iter()
          .filter(|entry| matches!(entry, LogType::Warn { .. }))
          .count()
      })
      .unwrap_or_default()
  }

  #[tokio::test]
  async fn helper_file_test() {
    let fs = Arc::new(MemoryFileSystem::default());
    fs.create_dir_all("/".into()).await.unwrap();
    fs.write("/a.js".into(), r#"console.log("a")"#.as_bytes())
      .await
      .unwrap();
    fs.write("/a1.jsx".into(), r#"console.log('a1')"#.as_bytes())
      .await
      .unwrap();
    fs.write("/b.js".into(), r#"console.log('b')"#.as_bytes())
      .await
      .unwrap();
    fs.write("/c.txt".into(), r#"123"#.as_bytes())
      .await
      .unwrap();
    fs.write("/e.ts".into(), r#"console.log("e")"#.as_bytes())
      .await
      .unwrap();
    fs.write("/e1.tsx".into(), r#"console.log("e1")"#.as_bytes())
      .await
      .unwrap();
    fs.write("/f.json".into(), r#"{"name":"f"}"#.as_bytes())
      .await
      .unwrap();
    fs.write("/g.cjs".into(), r#"console.log("g")"#.as_bytes())
      .await
      .unwrap();
    fs.write("/h.mjs".into(), r#"console.log("h")"#.as_bytes())
      .await
      .unwrap();
    fs.write("/i.node".into(), r#""#.as_bytes()).await.unwrap();
    fs.create_dir_all("/node_modules/lib1/".into())
      .await
      .unwrap();
    fs.write(
      "/node_modules/lib1/package.json".into(),
      r#"{"name":"lib1","version": "1.0.0","main":"./index.js"}"#.as_bytes(),
    )
    .await
    .unwrap();
    fs.write("/node_modules/lib1/index.js".into(), r#""#.as_bytes())
      .await
      .unwrap();
    fs.write(
      "/package.json".into(),
      r#"{"name":"project","version": "1.0.0","main":"./index.js"}"#.as_bytes(),
    )
    .await
    .unwrap();
    fs.write(
      "/index.js".into(),
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
    .await
    .unwrap();

    let (logger, logging) = test_logger("test");
    let mut helper = Helper::new(fs, logger);
    let deps = helper
      .resolve("/index.js".into())
      .await
      .expect("should have deps");
    assert_eq!(deps.len(), 10);
    assert_eq!(warn_count(&logging, "test"), 3);
  }

  #[tokio::test]
  async fn helper_dir_test() {
    let fs = Arc::new(MemoryFileSystem::default());
    fs.create_dir_all("/configs/test".into()).await.unwrap();
    fs.write("/configs/a.js".into(), r#"console.log('a')"#.as_bytes())
      .await
      .unwrap();
    fs.write(
      "/configs/test/b.js".into(),
      r#"console.log('b')"#.as_bytes(),
    )
    .await
    .unwrap();
    fs.write(
      "/configs/test/b1.js".into(),
      r#"console.log('b1')"#.as_bytes(),
    )
    .await
    .unwrap();
    fs.write("/configs/c.txt".into(), r#"123"#.as_bytes())
      .await
      .unwrap();
    fs.write("/index.js".into(), r#"console.log('index')"#.as_bytes())
      .await
      .unwrap();

    let (logger, logging) = test_logger("test");
    let mut helper = Helper::new(fs, logger);
    let deps = helper
      .resolve("/configs/".into())
      .await
      .expect("should have deps");
    assert_eq!(deps.len(), 3);
    assert_eq!(warn_count(&logging, "test"), 0);
  }
}
