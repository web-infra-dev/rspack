use std::sync::Arc;

use crossbeam::{channel::Sender, queue::SegQueue};
use dashmap::DashSet;
use smol_str::SmolStr;
use swc::ecmascript::ast::ModuleItem;
use swc_ecma_ast::ModuleDecl;
use swc_ecma_visit::{VisitMutWith, VisitWith};

use crate::{
    js_ext_module::JsExtModule, js_module::JsModule, plugin::ResolvedId, utils::parse_file,
    visitors::DependencyScanner, Msg, PluginDriver, Relation,
};

pub(crate) struct Worker {
    pub job_queue: Arc<SegQueue<ResolvedId>>,
    pub tx: Sender<Msg>,
    pub processed_id: Arc<DashSet<SmolStr>>,
    pub plugin_driver: Arc<PluginDriver>,
}

impl Worker {
    fn fetch_job(&self) -> Option<ResolvedId> {
        self.job_queue
            .pop()
            .filter(|resolved_id| !self.processed_id.contains(&resolved_id.id))
            .map(|resolved_id| {
                self.processed_id.insert(resolved_id.id.clone());
                resolved_id
            })
    }

    pub async fn run(&self) {
        if let Some(resolved_id) = self.fetch_job() {
            if resolved_id.external {
                let mut js_ext_module = JsExtModule::new(resolved_id.id.clone());
            } else {
                let mut js_module = JsModule::new(resolved_id.id.clone());
                let source = self.plugin_driver.load(&js_module.id).await;
                js_module.source = source.clone();
                let mut ast = parse_file(source, &resolved_id.id);
                self.pre_analyze_imported_module(&js_module, &ast).await;
                let mut dependenecy_scanner = DependencyScanner::new(&self.tx, &js_module);
                ast.visit_children_with(&mut dependenecy_scanner);

                for imported in dependenecy_scanner.dependencies.keys() {
                    let resolved_id = js_module.resolve_id(&self.plugin_driver, imported).await;
                    self.tx
                        .send(Msg::DependencyReference(
                            js_module.id.clone(),
                            resolved_id.id,
                            Relation::StaticImport,
                        ))
                        .unwrap();
                }
                for dyn_imported in &dependenecy_scanner.dynamic_dependencies {
                    let resolved_id = js_module
                        .resolve_id(&self.plugin_driver, &dyn_imported.argument)
                        .await;
                    self.tx
                        .send(Msg::DependencyReference(
                            js_module.id.clone(),
                            resolved_id.id,
                            Relation::AsyncImport,
                        ))
                        .unwrap();
                }
                println!("js_module {:#?}", js_module);
                self.tx.send(Msg::NewMod(js_module)).unwrap();
            }
        }
    }
    // Fast path for analyzing static import and export.
    pub async fn pre_analyze_imported_module(
        &self,
        js_module: &JsModule,
        ast: &swc_ecma_ast::Module,
    ) {
        for module_item in &ast.body {
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
                    let resolved_id = js_module
                        .resolve_id(&self.plugin_driver, &depended.to_string())
                        .await;
                    self.job_queue.push(resolved_id);
                }
            }
        }
    }
}
