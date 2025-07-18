mod error;
mod visitor;

use std::{collections::VecDeque, sync::Arc};

use rspack_error::{
  miette::{diagnostic, Error as MietteError},
  Diagnostic, DiagnosticExt,
};
use rspack_fs::ReadableFileSystem;
use rspack_javascript_compiler::JavaScriptCompiler;
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rustc_hash::FxHashSet as HashSet;
use swc_core::{
  base::config::IsModule,
  common::FileName,
  ecma::{ast::EsVersion, parser::Syntax},
};

use self::{error::BuildDependencyError, visitor::DependencyVisitor};
use crate::{Resolve as ResolveOption, ResolveResult, Resolver};

struct Helper {
  resolver: Resolver,
  diagnostics: Vec<Diagnostic>,
}

impl Helper {
  fn new(fs: Arc<dyn ReadableFileSystem>) -> Self {
    Helper {
      resolver: Resolver::new(
        ResolveOption {
          condition_names: Some(vec!["import".into(), "require".into(), "node".into()]),
          exports_fields: Some(vec![vec!["export".into()]]),
          ..Default::default()
        },
        fs,
      ),
      diagnostics: Default::default(),
    }
  }

  async fn resolve_directory(&mut self, dir: &Utf8Path) -> Option<HashSet<Utf8PathBuf>> {
    let entries = self.resolver.inner_fs().read_dir(dir).await;
    match entries {
      Ok(entries) => Some(entries.into_iter().map(|item| dir.join(item)).collect()),
      Err(err) => {
        self.diagnostics.push(
          BuildDependencyError::new(MietteError::from(err))
            .boxed()
            .into(),
        );
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
        self.diagnostics.push(
          BuildDependencyError::new(MietteError::from(err))
            .boxed()
            .into(),
        );
        return None;
      }
    };
    let mut visitor = DependencyVisitor::default();
    let javascript_compiler = JavaScriptCompiler::new();
    let Ok(ast) = javascript_compiler.parse(
      FileName::Custom("".into()),
      source,
      EsVersion::EsNext,
      syntax,
      IsModule::Unknown,
      None,
    ) else {
      self.diagnostics.push(
        BuildDependencyError::new(diagnostic!("parse {} failed", file).into())
          .boxed()
          .into(),
      );
      return None;
    };
    ast.visit(|program, _| {
      program.visit_with(&mut visitor);
    });

    let mut result = HashSet::default();
    let dirname = file.parent().expect("can not get parent dir");
    for req in visitor.requests {
      let Ok(res) = self.resolver.resolve(dirname.as_std_path(), &req).await else {
        self.diagnostics.push(
          BuildDependencyError::new(diagnostic!("can't resolve {} in {}", req, dirname).into())
            .boxed()
            .into(),
        );
        continue;
      };
      match res {
        ResolveResult::Resource(resource) => {
          result.insert(resource.path);
        }
        _ => {}
      }
    }

    Some(result)
  }

  async fn resolve(&mut self, path: &Utf8Path) -> Option<HashSet<Utf8PathBuf>> {
    let metadata = match self.resolver.inner_fs().metadata(path).await {
      Ok(metadata) => metadata,
      Err(err) => {
        self.diagnostics.push(
          BuildDependencyError::new(MietteError::from(err))
            .boxed()
            .into(),
        );
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

pub async fn resolve_build_dependencies(
  fs: Arc<dyn ReadableFileSystem>,
  build_dependencies: HashSet<Utf8PathBuf>,
) -> (HashSet<Utf8PathBuf>, Vec<Diagnostic>) {
  let mut helper = Helper::new(fs.clone());
  let mut resolved_files = HashSet::default();
  let mut queue = VecDeque::new();
  queue.extend(build_dependencies);
  loop {
    let Some(current) = queue.pop_front() else {
      break;
    };
    if resolved_files.contains(&current) {
      continue;
    }
    if let Some(childs) = helper.resolve(&current).await {
      queue.extend(childs);
    }
    resolved_files.insert(current);
  }
  (resolved_files, helper.diagnostics)
}
