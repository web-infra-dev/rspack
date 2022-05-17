use std::{
  path::Path,
  sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
  },
};

use crate::{
  bundle::Msg, dependency_scanner::DependencyScanner, plugin_hook, utils::parse_file, JsModule,
  PluginDriver, ResolvedURI,
};
use crate::{get_swc_compiler, path::normalize_path};
use dashmap::{DashMap, DashSet};
use rspack_swc::{
  swc_atoms,
  swc_ecma_ast::{self as ast},
  swc_ecma_transforms_base,
  swc_ecma_visit::{self},
};
use swc_atoms::JsWord;
use swc_ecma_transforms_base::resolver;
use swc_ecma_visit::VisitMutWith;
use tokio::sync::mpsc::UnboundedSender;
use tracing::instrument;

#[derive(Debug)]
pub struct DependencyIdResolver {
  pub module_id: String,
  pub resolved_ids: DashMap<JsWord, ResolvedURI>,
  pub plugin_driver: Arc<PluginDriver>,
}

impl DependencyIdResolver {
  pub async fn resolve(&self, dep_src: &JsWord) -> ResolvedURI {
    let resolved_id;
    if let Some(cached) = self.resolved_ids.get(dep_src) {
      resolved_id = cached.clone();
    } else {
      resolved_id =
        plugin_hook::resolve_id(dep_src, Some(&self.module_id), false, &self.plugin_driver).await;
      self
        .resolved_ids
        .insert(dep_src.clone(), resolved_id.clone());
    }
    resolved_id
  }
}

#[derive(Debug)]
pub struct Task {
  pub root: String,
  pub resolved_uri: ResolvedURI,
  pub active_task_count: Arc<AtomicUsize>,
  pub tx: UnboundedSender<Msg>,
  pub visited_module_uri: Arc<DashSet<String>>,
  pub plugin_driver: Arc<PluginDriver>,
  pub code_splitting: bool,
}

impl Task {
  #[instrument(skip(self))]
  pub async fn run(&mut self) {
    let resolved_uri = self.resolved_uri.clone();
    if resolved_uri.external {
      // TODO: external module
    } else {
      tracing::trace!("start process {:?}", resolved_uri);
      let uri_resolver = DependencyIdResolver {
        module_id: resolved_uri.uri.clone(),
        resolved_ids: Default::default(),
        plugin_driver: self.plugin_driver.clone(),
      };

      let module_id: &str = &resolved_uri.uri;
      let (source, mut loader) = plugin_hook::load(module_id, &self.plugin_driver).await;
      let transformed_source = self
        .plugin_driver
        .transform_raw(module_id, &mut loader, source);
      let mut dependency_scanner = DependencyScanner::default();
      let mut raw_ast = parse_file(transformed_source, module_id, &loader).expect_module();
      {
        // The Resolver is not send. We need this block to tell compiler that
        // the Resolver won't be sent over the threads
        get_swc_compiler().run(|| {
          let mut syntax_context_resolver = resolver(
            self.plugin_driver.ctx.unresolved_mark.clone(),
            self.plugin_driver.ctx.top_level_mark.clone(),
            false,
          );
          raw_ast.visit_mut_with(&mut syntax_context_resolver);
        })
      }
      let mut ast = plugin_hook::transform(Path::new(module_id), raw_ast, &self.plugin_driver);

      self.pre_analyze_imported_module(&uri_resolver, &ast).await;

      ast.visit_mut_with(&mut dependency_scanner);

      for dyn_import in &dependency_scanner.dyn_dependencies {
        let resolved_id = uri_resolver.resolve(&dyn_import.argument).await;

        self.spawn_new_task(resolved_id);
      }
      for (import, _) in &dependency_scanner.dependencies {
        let resolved_id = uri_resolver.resolve(import).await;
        self.spawn_new_task(resolved_id);
      }
      let module = JsModule {
        exec_order: Default::default(),
        uri: resolved_uri.uri.clone(),
        id: normalize_path(
          resolved_uri.uri.clone().as_str(),
          self.root.clone().as_str(),
        )
        .into(),
        ast,
        dependencies: dependency_scanner.dependencies,
        dyn_imports: dependency_scanner.dyn_dependencies,
        is_user_defined_entry_point: Default::default(),
        chunkd_ids: Default::default(),
        resolved_uris: uri_resolver
          .resolved_ids
          .into_iter()
          .map(|(key, value)| (key, value))
          .collect(),
        code_splitting: self.code_splitting,
      };
      self.tx.send(Msg::TaskFinished(module)).unwrap()
    }
  }

  pub fn spawn_new_task(&self, resolved_uri: ResolvedURI) {
    if !self.visited_module_uri.contains(&resolved_uri.uri) {
      self.visited_module_uri.insert(resolved_uri.uri.clone());
      self.active_task_count.fetch_add(1, Ordering::SeqCst);
      let mut task = Task {
        root: self.root.clone(),
        resolved_uri,
        active_task_count: self.active_task_count.clone(),
        visited_module_uri: self.visited_module_uri.clone(),
        tx: self.tx.clone(),
        plugin_driver: self.plugin_driver.clone(),
        code_splitting: self.code_splitting,
      };
      tokio::task::spawn(async move {
        task.run().await;
      });
    }
  }

  // Fast path for analyzing static import and export.
  pub async fn pre_analyze_imported_module(
    &self,
    resolver: &DependencyIdResolver,
    ast: &ast::Module,
  ) {
    for module_item in &ast.body {
      if let ast::ModuleItem::ModuleDecl(module_decl) = module_item {
        let mut depended = None;
        match module_decl {
          ast::ModuleDecl::Import(import_decl) => {
            depended = Some(&import_decl.src.value);
          }
          ast::ModuleDecl::ExportNamed(node) => {
            if let Some(source_node) = &node.src {
              depended = Some(&source_node.value);
            }
          }
          ast::ModuleDecl::ExportAll(node) => {
            depended = Some(&node.src.value);
          }
          _ => {}
        }
        if let Some(depended) = depended {
          let uri = resolver.resolve(depended).await;
          self.spawn_new_task(uri);
        }
      }
    }
  }
}
