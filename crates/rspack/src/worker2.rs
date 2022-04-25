use std::sync::Arc;

use crossbeam::{channel::Sender, queue::SegQueue};
use dashmap::DashSet;
use smol_str::SmolStr;
use swc_ecma_ast::{ModuleDecl, ModuleItem};
use swc_ecma_visit::VisitMutWith;
use tracing::instrument;

use crate::{
  js_module::{DependencyIdResolver, JsModule},
  module_graph::Msg,
  plugin_driver::PluginDriver,
  structs::ResolvedId,
  utils::{load, parse_file},
  visitors::dependency_scanner::DependencyScanner,
};

#[derive(Debug)]
pub struct Worker {
  pub job_queue: Arc<SegQueue<ResolvedId>>,
  pub tx: Sender<Msg>,
  pub visited_module_id: Arc<DashSet<SmolStr>>,
  pub plugin_driver: Arc<PluginDriver>,
}

impl Worker {
  fn fetch_job(&self) -> Option<ResolvedId> {
    self
      .job_queue
      .pop()
      .filter(|resolved_id| !self.visited_module_id.contains(&resolved_id.id))
      .map(|resolved_id| {
        self.visited_module_id.insert(resolved_id.id.clone());
        resolved_id
      })
  }

  #[instrument]
  pub async fn run(&mut self) {
    if let Some(resolved_id) = self.fetch_job() {
      if resolved_id.external {
        // TODO: external module
      } else {
        let id_resolver = DependencyIdResolver {
          module_id: resolved_id.id.clone(),
          resolved_ids: Default::default(),
          plugin_driver: self.plugin_driver.clone(),
        };

        let module_id: &str = &resolved_id.id;
        let source = load(module_id, &self.plugin_driver).await;
        let mut dependency_scanner = DependencyScanner::default();

        let mut ast = parse_file(source, &module_id);

        self.pre_analyze_imported_module(&id_resolver, &ast).await;

        ast.visit_mut_with(&mut dependency_scanner);

        for dyn_import in &dependency_scanner.dyn_dependencies {
          let resolved_id = id_resolver.resolve_id(&dyn_import.argument).await;

          self.job_queue.push(resolved_id);
        }
        let module = JsModule {
          exec_order: Default::default(),
          id: resolved_id.id.clone(),
          ast,
          dependencies: dependency_scanner.dependencies,
          dyn_dependencies: dependency_scanner.dyn_dependencies,
          is_user_defined_entry_point: Default::default(),
          dependency_resolver: id_resolver,
        };

        self.tx.send(Msg::NewMod(module)).unwrap()
      }
    }
  }

  // Fast path for analyzing static import and export.
  pub async fn pre_analyze_imported_module(
    &self,
    resolver: &DependencyIdResolver,
    ast: &swc_ecma_ast::Program,
  ) {
    for module_item in &ast.as_module().unwrap().body {
      if let ModuleItem::ModuleDecl(module_decl) = module_item {
        let mut depended = None;
        match module_decl {
          ModuleDecl::Import(import_decl) => {
            depended = Some(&import_decl.src.value);
          }
          ModuleDecl::ExportNamed(node) => {
            if let Some(source_node) = &node.src {
              depended = Some(&source_node.value);
            }
          }
          ModuleDecl::ExportAll(node) => {
            depended = Some(&node.src.value);
          }
          _ => {}
        }
        if let Some(depended) = depended {
          let resolved_id = resolver.resolve_id(depended).await;
          self.job_queue.push(resolved_id);
        }
      }
    }
  }
}
