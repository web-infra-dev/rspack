mod visitor;

use std::sync::Arc;

use indoc::formatdoc;
use rspack_fs::ReadableFileSystem;
use rspack_javascript_compiler::JavaScriptCompiler;
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rustc_hash::FxHashSet as HashSet;
use swc_core::{
  base::config::IsModule,
  common::FileName,
  ecma::{ast::EsVersion, parser::Syntax},
};

use self::visitor::DependencyVisitor;
use crate::{Resolve as ResolveOption, ResolveResult, Resolver};

pub struct Helper {
  resolver: Resolver,
  warnings: Vec<String>,
}

impl Helper {
  pub fn new(fs: Arc<dyn ReadableFileSystem>) -> Self {
    Helper {
      resolver: Resolver::new(
        ResolveOption {
          condition_names: Some(vec!["import".into(), "require".into(), "node".into()]),
          exports_fields: Some(vec![vec!["export".into()]]),
          ..Default::default()
        },
        fs,
      ),
      warnings: Default::default(),
    }
  }

  async fn resolve_directory(&mut self, dir: &Utf8Path) -> Option<HashSet<Utf8PathBuf>> {
    let entries = self.resolver.inner_fs().read_dir(dir).await;
    match entries {
      Ok(entries) => Some(entries.into_iter().map(|item| dir.join(item)).collect()),
      Err(err) => {
        self.warnings.push(formatdoc!(
          r#"
            BuildDependencies: can't read directory {dir}.
            - {err}
          "#,
        ));
        None
      }
    }
  }

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
        self.warnings.push(formatdoc!(
          r#"
            BuildDependencies: can't read file ${file}.
            - {err}
          "#,
        ));
        return None;
      }
    };
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
        self.warnings.push(formatdoc!(
          r#"
            BuildDependencies: can't parse {file}.
            - {err:?}
          "#,
        ));
        return None;
      }
    };
    ast.visit(|program, _| {
      program.visit_with(&mut visitor);
    });

    let mut result = HashSet::default();
    let dirname = file.parent().expect("can not get parent dir");
    for req in visitor.requests {
      match self.resolver.resolve(dirname.as_std_path(), &req).await {
        Ok(ResolveResult::Resource(resource)) => {
          result.insert(resource.path);
        }
        Err(err) => {
          self.warnings.push(formatdoc!(
            r#"
              BuildDependencies: can't resolve {req} in {dirname}.
              - {err}
            "#,
          ));
        }
        _ => {}
      };
    }

    Some(result)
  }

  pub async fn resolve(&mut self, path: &Utf8Path) -> Option<HashSet<Utf8PathBuf>> {
    let metadata = match self.resolver.inner_fs().metadata(path).await {
      Ok(metadata) => metadata,
      Err(err) => {
        self.warnings.push(formatdoc!(
          r#"
            BuildDependencies: can't resolve {path}.
            - {err}
          "#,
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

  pub fn into_warnings(self) -> Vec<String> {
    self.warnings
  }
}
