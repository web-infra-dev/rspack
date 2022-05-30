use crate::{
  bundle::Msg, dependency_scanner::DependencyScanner, plugin_hook, utils::parse_file, ImportKind,
  JsModule, JsModuleKind, LoadArgs, ModuleGraph, ModuleGraphContainer, PluginDriver, ResolveArgs,
  ResolvedURI,
};
use crate::{get_swc_compiler, path::normalize_path};
use dashmap::{DashMap, DashSet};
use nodejs_resolver::Resolver;
use rspack_swc::{
  swc_ecma_ast::{self as ast},
  swc_ecma_transforms_base,
  swc_ecma_visit::{self},
};
use std::{
  path::Path,
  sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
  },
};
use swc_ecma_transforms_base::resolver;
use swc_ecma_visit::VisitMutWith;
use tokio::sync::mpsc::UnboundedSender;
use tracing::instrument;

#[derive(Debug)]
pub struct Task {
  pub root: String,
  pub resolve_args: ResolveArgs,
  pub active_task_count: Arc<AtomicUsize>,
  pub tx: UnboundedSender<Msg>,
  pub visited_module_uri: Arc<DashSet<String>>,
  pub plugin_driver: Arc<PluginDriver>,
  pub resolver: Arc<Resolver>,
}

impl Task {
  #[instrument(skip(self))]
  pub async fn run(&mut self) {
    let resolve_args = self.resolve_args.clone();
    tracing::trace!("start process {:?}", resolve_args);

    let resolved =
      plugin_hook::resolve_id(&resolve_args, false, &self.plugin_driver, &self.resolver).await;

    self
      .tx
      .send(Msg::DependencyReference((
        resolve_args.clone(),
        ResolvedURI {
          uri: resolved.uri.clone(),
          kind: resolve_args.kind.clone(),
          external: resolved.external,
        },
      )))
      .unwrap();

    let module_id: &str = &resolved.uri;

    if resolved.external || self.visited_module_uri.contains(module_id) {
      self.tx.send(Msg::CanCel()).unwrap();
      return;
    }
    self.visited_module_uri.insert(module_id.to_string());

    let (content, mut loader) = if let Some(source) = resolved.source {
      (source.content, source.loader)
    } else {
      let (content, loader) = plugin_hook::load(
        LoadArgs {
          id: resolved.uri.to_string(),
        },
        &self.plugin_driver,
      )
      .await;
      (content, loader)
    };

    let transformed_source =
      plugin_hook::transform(module_id, &mut loader, content.clone(), &self.plugin_driver);
    let loader = loader
      .as_ref()
      .unwrap_or_else(|| panic!("No loader to deal with file: {:?}", module_id));
    let mut dependency_scanner = DependencyScanner::default();
    let mut raw_ast = parse_file(transformed_source, module_id, loader).expect_module();
    {
      // The Resolver is not send. We need this block to tell compiler that
      // the Resolver won't be sent over the threads
      get_swc_compiler().run(|| {
        let mut syntax_context_resolver = resolver(
          self.plugin_driver.ctx.unresolved_mark,
          self.plugin_driver.ctx.top_level_mark,
          false,
        );
        raw_ast.visit_mut_with(&mut syntax_context_resolver);
      })
    }
    let mut ast = plugin_hook::transform_ast(Path::new(module_id), raw_ast, &self.plugin_driver);

    self.pre_analyze_imported_module(&ast, module_id).await;

    ast.visit_mut_with(&mut dependency_scanner);

    for (import, _) in &dependency_scanner.dependencies {
      self.spawn_new_task(ResolveArgs {
        id: import.to_string(),
        importer: Some(module_id.to_string()),
        kind: ImportKind::Import,
      });
    }

    for dyn_import in &dependency_scanner.dyn_dependencies {
      self.spawn_new_task(ResolveArgs {
        id: dyn_import.argument.to_string(),
        importer: Some(module_id.to_string()),
        kind: ImportKind::DynamicImport,
      });
    }

    let module = JsModule {
      kind: JsModuleKind::Normal,
      exec_order: Default::default(),
      uri: module_id.to_string(),
      id: normalize_path(module_id, self.root.clone().as_str()),
      ast,
      dependencies: dependency_scanner.dependencies,
      dyn_imports: dependency_scanner.dyn_dependencies,
      resolved_uris: Default::default(),
      loader: *loader,
      cached_output: Default::default(),
    };
    self.tx.send(Msg::TaskFinished(module)).unwrap()
  }

  pub fn spawn_new_task(&self, resolve_args: ResolveArgs) {
    self.active_task_count.fetch_add(1, Ordering::SeqCst);
    let mut task = Task {
      root: self.root.clone(),
      resolve_args,
      active_task_count: self.active_task_count.clone(),
      visited_module_uri: self.visited_module_uri.clone(),
      tx: self.tx.clone(),
      plugin_driver: self.plugin_driver.clone(),
      resolver: self.resolver.clone(),
    };
    tokio::task::spawn(async move {
      task.run().await;
    });
  }

  // Fast path for analyzing static import and export.
  pub async fn pre_analyze_imported_module(&self, ast: &ast::Module, module_id: &str) {
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
          self.spawn_new_task(ResolveArgs {
            id: depended.to_string(),
            importer: Some(module_id.to_string()),
            kind: ImportKind::Import,
          });
        }
      }
    }
  }
}
