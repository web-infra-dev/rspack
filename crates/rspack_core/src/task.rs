use std::{
  path::Path,
  sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
  },
};

use anyhow::Result;
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

use crate::{
  bundle::Msg, dependency_scanner::DependencyScanner, plugin_hook, utils::parse_file, ImportKind,
  JsModule, JsModuleKind, LoadArgs, PluginDriver, ResolveArgs, ResolvedURI,
};
use crate::{get_swc_compiler, path::normalize_path};

#[derive(Debug)]
pub struct DependencyIdResolver {
  pub module_id: String,
  pub resolved_ids: DashMap<JsWord, ResolvedURI>,
  pub plugin_driver: Arc<PluginDriver>,
}

impl DependencyIdResolver {
  pub async fn resolve(&self, dep_src: &JsWord, kind: ImportKind) -> Result<ResolvedURI> {
    let resolved_id;
    if let Some(cached) = self.resolved_ids.get(dep_src) {
      resolved_id = cached.clone();
    } else {
      resolved_id = plugin_hook::resolve_id(
        ResolveArgs {
          id: dep_src.to_string(),
          importer: Some(self.module_id.clone()),
          kind,
        },
        false,
        &self.plugin_driver,
      )
      .await?;
      self
        .resolved_ids
        .insert(dep_src.clone(), resolved_id.clone());
    }
    Ok(resolved_id)
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
}

impl Task {
  #[instrument(skip(self))]
  pub async fn run(&mut self) -> Result<()> {
    let resolved_uri = self.resolved_uri.clone();
    if resolved_uri.external {
    } else {
      tracing::trace!("start process {:?}", resolved_uri);
      let uri_resolver = DependencyIdResolver {
        module_id: resolved_uri.uri.clone(),
        resolved_ids: Default::default(),
        plugin_driver: self.plugin_driver.clone(),
      };

      let module_id: &str = &resolved_uri.uri;
      let (source, mut loader) = plugin_hook::load(
        LoadArgs {
          kind: resolved_uri.kind,
          id: module_id.to_string(),
        },
        &self.plugin_driver,
      )
      .await?;
      let transformed_source =
        plugin_hook::transform(module_id, &mut loader, source, &self.plugin_driver)?;
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
      let mut ast = plugin_hook::transform_ast(Path::new(module_id), raw_ast, &self.plugin_driver)?;

      self
        .pre_analyze_imported_module(&uri_resolver, &ast)
        .await?;

      ast.visit_mut_with(&mut dependency_scanner);

      for (import, _) in &dependency_scanner.dependencies {
        let resolved_id = uri_resolver.resolve(import, ImportKind::Import).await?;
        self.spawn_new_task(resolved_id);
      }

      for dyn_import in &dependency_scanner.dyn_dependencies {
        let resolved_id = uri_resolver
          .resolve(&dyn_import.argument, ImportKind::DynamicImport)
          .await?;
        self.spawn_new_task(resolved_id);
      }

      let module = JsModule {
        kind: JsModuleKind::Normal,
        exec_order: Default::default(),
        uri: resolved_uri.uri.clone(),
        id: normalize_path(
          resolved_uri.uri.clone().as_str(),
          self.root.clone().as_str(),
        ),
        ast,
        dependencies: dependency_scanner.dependencies,
        dyn_imports: dependency_scanner.dyn_dependencies,
        resolved_uris: uri_resolver
          .resolved_ids
          .into_iter()
          .map(|(key, value)| (key, value))
          .collect(),
        loader: *loader,
        cached_output: Default::default(),
      };
      self.tx.send(Msg::TaskFinished(module))?
    }

    Ok(())
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
      };
      let tx = self.tx.clone();
      tokio::task::spawn(async move {
        if let Err(err) = task.run().await {
          tx.send(Msg::TaskErrorEncountered(err))
            .expect("failed to send task error");
        }
      });
    }
  }

  // Fast path for analyzing static import and export.
  pub async fn pre_analyze_imported_module(
    &self,
    resolver: &DependencyIdResolver,
    ast: &ast::Module,
  ) -> Result<()> {
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
          let uri = resolver.resolve(depended, ImportKind::Import).await?;
          self.spawn_new_task(uri);
        }
      }
    }
    Ok(())
  }
}
