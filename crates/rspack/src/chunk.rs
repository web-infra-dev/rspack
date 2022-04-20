use dashmap::DashSet;
use smol_str::SmolStr;
use std::{
    collections::{HashMap, HashSet},
    path::Path,
    sync::{Arc, Mutex},
};
use swc::config::{Config, ModuleConfig, Options};
use swc_ecma_visit::VisitMutWith;

use crate::{
    bundler::BundleOptions,
    mark_box::MarkBox,
    module::Module,
    structs::{OutputChunk, RenderedChunk},
    utils::get_compiler,
};

pub struct Chunk {
    pub id: SmolStr,
    pub order_modules: Vec<SmolStr>,
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
            entries,
        }
    }
    pub fn render(
        &mut self,
        _options: &BundleOptions,
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
