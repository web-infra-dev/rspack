use std::borrow::Cow;

use concat_string::concat_string;
use rspack_core::{
  ChunkGraph, CssExport, CssExports, GenerateContext, Module, ModuleArgument, RESERVED_IDENTIFIER,
  RuntimeGlobals, UsageState, UsedNameItem,
  rspack_sources::{BoxSource, ConcatSource, RawStringSource, SourceExt},
  to_identifier,
};
use rspack_error::Result;
use rspack_util::{
  atom::Atom,
  fx_hash::{FxIndexMap, FxIndexSet},
  itoa, json_stringify, json_stringify_str,
};
use rustc_hash::FxHashSet as HashSet;

use crate::{
  parser_and_generator::{get_unused_local_ident, get_used_exports},
  utils::{replace_css_module_id_placeholder, unescape},
};

pub fn update_css_exports(exports: &mut CssExports, name: &str, css_export: CssExport) -> bool {
  if let Some(existing) = exports.get_mut(name) {
    existing.insert(css_export)
  } else {
    exports
      .insert(name.into(), FxIndexSet::from_iter([css_export]))
      .is_none()
  }
}

pub(crate) struct CssModuleGenerator<'a, 'g> {
  module: &'a dyn Module,
  generate_context: &'a mut GenerateContext<'g>,
  with_hmr: bool,
  es_module: bool,
  module_argument: Option<String>,
  concat_source: ConcatSource,
}

impl<'a, 'g> CssModuleGenerator<'a, 'g> {
  pub fn new(
    module: &'a dyn Module,
    generate_context: &'a mut GenerateContext<'g>,
    with_hmr: bool,
    es_module: bool,
  ) -> Self {
    Self {
      module,
      generate_context,
      with_hmr,
      es_module,
      module_argument: None,
      concat_source: Default::default(),
    }
  }

  fn module_argument(&mut self) -> &str {
    self.module_argument.get_or_insert_with(|| {
      self
        .generate_context
        .runtime_template
        .render_module_argument(ModuleArgument::Module)
    })
  }

  pub fn generate_javascript_source(mut self) -> Result<BoxSource> {
    self.generate_js_exports()?;
    Ok(self.concat_source.boxed())
  }

