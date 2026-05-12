use rspack_core::{
  CssExport, CssExports, GenerateContext, Module, ModuleArgument, RuntimeGlobals, UsageState,
  rspack_sources::{BoxSource, ConcatSource, RawStringSource, SourceExt},
};
use rspack_error::Result;
use rspack_util::fx_hash::FxIndexSet;

use crate::{
  parser_and_generator::{get_unused_local_ident, get_used_exports},
  utils::{css_modules_exports_to_concatenate_module_string, css_modules_exports_to_string},
};

pub fn update_css_exports(exports: &mut CssExports, name: String, css_export: CssExport) -> bool {
  if let Some(existing) = exports.get_mut(&name) {
    existing.insert(css_export)
  } else {
    exports
      .insert(name, FxIndexSet::from_iter([css_export]))
      .is_none()
  }
}

pub(crate) struct CssModuleGenerator<'a, 'g> {
  module: &'a dyn Module,
  generate_context: &'a mut GenerateContext<'g>,
  with_hmr: bool,
  es_module: bool,
  module_argument: String,
  concat_source: ConcatSource,
}

impl<'a, 'g> CssModuleGenerator<'a, 'g> {
  pub fn new(
    module: &'a dyn Module,
    generate_context: &'a mut GenerateContext<'g>,
    with_hmr: bool,
    es_module: bool,
  ) -> Self {
    let module_argument = if with_hmr {
      generate_context
        .runtime_template
        .render_module_argument(ModuleArgument::Module)
    } else {
      String::new()
    };

    Self {
      module,
      generate_context,
      with_hmr,
      es_module,
      module_argument,
      concat_source: Default::default(),
    }
  }

  pub fn generate_javascript_source(mut self) -> Result<BoxSource> {
    self.generate_js_exports()?;
    Ok(self.concat_source.boxed())
  }

  fn generate_js_exports(&mut self) -> Result<()> {
    let module = self.module;
    let build_info = module.build_info();

    if self.generate_context.concatenation_scope.is_some() {
      if let Some(ref exports) = build_info.css_exports {
        let exports_info_artifact = &self.generate_context.compilation.exports_info_artifact;
        if let Some(local_names) = &build_info.css_local_names {
          let unused_exports = get_unused_local_ident(
            exports,
            local_names,
            module.identifier(),
            self.generate_context.runtime,
            exports_info_artifact,
          );
          self.generate_context.data.insert(unused_exports);
        }
        let exports = get_used_exports(
          exports,
          module.identifier(),
          self.generate_context.runtime,
          exports_info_artifact,
        );

        css_modules_exports_to_concatenate_module_string(
          exports,
          module,
          self.generate_context,
          &mut self.concat_source,
        )?;
      }
      return Ok(());
    }

    let exports_info = self
      .generate_context
      .compilation
      .exports_info_artifact
      .get_exports_info_data(&module.identifier());
    let (ns_obj, left, right) = if self.es_module
      && exports_info
        .other_exports_info()
        .get_used(self.generate_context.runtime)
        != UsageState::Unused
    {
      (
        self
          .generate_context
          .runtime_template
          .render_runtime_globals(&RuntimeGlobals::MAKE_NAMESPACE_OBJECT),
        "(".to_string(),
        ")".to_string(),
      )
    } else {
      (String::new(), String::new(), String::new())
    };

    let exports_str = if let Some(exports) = &build_info.css_exports {
      if let Some(local_names) = &build_info.css_local_names {
        let unused_exports = get_unused_local_ident(
          exports,
          local_names,
          module.identifier(),
          self.generate_context.runtime,
          &self.generate_context.compilation.exports_info_artifact,
        );
        self.generate_context.data.insert(unused_exports);
      }

      let exports = get_used_exports(
        exports,
        module.identifier(),
        self.generate_context.runtime,
        &self.generate_context.compilation.exports_info_artifact,
      );

      css_modules_exports_to_string(
        exports,
        module,
        self.generate_context.compilation,
        self.generate_context.runtime,
        self.generate_context.runtime_template,
        &ns_obj,
        &left,
        &right,
        self.with_hmr,
      )?
    } else {
      let module_argument = self
        .generate_context
        .runtime_template
        .render_module_argument(ModuleArgument::Module);
      format!(
        "{}{}{module_argument}.exports = {{}}{};\n{}",
        &ns_obj,
        &left,
        &right,
        self.render_accept_hmr()
      )
    };

    self.concat_source.add(RawStringSource::from(exports_str));
    Ok(())
  }

  fn render_accept_hmr(&self) -> String {
    if self.with_hmr {
      format!("{}.hot.accept();\n", self.module_argument)
    } else {
      Default::default()
    }
  }
}
