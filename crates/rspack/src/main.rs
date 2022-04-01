#![feature(async_closure)]

use std::{
    collections::HashMap,
    fmt::Debug,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use crossbeam::channel::{self};
use crossbeam::queue::SegQueue;
use dashmap::DashSet;
use futures::future::join_all;
use js_module::JsModule;
use petgraph::{dot::Dot, graph::NodeIndex};
use plugin::{Plugin, ResolvedId};
use smol_str::SmolStr;
use sugar_path::PathSugar;
use swc::{common::FileName, config::IsModule};
use swc_ecma_parser::{EsConfig, Syntax, TsConfig};
use utils::get_compiler;
use worker::Worker;
mod js_ext_module;

use crate::plugin::ResolveExtensionPlugin;
mod utils;

mod js_module;
mod plugin;
mod visitors;
mod bundler;

#[derive(Debug, Clone, Copy)]
pub enum Relation {
    AsyncImport,
    StaticImport,
}

type DepGraph = petgraph::Graph<SmolStr, Relation>;

pub enum Msg {
    DependencyReference(SmolStr, SmolStr, Relation),
    NewMod(JsModule),
    // NewExtMod(ExternalModule),
}

#[derive(Debug)]
struct PluginDriver {
    plugins: Vec<Box<dyn Plugin>>,
}

#[inline]
pub fn is_external_module(source: &str) -> bool {
    source.starts_with("node:") || (!Path::new(source).is_absolute() && !source.starts_with('.'))
}

impl PluginDriver {
    async fn resolve_id(&self, importer: Option<&str>, importee: &str) -> ResolvedId {
        let mut result = None;
        for plugin in &self.plugins {
            let res = plugin.resolve(importer, importee).await;
            if res.is_some() {
                result = res;
            }
        }
        result.unwrap_or_else(|| {
            if importer.is_some() && is_external_module(importee) {
                ResolvedId::new(importee.to_string(), true)
            } else {
                let id = if let Some(importer) = importer {
                    Path::new(importer)
                        .parent()
                        .unwrap()
                        .join(importee)
                        .resolve()
                    // nodejs_path::resolve!(&nodejs_path::dirname(importer), source)
                } else {
                    Path::new(importee).resolve()
                };
                ResolvedId::new(id.to_string_lossy().to_string(), false)
            }
        })
    }

    async fn load(&self, id: &str) -> String {
        for plugin in &self.plugins {
            let res = plugin.load(id).await;
            if res.is_some() {
                return res.unwrap();
            }
        }
        tokio::fs::read_to_string(id)
            .await
            .expect(&format!("{:?} is not exsit", id))
    }
}

#[derive(Debug)]
struct InputOptions {
    entries: Vec<String>,
}

struct GraphContainer {
    plugin_driver: Arc<PluginDriver>,
    resolved_entries: Vec<ResolvedId>,
    module_by_id: HashMap<SmolStr, JsModule>,
    input: InputOptions,
}

impl GraphContainer {
    // build dependency graph via entry modules.
    pub async fn generate_module_graph(&mut self) {
        let nums_of_thread = num_cpus::get();
        let idle_thread_count: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(nums_of_thread));
        let job_queue: Arc<SegQueue<ResolvedId>> = Default::default();

        self.resolved_entries = join_all(
            self.input
                .entries
                .iter()
                .map(|entry| self.plugin_driver.resolve_id(None, entry)),
        )
        .await
        .into_iter()
        .collect();

        println!("resolved_entries {:?}", self.resolved_entries);

        let mut path_to_node_idx: HashMap<SmolStr, NodeIndex> = Default::default();
        let mut dep_graph = DepGraph::new();

        self.resolved_entries.iter().for_each(|resolved_entry_id| {
            let entry_idx = dep_graph.add_node(resolved_entry_id.id.clone());
            // self.entry_indexs.push(entry_idx);
            path_to_node_idx.insert(resolved_entry_id.id.clone(), entry_idx);
            job_queue.push(resolved_entry_id.clone());
        });

        let processed_id: Arc<DashSet<SmolStr>> = Default::default();
        let (tx, rx) = channel::unbounded::<Msg>();
        println!("job_queue {:?}", job_queue.len());
        for idx in 0..nums_of_thread {
            println!("spawing {:?}", idx);
            let idle_thread_count = idle_thread_count.clone();
            let plugin_driver = self.plugin_driver.clone();
            let worker = Worker {
                tx: tx.clone(),
                job_queue: job_queue.clone(),
                processed_id: processed_id.clone(),
                plugin_driver,
            };
            tokio::task::spawn(async move {
                'root: loop {
                    println!("worker: {:?}", idx);
                    idle_thread_count.fetch_sub(1, Ordering::SeqCst);
                    worker.run().await;
                    idle_thread_count.fetch_add(1, Ordering::SeqCst);
                    loop {
                        if !worker.job_queue.is_empty() {
                            break;
                            // need to work again
                        } else if idle_thread_count.load(Ordering::SeqCst) == nums_of_thread {
                            // All threads are idle now. There's no more work to do.
                            break 'root;
                        }
                    }
                }
            });
        }

        while idle_thread_count.load(Ordering::SeqCst) != nums_of_thread
            || job_queue.len() > 0
            || !rx.is_empty()
        {
            if let Ok(job) = rx.try_recv() {
                match job {
                    Msg::NewMod(module) => {
                        self.module_by_id.insert(module.id.clone(), module);
                    }
                    Msg::DependencyReference(from, to, rel) => {
                        let from_id = *path_to_node_idx
                            .entry(from)
                            .or_insert_with_key(|key| dep_graph.add_node(key.clone()));
                        let to_id = *path_to_node_idx
                            .entry(to)
                            .or_insert_with_key(|key| dep_graph.add_node(key.clone()));
                        dep_graph.add_edge(from_id, to_id, rel);
                    }
                    _ => {}
                }
            }
        }

        println!("grpah {:?}", Dot::new(&dep_graph))

        // let entries_id = self
        //     .entry_indexs
        //     .iter()
        //     .map(|idx| &self.module_graph[*idx])
        //     .collect::<HashSet<&SmolStr>>();
        // self.module_by_id.par_iter_mut().for_each(|(_key, module)| {
        //     module.is_user_defined_entry_point = entries_id.contains(&module.id);
        // });
    }
}