  fn generate_js_exports(&mut self) -> Result<()> {
    let module = self.module;
    let build_info = module.build_info();
    let css_build_info = build_info
      .css
      .as_deref()
      .expect("CSS modules should have CssBuildInfo");
    let exports_info_artifact = &self.generate_context.compilation.exports_info_artifact;

    if self.generate_context.concatenation_scope.is_some() {
      if let Some(exports) = get_used_exports(
        css_build_info,
        module.identifier(),
        self.generate_context.runtime,
        exports_info_artifact,
      ) {
        if let Some(unused_exports) = get_unused_local_ident(
          css_build_info,
          module.identifier(),
          self.generate_context.runtime,
          exports_info_artifact,
        ) {
          self.generate_context.data.insert(unused_exports);
        }

        self.css_modules_exports_to_concatenate_module_string(exports)?;
      }
      return Ok(());
    }

    let exports_info = exports_info_artifact.get_exports_info_data(&module.identifier());
    let (ns_obj, left, right): (Cow<'_, str>, &str, &str) = if self.es_module
      && exports_info
        .other_exports_info()
        .get_used(self.generate_context.runtime)
        != UsageState::Unused
    {
      (
        Cow::Owned(
          self
            .generate_context
            .runtime_template
            .render_runtime_globals(&RuntimeGlobals::MAKE_NAMESPACE_OBJECT),
        ),
        "(",
        ")",
      )
    } else {
      (Cow::Borrowed(""), "", "")
    };

    let exports_str = if let Some(exports) = get_used_exports(
      css_build_info,
      module.identifier(),
      self.generate_context.runtime,
      exports_info_artifact,
    ) {
      if let Some(unused_exports) = get_unused_local_ident(
        css_build_info,
        module.identifier(),
        self.generate_context.runtime,
        exports_info_artifact,
      ) {
        self.generate_context.data.insert(unused_exports);
      }

      self.css_modules_exports_to_string(exports, &ns_obj, left, right)
    } else {
      let hmr_code = self.render_accept_hmr();
      let module_argument = self.module_argument();
      concat_string!(
        ns_obj,
        left,
        module_argument,
        ".exports = {}",
        right,
        ";\n",
        hmr_code
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
  ) -> String {
    let (decl_name, exports_string) = self.stringified_exports(exports);
    let hmr_code = self.render_exports_hmr(decl_name);
    let module_argument = self.module_argument();

    concat_string!(
      exports_string,
      "\n",
      hmr_code,
      "\n",
      ns_obj,
      left,
      module_argument,
      ".exports = ",
      decl_name,
      right,
      ";\n"
    )
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
      let used_name: Cow<'_, str> = match used_name {
        Some(UsedNameItem::Str(name)) => Cow::Owned(name.to_string()),
        _ => Cow::Borrowed(key),
      };

      let mut content = String::new();
      for CssExport {
        ident,
        from,
        id: _,
        orig_name: _,
      } in elements
      {
        if !content.is_empty() {
          content.push_str(" + \" \" + ");
        }

        match from {
          None => {
            let ident = replace_css_module_id_placeholder(ident, compilation, module);
            content.push_str(&json_stringify_str(&ident));
          }
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
            content.push_str(&runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE));
            content.push('(');
            content.push_str(&from);
            content.push_str(")[");
            content.push_str(&from_used_name);
            content.push(']');
          }
        }
      }
      let mut identifier: Cow<'_, str> = Cow::Owned(to_identifier(&used_name).into_owned());
      if RESERVED_IDENTIFIER.contains(identifier.as_ref()) {
        identifier = Cow::Owned(format!("_{identifier}"));
      }
      let base_identifier = identifier.clone();
      let mut i = 0;
      while used_identifiers.contains(&identifier) {
        let mut i_buffer = itoa::Buffer::new();
        let i_str = i_buffer.format(i);
        identifier = Cow::Owned(format!("{base_identifier}{i_str}"));
        i += 1;
      }
      // TODO: conditional support `const or var` after we finished runtimeTemplate utils
      let export_source = concat_string!("var ", identifier, " = ", content, ";\n");
      self.concat_source.add(RawStringSource::from(export_source));
      used_identifiers.insert(identifier.clone());
      scope.register_export(key.into(), identifier.into_owned());
    }
    Ok(())
  }

  fn stringified_exports<'b>(
    &mut self,
    exports: FxIndexMap<&'b str, &'b FxIndexSet<CssExport>>,
  ) -> (&'static str, String) {
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
      let used_name: Cow<'_, str> = match used_name {
        Some(UsedNameItem::Str(name)) => Cow::Owned(name.to_string()),
        _ => Cow::Borrowed(key),
      };

      stringified_exports.push_str("  ");
      stringified_exports.push_str(&json_stringify_str(&used_name));
      stringified_exports.push_str(": ");

      let mut is_first = true;
      for CssExport {
        ident,
        from,
        id: _,
        orig_name: _,
      } in elements
      {
        if is_first {
          is_first = false;
        } else {
          stringified_exports.push_str(" + \" \" + ");
        }

        match from {
          None => {
            let ident = replace_css_module_id_placeholder(
              ident,
              self.generate_context.compilation,
              self.module,
            );
            stringified_exports.push_str(&json_stringify_str(&ident));
          }
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
            stringified_exports.push_str(
              &self
                .generate_context
                .runtime_template
                .render_runtime_globals(&RuntimeGlobals::REQUIRE),
            );
            stringified_exports.push('(');
            stringified_exports.push_str(&from);
            stringified_exports.push_str(")[");
            stringified_exports.push_str(&from_used_name);
            stringified_exports.push(']');
          }
        }
      }

      stringified_exports.push_str(",\n");
    }

    let decl_name = "exports";
    let exports_source = concat_string!("var ", decl_name, " = {\n", stringified_exports, "};");
    (decl_name, exports_source)
  }

  fn render_exports_hmr<'b>(&mut self, decl_name: &str) -> Cow<'b, str> {
    let with_hmr = self.with_hmr;
    let accept = self.render_accept_hmr();
    let module_argument = self.module_argument();

    if with_hmr {
      Cow::Owned(format!(
        "// only invalidate when locals change
var stringified_exports = JSON.stringify({decl_name});
if ({module_argument}.hot.data && {module_argument}.hot.data.exports && {module_argument}.hot.data.exports != stringified_exports) {{
  {module_argument}.hot.invalidate();
}} else {{
  {accept}}}
{module_argument}.hot.dispose(function(data) {{ data.exports = stringified_exports; }});"
      ))
    } else {
      Cow::Borrowed("")
    }
  }

  fn render_accept_hmr(&mut self) -> String {
    let with_hmr = self.with_hmr;
    let module_argument = self.module_argument();
    if with_hmr {
      format!("{module_argument}.hot.accept();\n")
    } else {
      Default::default()
    }
  }
}
