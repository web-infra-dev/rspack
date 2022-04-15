use std::{
    sync::{Arc, Mutex},
};

use crossbeam::{channel::Sender, queue::SegQueue};
use dashmap::{ DashSet};
use smol_str::SmolStr;
use swc_ecma_ast::{ModuleDecl, ModuleItem};
use swc_ecma_visit::VisitMutWith;

use crate::{
    module_graph_container::{Msg, Rel},
    module::Module,
    plugin_driver::PluginDriver,
    scanner::{scope::BindType, Scanner},
    mark_box::MarkBox,
    types::ResolvedId,
    utils::{load, parse_file},
};

pub struct Worker {
    pub symbol_box: Arc<Mutex<MarkBox>>,
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

    pub async fn run(&mut self) {
        if let Some(resolved_id) = self.fetch_job() {
            if resolved_id.external {
                // TODO: external module
            } else {
                let mut module = Module::new(resolved_id.id.clone());
                let id: &str = &resolved_id.id;
                let source = load(id, &self.plugin_driver).await;
                let mut ast = parse_file(source, &module.id);
                self.pre_analyze_imported_module(&mut module, &ast).await;

                let mut scanner = Scanner::new(self.symbol_box.clone(), self.tx.clone());
                ast.visit_mut_with(&mut scanner);

                for (imported, info) in &scanner.import_infos {
                    let resolved_id = module.resolve_id(imported, &self.plugin_driver).await;
                    self.tx
                        .send(Msg::DependencyReference(
                            module.id.clone(),
                            resolved_id.id,
                            info.clone().into(),
                        ))
                        .unwrap()
                }
                for (re_exported, info) in &scanner.re_export_infos {
                    let resolved_id = module.resolve_id(re_exported, &self.plugin_driver).await;
                    self.tx
                        .send(Msg::DependencyReference(
                            module.id.clone(),
                            resolved_id.id,
                            info.clone().into(),
                        ))
                        .unwrap()
                }
                for re_exported in &scanner.export_all_sources {
                    let resolved_id = module.resolve_id(re_exported, &self.plugin_driver).await;
                    self.tx
                        .send(Msg::DependencyReference(
                            module.id.clone(),
                            resolved_id.id,
                            Rel::ReExportAll,
                        ))
                        .unwrap()
                }
                module.dependencies = scanner.dependencies;
                module.dyn_dependencies = scanner.dyn_dependencies;
                module.local_exports = scanner.local_exports;
                module.re_exports = scanner.re_exports;
                module.re_export_all_sources =
                    scanner.export_all_sources.into_iter().map(|s| s).collect();
                {
                    let root_scope = scanner.stacks.into_iter().next().unwrap();
                    let declared_symbols = root_scope.declared_symbols;
                    let mut declared_symbols_kind = root_scope.declared_symbols_kind;
                    declared_symbols.into_iter().for_each(|(name, mark)| {
                        let bind_type = declared_symbols_kind.remove(&name).unwrap();
                        if BindType::Import == bind_type {
                            module.imported_symbols.insert(name, mark);
                        } else {
                            module.declared_symbols.insert(name, mark);
                        }
                    });
                }
                module.namespace.mark = self
                    .symbol_box
                    .lock()
                    .unwrap()
                    // .unwrap()
                    .new_mark();

                module.set_statements(ast);

                module.bind_local_references(&mut self.symbol_box.lock().unwrap());

                module.link_local_exports();

                log::debug!("[worker]: emit module {:#?}", module);
                self.tx.send(Msg::NewMod(Box::new(module))).unwrap()
            }
        }
    }

    // Fast path for analyzing static import and export.
    pub async fn pre_analyze_imported_module(
        &self,
        module: &mut Module,
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
                    let resolved_id = module.resolve_id(depended, &self.plugin_driver).await;
                    self.job_queue.push(resolved_id);
                }
            }
        }
    }
}
