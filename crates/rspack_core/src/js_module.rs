use std::collections::{HashMap, HashSet};

use linked_hash_map::LinkedHashMap;
use rspack_swc::{
  swc::{self, config::ModuleConfig},
  swc_atoms, swc_common,
  swc_ecma_ast::{self as ast, EsVersion},
  swc_ecma_transforms_base, swc_ecma_transforms_react,
};
use swc::{Compiler, TransformOutput};
use swc_atoms::JsWord;
use swc_common::{errors::Handler, util::take::Take, FileName, Mark};
use swc_ecma_transforms_base::pass::noop;
use tracing::instrument;

use crate::{
  hmr::hmr_module, syntax_by_loader, BundleContext, BundleMode, Loader, NormalizedBundleOptions,
  ResolvedURI,
};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct DynImportDesc {
  pub argument: JsWord,
  // pub id: Option<JsWord>,
}

pub struct JsModule {
  pub exec_order: i32,
  /**
   * module id for module render
   */
  pub id: String,
  /**
   * logical or physical resource identifier for the js file.
   */
  pub uri: String,
  pub ast: ast::Module,
  pub dependencies: LinkedHashMap<JsWord, ()>,
  pub dyn_imports: HashSet<DynImportDesc>,
  pub is_user_defined_entry_point: bool,
  pub resolved_uris: HashMap<JsWord, ResolvedURI>,
  pub chunk_ids: HashSet<String>,
  pub code_splitting: bool,
  pub loader: Loader,
}
impl std::fmt::Debug for JsModule {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("JsModule")
      .field("exec_order", &self.exec_order)
      .field("uri", &self.uri)
      .field("id", &self.id)
      // .field("ast", &self.ast)
      .field("dependencies", &self.dependencies)
      .field("dyn_dependencies", &self.dyn_imports)
      .field(
        "is_user_defined_entry_point",
        &self.is_user_defined_entry_point,
      )
      // .field("dependency_resolver", &self.dependency_resolver)
      .finish()
  }
}

impl JsModule {
  pub fn new() -> Self {
    Self {
      exec_order: Default::default(),
      uri: Default::default(),
      ast: Take::dummy(),
      id: Default::default(),
      dependencies: Default::default(),
      dyn_imports: Default::default(),
      is_user_defined_entry_point: Default::default(),
      resolved_uris: Default::default(),
      chunk_ids: Default::default(),
      code_splitting: Default::default(),
      loader: Default::default(),
    }
  }
  pub fn add_chunk(&mut self, chunk_id: String) {
    self.chunk_ids.insert(chunk_id);
  }

  #[instrument(skip_all)]
  pub fn render(
    &self,
    compiler: &Compiler,
    modules: &HashMap<String, JsModule>,
    options: &NormalizedBundleOptions,
    bundle: &BundleContext,
  ) -> TransformOutput {
    use swc::config::{self as swc_config, SourceMapsConfig};

    swc::try_with_handler(compiler.cm.clone(), Default::default(), |handler| {
      let fm = compiler
        .cm
        .new_source_file(FileName::Custom(self.uri.to_string()), self.uri.to_string());
      let source_map = if self.id.contains("node_modules") {
        false
      } else {
        options.source_map
      };
      compiler.process_js_with_custom_pass(
        fm,
        Some(ast::Program::Module(self.ast.clone())),
        handler,
        &swc_config::Options {
          config: swc_config::Config {
            jsc: swc_config::JscConfig {
              target: Some(EsVersion::Es2022),
              syntax: Some(syntax_by_loader(self.uri.as_str(), &self.loader)),
              transform: Some(swc_config::TransformConfig {
                react: swc_ecma_transforms_react::Options {
                  runtime: Some(swc_ecma_transforms_react::Runtime::Automatic),
                  ..Default::default()
                },
                ..Default::default()
              })
              .into(),
              ..Default::default()
            },
            inline_sources_content: true.into(),
            emit_source_map_columns: (!matches!(options.mode, BundleMode::Dev)).into(),
            source_maps: Some(SourceMapsConfig::Bool(source_map)),
            ..Default::default()
          },
          top_level_mark: Some(bundle.top_level_mark),
          ..Default::default()
        },
        |_, _| noop(),
        |_, _| {
          hmr_module(
            self.id.to_string(),
            bundle.top_level_mark,
            &self.resolved_uris,
            self.is_user_defined_entry_point,
            modules,
            self.code_splitting,
          )
        },
      )
    })
    .unwrap()
  }
}