mod worker {
    use std::sync::Arc;

    use crossbeam::{channel::Sender, queue::SegQueue};
    use dashmap::DashSet;
    use smol_str::SmolStr;
    use swc::ecmascript::ast::ModuleItem;
    use swc_ecma_ast::ModuleDecl;
    use swc_ecma_visit::{VisitMutWith, VisitWith};

    use crate::{
        js_ext_module::JsExtModule, js_module::JsModule, parse_file, plugin::ResolvedId,
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
                println!("run");
                if resolved_id.external {
                    let mut js_ext_module = JsExtModule::new(resolved_id.id.clone());
                } else {
                    let mut js_module = JsModule::new(resolved_id.id.clone());
                    let source = self.plugin_driver.load(&js_module.id).await;
                    js_module.source = source.clone();
                    let mut ast = parse_file(source, &resolved_id.id);
                    self.pre_analyze_imported_module(&js_module, &ast).await;
                    let mut dependenecy_scanner = DependencyScanner {
                        tx: &self.tx,
                        js_module: &&js_module,
                        imported: Default::default(),
                        // re_exported: Default::default(),
                        dyn_imported: Default::default(),
                    };
                    ast.visit_children_with(&mut dependenecy_scanner);

                    for imported in dependenecy_scanner.imported.iter() {
                        let resolved_id = js_module.resolve_id(&self.plugin_driver, imported).await;
                        self.tx
                            .send(Msg::DependencyReference(
                                js_module.id.clone(),
                                resolved_id.id,
                                Relation::StaticImport,
                            ))
                            .unwrap();
                    }
                    for dyn_imported in dependenecy_scanner.dyn_imported.iter() {
                        let resolved_id = js_module
                            .resolve_id(&self.plugin_driver, dyn_imported)
                            .await;
                        self.tx
                            .send(Msg::DependencyReference(
                                js_module.id.clone(),
                                resolved_id.id,
                                Relation::AsyncImport,
                            ))
                            .unwrap();
                    }
                    println!("js_module {:?}", js_module);
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
}

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let mut g = GraphContainer {
        plugin_driver: Arc::new(PluginDriver {
            plugins: vec![Box::new(ResolveExtensionPlugin {
                extensions: vec!["js".to_string()],
            })],
        }),
        resolved_entries: Default::default(),
        module_by_id: Default::default(),
        input: InputOptions {
            entries: vec!["./node_modules/lodash-es/lodash.default.js".to_string()],
        },
    };
    g.generate_module_graph().await;
}

fn parse_file(source_code: String, filename: &str) -> swc_ecma_ast::Module {
    let p = Path::new(filename);
    let ext = p.extension().and_then(|ext| ext.to_str()).unwrap_or("js");
    let syntax = if ext == "ts" || ext == "tsx" {
        Syntax::Typescript(TsConfig {
            decorators: false,
            tsx: ext == "tsx",
            ..Default::default()
        })
    } else {
        Syntax::Es(EsConfig {
            static_blocks: true,
            private_in_object: true,
            import_assertions: true,
            jsx: ext == "jsx",
            export_default_from: true,
            decorators_before_export: true,
            decorators: true,
            fn_bind: true,
            allow_super_outside_method: true,
        })
    };
    let compiler = get_compiler();
    let fm = compiler
        .cm
        .new_source_file(FileName::Custom(filename.to_string()), source_code);
    swc::try_with_handler(compiler.cm.clone(), Default::default(), |handler| {
        compiler.parse_js(
            fm,
            handler,
            swc_ecma_ast::EsVersion::Es2022,
            syntax,
            IsModule::Bool(true),
            None,
        )
    })
    .unwrap()
    .expect_module()
}


