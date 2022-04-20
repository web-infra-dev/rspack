use dashmap::DashSet;
use smol_str::SmolStr;
use std::{
    collections::{HashMap, HashSet},
    path::Path,
    sync::{Arc, Mutex},
};
use sugar_path::PathSugar;
use swc::config::{Config, ModuleConfig, Options, SourceMapsConfig};
use swc_ecma_parser::EsConfig;
use swc_ecma_visit::VisitMutWith;

use crate::{
    bundler::BundleOptions,
    mark_box::MarkBox,
    module::Module,
    structs::{OutputChunk, RenderedChunk},
    utils::get_compiler,
    visitors::Renamer,
};

use rayon::prelude::*;

use swc_common::comments::{Comment, Comments, SingleThreadedComments};
use swc_ecma_ast::EsVersion;
use swc_ecma_codegen::text_writer::JsWriter;

pub struct Chunk {
    pub id: SmolStr,
    pub order_modules: Vec<SmolStr>,
    pub symbol_box: Arc<Mutex<MarkBox>>,
    pub entries: DashSet<SmolStr>,
}

impl Chunk {
    pub fn new(
        order_modules: Vec<SmolStr>,
        symbol_box: Arc<Mutex<MarkBox>>,
        entries: DashSet<SmolStr>,
    ) -> Self {
        Self {
            id: Default::default(),
            order_modules,
            symbol_box,
            entries,
        }
    }

    pub fn de_conflict(&mut self, modules: &mut HashMap<SmolStr, Box<Module>>) {
        let mut used_names = HashSet::new();
        let mut mark_to_name = HashMap::new();

        // De-conflict from the entry module to keep namings as simple as possible
        self.order_modules
            .iter()
            .map(|id| modules.get(id).unwrap())
            .rev()
            .for_each(|module| {
                module.declared_symbols.iter().for_each(|(name, mark)| {
                    let root_mark = self.symbol_box.lock().unwrap().find_root(*mark);
                    if let std::collections::hash_map::Entry::Vacant(e) =
                        mark_to_name.entry(root_mark)
                    {
                        let original_name = name.to_string();
                        let mut name = name.to_string();
                        let mut count = 0;
                        while used_names.contains(&name) {
                            name = format!("{}${}", original_name, count);
                            count += 1;
                        }
                        e.insert(name.clone());
                        used_names.insert(name);
                    } else {
                    }
                });
            });

        modules.par_iter_mut().for_each(|(_, module)| {
            module.statements.iter_mut().for_each(|stmt| {
                let mut renamer = Renamer {
                    mark_to_names: &mark_to_name,
                    symbol_box: self.symbol_box.clone(),
                };
                stmt.node.visit_mut_with(&mut renamer);
            });
        });

        log::debug!("mark_to_name {:#?}", mark_to_name);
    }

    pub fn render(
        &mut self,
        options: &BundleOptions,
        modules: &mut HashMap<SmolStr, Box<Module>>,
    ) -> RenderedChunk {
        let compiler = get_compiler();

        let mut output_code = String::new();
        self.order_modules.iter().for_each(|idx| {
            let module = modules.get(idx).unwrap();
            let mut transform_output =
                swc::try_with_handler(compiler.cm.clone(), Default::default(), |handler| {
                    Ok(compiler
                        .process_js(
                            handler,
                            module.ast.clone(),
                            &Options {
                                config: Config {
                                    module: Some(ModuleConfig::CommonJs(
                                        swc_ecma_transforms_module::util::Config {
                                            ignore_dynamic: true,
                                            ..Default::default()
                                        },
                                    )),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                        )
                        .unwrap())
                })
                .unwrap();
            output_code += &mut transform_output.code
        });

        RenderedChunk {
            code: output_code,
            file_name: self.id.clone().into(),
        }
    }

    pub fn get_chunk_info_with_file_names(&self) -> OutputChunk {
        OutputChunk {
            code: "".to_string(),
            file_name: self.id.clone().into(),
        }
    }

    #[inline]
    pub fn get_fallback_chunk_name(&self) -> &str {
        println!(
            "self.order_modules.last().unwrap() {:?}",
            self.order_modules.last().unwrap()
        );
        get_alias_name(self.order_modules.last().unwrap())
    }

    #[inline]
    pub fn get_chunk_name(&self) -> &str {
        self.get_fallback_chunk_name()
    }

    pub fn generate_id(&self, options: &BundleOptions) -> SmolStr {
        let pattern = &options.entry_file_names;
        pattern.replace("[name]", self.get_chunk_name()).into()
    }
}

#[inline]
fn get_alias_name(id: &str) -> &str {
    let p = Path::new(id);
    // +1 to include `.`
    let ext_len = p.extension().map_or(0, |s| s.to_string_lossy().len() + 1);
    let file_name = p.file_name().unwrap().to_str().unwrap();
    &file_name[0..file_name.len() - ext_len]
}
