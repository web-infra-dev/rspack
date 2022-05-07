use std::collections::{HashMap, HashSet};

use linked_hash_map::LinkedHashMap;
use swc::{Compiler, TransformOutput};
use swc_atoms::JsWord;
use swc_common::{errors::Handler, util::take::Take, FileName, Mark};
use swc_ecma_transforms_base::pass::noop;

use crate::{hmr::hmr_module, syntax, ResolvedId};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct DynImportDesc {
  pub argument: JsWord,
  // pub id: Option<JsWord>,
}

pub struct JsModule {
  pub exec_order: usize,
  /**
   * module id for module render
   */
  pub id: String,
  /**
   * absolute path for JsModule
   */
  pub path: String,
  pub ast: ast::Module,
  pub dependencies: LinkedHashMap<JsWord, ()>,
  pub dyn_imports: HashSet<DynImportDesc>,
  pub is_user_defined_entry_point: bool,
  pub resolved_ids: HashMap<JsWord, ResolvedId>,
}

impl std::fmt::Debug for JsModule {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("JsModule")
      .field("exec_order", &self.exec_order)
      .field("path", &self.path)
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
      path: Default::default(),
      ast: Take::dummy(),
      id: Default::default(),
      dependencies: Default::default(),
      dyn_imports: Default::default(),
      is_user_defined_entry_point: Default::default(),
      resolved_ids: Default::default(),
    }
  }

  pub fn render(
    &self,
    compiler: &Compiler,
    handler: &Handler,
    top_level_mark: Mark,
    modules: &HashMap<String, JsModule>,
  ) -> anyhow::Result<TransformOutput> {
    use swc::config as swc_config;
    let fm = compiler.cm.new_source_file(
      FileName::Custom(self.path.to_string()),
      self.path.to_string(),
    );
    compiler.process_js_with_custom_pass(
      fm,
      Some(ast::Program::Module(self.ast.clone())),
      handler,
      &swc_config::Options {
        config: swc_config::Config {
          jsc: swc_config::JscConfig {
            syntax: Some(syntax(self.path.as_str())),
            transform: Some(swc_config::TransformConfig {
              react: swc_ecma_transforms_react::Options {
                runtime: Some(swc_ecma_transforms_react::Runtime::Automatic),
                ..Default::default()
              },
              ..Default::default()
            }),
            ..Default::default()
          },
          ..Default::default()
        },
        global_mark: Some(top_level_mark),
        ..Default::default()
      },
      |_, _| noop(),
      |_, _| {
        hmr_module(
          self.id.to_string(),
          top_level_mark,
          &self.resolved_ids,
          self.is_user_defined_entry_point,
          modules,
        )
      },
    )
  }
}
