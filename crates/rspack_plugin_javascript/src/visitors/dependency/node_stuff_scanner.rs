use rspack_core::{
  CompilerOptions, ConstDependency, DependencyLocation, DependencyTemplate, NodeOption,
  ResourceData, RuntimeGlobals, SpanExt,
};
use rustc_hash::FxHashSet;
use sugar_path::SugarPath;
use swc_core::common::SyntaxContext;
use swc_core::ecma::ast::Ident;
use swc_core::ecma::visit::{noop_visit_type, Visit, VisitWith};

use crate::no_visit_ignored_stmt;

const DIR_NAME: &str = "__dirname";
const FILE_NAME: &str = "__filename";
const GLOBAL: &str = "global";

pub struct NodeStuffScanner<'a> {
  pub presentational_dependencies: &'a mut Vec<Box<dyn DependencyTemplate>>,
  pub unresolved_ctxt: SyntaxContext,
  pub compiler_options: &'a CompilerOptions,
  pub node_option: &'a NodeOption,
  pub resource_data: &'a ResourceData,
  pub ignored: &'a mut FxHashSet<DependencyLocation>,
}

impl<'a> NodeStuffScanner<'a> {
  pub fn new(
    presentational_dependencies: &'a mut Vec<Box<dyn DependencyTemplate>>,
    unresolved_ctxt: SyntaxContext,
    compiler_options: &'a CompilerOptions,
    node_option: &'a NodeOption,
    resource_data: &'a ResourceData,
    ignored: &'a mut FxHashSet<DependencyLocation>,
  ) -> Self {
    Self {
      presentational_dependencies,
      unresolved_ctxt,
      compiler_options,
      node_option,
      resource_data,
      ignored,
    }
  }
}

impl Visit for NodeStuffScanner<'_> {
  noop_visit_type!();
  no_visit_ignored_stmt!();

  fn visit_ident(&mut self, ident: &Ident) {
    if ident.span.ctxt == self.unresolved_ctxt {
      match ident.sym.as_ref() as &str {
        DIR_NAME => {
          let dirname = match self.node_option.dirname.as_str() {
            "mock" => Some("/".to_string()),
            "warn-mock" => Some("/".to_string()),
            "true" => Some(
              self
                .resource_data
                .resource_path
                .parent()
                .expect("TODO:")
                .relative(&self.compiler_options.context)
                .to_string_lossy()
                .to_string(),
            ),
            _ => None,
          };
          if let Some(dirname) = dirname {
            self
              .presentational_dependencies
              .push(Box::new(ConstDependency::new(
                ident.span.real_lo(),
                ident.span.real_hi(),
                format!("'{dirname}'").into(),
                None,
              )));
          }
        }
        FILE_NAME => {
          let filename = match self.node_option.filename.as_str() {
            "mock" => Some("/index.js".to_string()),
            "warn-mock" => Some("/index.js".to_string()),
            "true" => Some(
              self
                .resource_data
                .resource_path
                .relative(&self.compiler_options.context)
                .to_string_lossy()
                .to_string(),
            ),
            _ => None,
          };
          if let Some(filename) = filename {
            self
              .presentational_dependencies
              .push(Box::new(ConstDependency::new(
                ident.span.real_lo(),
                ident.span.real_hi(),
                serde_json::to_string(&filename)
                  .expect("should render filename")
                  .into(),
                None,
              )));
          }
        }
        GLOBAL => {
          if matches!(self.node_option.global.as_str(), "true" | "warn") {
            self
              .presentational_dependencies
              .push(Box::new(ConstDependency::new(
                ident.span.real_lo(),
                ident.span.real_hi(),
                RuntimeGlobals::GLOBAL.name().into(),
                Some(RuntimeGlobals::GLOBAL),
              )));
          }
        }
        _ => {}
      }
    } else {
      ident.visit_children_with(self);
    }
  }
}
