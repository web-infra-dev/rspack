use std::borrow::Cow;

use rspack_core::{
  ChunkGraph, CssExport, CssExports, GenerateContext, Module, ModuleArgument, RESERVED_IDENTIFIER,
  RuntimeGlobals, UsageState, UsedNameItem,
  rspack_sources::{BoxSource, ConcatSource, RawStringSource, SourceExt},
  to_identifier,
};
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rspack_util::{
  atom::Atom,
  fx_hash::{FxIndexMap, FxIndexSet},
  itoa, json_stringify, json_stringify_str,
};
use rustc_hash::FxHashSet as HashSet;

use crate::{
  parser_and_generator::{get_unused_local_ident, get_used_exports},
  utils::unescape,
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

        self.css_modules_exports_to_concatenate_module_string(exports)?;
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

      self.css_modules_exports_to_string(exports, &ns_obj, &left, &right)?
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

  fn css_modules_exports_to_string<'b>(
    &mut self,
    exports: rspack_util::fx_hash::FxIndexMap<&'b str, &'b FxIndexSet<CssExport>>,
    ns_obj: &str,
    left: &str,
    right: &str,
  ) -> Result<String> {
    let (decl_name, exports_string) = self.stringified_exports(exports)?;
    let module_argument = self
      .generate_context
      .runtime_template
      .render_module_argument(ModuleArgument::Module);
    let hmr_code = self.render_exports_hmr(&module_argument, decl_name);

    let mut code = format!(
      "{exports_string}\n{hmr_code}\n{ns_obj}{left}{module_argument}.exports = {decl_name}"
    );
    code += right;
    code += ";\n";
    Ok(code)
  }

  fn css_modules_exports_to_concatenate_module_string<'b>(
    &mut self,
    exports: FxIndexMap<&'b str, &'b FxIndexSet<CssExport>>,
  ) -> Result<()> {
    let module = self.module;
    let GenerateContext {
      compilation,
      concatenation_scope,
      runtime,
      runtime_template,
      ..
    } = self.generate_context;
    let Some(scope) = concatenation_scope else {
      return Ok(());
    };
    let module_graph = compilation.get_module_graph();
    let mut used_identifiers = HashSet::default();
    let exports_info = compilation
      .exports_info_artifact
      .get_exports_info_data(&module.identifier());

    for (key, elements) in exports {
      let export_info = exports_info.get_read_only_export_info(&Atom::from(key));
      let used_name = export_info.get_used_name(None, *runtime);
      let used_name = match used_name {
        Some(UsedNameItem::Str(name)) => name.to_string(),
        _ => key.to_string(),
      };

      let content = elements
        .iter()
        .map(
          |CssExport {
             ident,
             from,
             id: _,
             orig_name: _,
           }| match from {
            None => json_stringify_str(ident),
            Some(from_name) => {
              let from = module
                .get_dependencies()
                .iter()
                .find_map(|id| {
                  let dependency = module_graph.dependency_by_id(id);
                  let request = if let Some(d) = dependency.as_module_dependency() {
                    Some(d.request())
                  } else {
                    dependency.as_context_dependency().map(|d| d.request())
                  };
                  if let Some(request) = request
                    && request == from_name
                  {
                    return module_graph.module_graph_module_by_dependency_id(id);
                  }
                  None
                })
                .expect("should have css from module");

              let from_exports_info = compilation
                .exports_info_artifact
                .get_exports_info_data(&from.module_identifier);
              let from_used_name = match from_exports_info
                .get_read_only_export_info(&Atom::from(ident.as_str()))
                .get_used_name(None, *runtime)
              {
                Some(UsedNameItem::Str(name)) => json_stringify_str(&name),
                _ => json_stringify_str(ident),
              };

              let from = json_stringify(
                ChunkGraph::get_module_id(&compilation.module_ids_artifact, from.module_identifier)
                  .expect("should have module"),
              );
              format!(
                "{}({from})[{}]",
                runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE),
                from_used_name
              )
            }
          },
        )
        .collect::<Vec<_>>()
        .join(" + \" \" + ");
      let mut identifier: Cow<'_, str> = Cow::Owned(to_identifier(&used_name).into_owned());
      if RESERVED_IDENTIFIER.contains(identifier.as_ref()) {
        identifier = Cow::Owned(format!("_{identifier}"));
      }
      let mut i = 0;
      while used_identifiers.contains(&identifier) {
        let mut i_buffer = itoa::Buffer::new();
        let i_str = i_buffer.format(i);
        identifier = Cow::Owned(format!("{identifier}{i_str}"));
        i += 1;
      }
      // TODO: conditional support `const or var` after we finished runtimeTemplate utils
      self.concat_source.add(RawStringSource::from(format!(
        "var {identifier} = {content};\n"
      )));
      used_identifiers.insert(identifier.clone());
      scope.register_export(key.into(), identifier.into_owned());
    }
    Ok(())
  }

  fn stringified_exports<'b>(
    &mut self,
    exports: FxIndexMap<&'b str, &'b FxIndexSet<CssExport>>,
  ) -> Result<(&'static str, String)> {
    let module = self.module;
    let compilation = self.generate_context.compilation;
    let module_graph = compilation.get_module_graph();
    let exports_info = compilation
      .exports_info_artifact
      .get_exports_info_data(&module.identifier());
    let mut stringified_exports = String::new();

    for (key, elements) in exports {
      let export_info = exports_info.get_read_only_export_info(&Atom::from(key));
      let used_name = export_info.get_used_name(None, self.generate_context.runtime);
      let used_name = match used_name {
        Some(UsedNameItem::Str(name)) => name.to_string(),
        _ => key.to_string(),
      };

      let content = elements
        .iter()
        .map(
          |CssExport {
             ident,
             from,
             id: _,
             orig_name: _,
           }| match from {
            None => json_stringify_str(ident),
            Some(from_name) => {
              let from = module
                .get_dependencies()
                .iter()
                .find_map(|id| {
                  let dependency = module_graph.dependency_by_id(id);
                  let request = if let Some(d) = dependency.as_module_dependency() {
                    Some(d.request())
                  } else {
                    dependency.as_context_dependency().map(|d| d.request())
                  };
                  if let Some(request) = request
                    && request == from_name
                  {
                    return module_graph.module_graph_module_by_dependency_id(id);
                  }
                  None
                })
                .expect("should have css from module");

              let from_exports_info = compilation
                .exports_info_artifact
                .get_exports_info_data(&from.module_identifier);
              let from_used_name = match from_exports_info
                .get_read_only_export_info(&Atom::from(ident.as_str()))
                .get_used_name(None, self.generate_context.runtime)
              {
                Some(UsedNameItem::Str(name)) => json_stringify_str(&unescape(name.as_str())),
                _ => json_stringify_str(&unescape(ident)),
              };

              let from = json_stringify(
                ChunkGraph::get_module_id(&compilation.module_ids_artifact, from.module_identifier)
                  .expect("should have module"),
              );
              format!(
                "{}({from})[{}]",
                self
                  .generate_context
                  .runtime_template
                  .render_runtime_globals(&RuntimeGlobals::REQUIRE),
                from_used_name
              )
            }
          },
        )
        .collect::<Vec<_>>()
        .join(" + \" \" + ");
      use std::fmt::Write;
      writeln!(
        stringified_exports,
        "  {}: {},",
        json_stringify_str(&used_name),
        content
      )
      .to_rspack_result()?;
    }

    let decl_name = "exports";
    Ok((
      decl_name,
      format!("var {decl_name} = {{\n{stringified_exports}}};"),
    ))
  }

  fn render_exports_hmr<'b>(&self, module_argument: &str, decl_name: &str) -> Cow<'b, str> {
    if self.with_hmr {
      Cow::Owned(format!(
        "// only invalidate when locals change
var stringified_exports = JSON.stringify({decl_name});
if ({module_argument}.hot.data && {module_argument}.hot.data.exports && {module_argument}.hot.data.exports != stringified_exports) {{
  {module_argument}.hot.invalidate();
}} else {{
  {module_argument}.hot.accept();
}}
{module_argument}.hot.dispose(function(data) {{ data.exports = stringified_exports; }});"
      ))
    } else {
      Cow::Borrowed("")
    }
  }

  fn render_accept_hmr(&self) -> String {
    if self.with_hmr {
      format!("{}.hot.accept();\n", self.module_argument)
    } else {
      Default::default()
    }
  }
}
