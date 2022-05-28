use std::{
  collections::{HashMap, HashSet},
  sync::{Arc, Mutex},
};

use linked_hash_map::LinkedHashMap;
use rspack_swc::{
  swc::{self},
  swc_atoms, swc_common,
  swc_ecma_ast::{self as ast, EsVersion},
  swc_ecma_transforms_base, swc_ecma_transforms_react,
};
use swc::TransformOutput;
use swc_atoms::JsWord;
use swc_common::{util::take::Take, FileName};
use swc_ecma_transforms_base::pass::noop;
use tracing::instrument;

use crate::{
  finalize::hmr_module, syntax_by_loader, Bundle, BundleMode, Loader, ModuleGraph, ResolvedURI,
};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct DynImportDesc {
  pub argument: JsWord,
  // pub id: Option<JsWord>,
}

pub struct JsModule {
  pub kind: JsModuleKind,
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
  pub resolved_uris: HashMap<JsWord, ResolvedURI>,
  pub loader: Loader,
  pub cached_output: Mutex<Option<Arc<TransformOutput>>>,
}
impl std::fmt::Debug for JsModule {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("JsModule")
      .field("kind", &self.kind)
      .field("exec_order", &self.exec_order)
      .field("uri", &self.uri)
      .field("id", &self.id)
      // .field("ast", &self.ast)
      .field("dependencies", &self.dependencies)
      .field("dyn_dependencies", &self.dyn_imports)
      // .field("dependency_resolver", &self.dependency_resolver)
      .finish()
  }
}

impl Default for JsModule {
  fn default() -> Self {
    Self::new()
  }
}

impl JsModule {
  pub fn new() -> Self {
    Self {
      kind: JsModuleKind::Normal,
      exec_order: Default::default(),
      uri: Default::default(),
      ast: Take::dummy(),
      id: Default::default(),
      dependencies: Default::default(),
      dyn_imports: Default::default(),
      resolved_uris: Default::default(),
      // TODO: We should not initialize loader using default value, it’s easy to forget and buggy.
      loader: Default::default(),
      cached_output: Default::default(),
    }
  }

  #[instrument(skip_all)]
  pub fn render(&self, bundle: &Bundle) -> Arc<TransformOutput> {
    use swc::config::{self as swc_config, SourceMapsConfig};
    let bundle_ctx = &bundle.context;
    let options = &bundle_ctx.options;
    let compiler = &bundle_ctx.compiler;
    let mut cached_output = self.cached_output.lock().unwrap();
    if let Some(cache) = cached_output.clone() {
      cache
    } else {
      let output = swc::try_with_handler(compiler.cm.clone(), Default::default(), |handler| {
        let fm = compiler
          .cm
          .new_source_file(FileName::Custom(self.uri.to_string()), self.uri.to_string());
        let source_map = if self.id.contains("node_modules") {
          false
        } else {
          options.source_map.is_enabled()
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
            top_level_mark: Some(bundle_ctx.top_level_mark),
            ..Default::default()
          },
          |_, _| noop(),
          |_, _| {
            hmr_module(
              self.id.to_string(),
              bundle_ctx.top_level_mark,
              &self.resolved_uris,
              self.kind.is_user_entry(),
              &bundle.module_graph_container.module_graph,
              options.code_splitting.is_some(),
              bundle,
            )
          },
        )
      })
      .unwrap();
      let output = Arc::new(output);
      *cached_output = Some(output.clone());
      output
    }
  }

  pub fn dependency_modules<'a>(&self, module_graph: &'a ModuleGraph) -> Vec<&'a JsModule> {
    self
      .dependencies
      .keys()
      .map(|dep| &self.resolved_uris[dep].uri)
      .filter_map(|uri| module_graph.module_by_uri(uri))
      .collect()
  }

  pub fn dynamic_dependency_modules<'a>(&self, module_graph: &'a ModuleGraph) -> Vec<&'a JsModule> {
    self
      .dyn_imports
      .iter()
      .map(|dyn_imp| &dyn_imp.argument)
      .map(|dep| &self.resolved_uris[dep].uri)
      .filter_map(|uri| module_graph.module_by_uri(uri))
      .collect()
  }
}

#[derive(Debug)]
pub enum JsModuleKind {
  UserEntry { name: String },
  Normal,
}

impl JsModuleKind {
  pub fn is_user_entry(&self) -> bool {
    matches!(self, JsModuleKind::UserEntry { .. })
  }

  pub fn is_normal(&self) -> bool {
    matches!(self, JsModuleKind::Normal)
  }

  pub fn name(&self) -> Option<&str> {
    match self {
      Self::UserEntry { name } => Some(name.as_str()),
      _ => None,
    }
  }
}
