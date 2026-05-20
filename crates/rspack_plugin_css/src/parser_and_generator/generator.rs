use std::borrow::Cow;

use concat_string::concat_string;
use cow_utils::CowUtils;
use rspack_core::{
  ChunkGraph, CssExport, CssExportType, CssExports, DependencyType, GenerateContext, Module,
  ModuleArgument, ModuleInitFragments, RESERVED_IDENTIFIER, RuntimeGlobals, TemplateContext,
  UsageState, UsedNameItem,
  rspack_sources::{
    BoxSource, ConcatSource, MapOptions, ObjectPool, OriginalSource, RawStringSource,
    ReplaceSource, Source, SourceExt,
  },
  to_identifier,
};
use rspack_error::Result;
use rspack_util::{
  atom::Atom,
  base64::encode_to_string,
  fx_hash::{FxIndexMap, FxIndexSet},
  itoa, json_stringify, json_stringify_str,
};
use rustc_hash::FxHashSet as HashSet;

use crate::{
  dependency::{CssImportDependency, CssMedia, CssSupports},
  parser_and_generator::{get_unused_local_ident, get_used_exports},
  utils::{replace_css_module_id_placeholder, unescape},
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
  source: &'a BoxSource,
  module: &'a dyn Module,
  generate_context: &'a mut GenerateContext<'g>,
  with_hmr: bool,
  export_type: Option<CssExportType>,
  exports_only: bool,
  es_module: bool,
  module_argument: Option<String>,
  concat_source: ConcatSource,
}

impl<'a, 'g> CssModuleGenerator<'a, 'g> {
  pub fn new(
    source: &'a BoxSource,
    module: &'a dyn Module,
    generate_context: &'a mut GenerateContext<'g>,
    with_hmr: bool,
    export_type: Option<CssExportType>,
    exports_only: bool,
    es_module: bool,
  ) -> Self {
    Self {
      source,
      module,
      generate_context,
      with_hmr,
      export_type,
      exports_only,
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
    match self.export_type {
      Some(CssExportType::Text) => {
        let css = self.stringify_css_source_for_javascript();
        let source = self.generate_css_text_exports(&css);
        self.concat_source.add(RawStringSource::from(source));
      }
      Some(CssExportType::CssStyleSheet) => {
        let css = self.stringify_css_source_for_javascript();
        let source = self.generate_css_style_sheet_exports(&css);
        self.concat_source.add(RawStringSource::from(source));
      }
      Some(CssExportType::Style) if !self.exports_only => {
        let imports = self.render_css_imports_for_style();
        let css = self.stringify_css_source_for_javascript();
        self.concat_source.add(RawStringSource::from(imports));
        let inject_style = self.render_css_inject_style(&css);
        self.concat_source.add(RawStringSource::from(inject_style));
        self.generate_js_exports()?;
      }
      _ => {
        self.generate_js_exports()?;
      }
    }
    let generated_source = self.concat_source.source().into_string_lossy().into_owned();
    if self.module.get_source_map_kind().enabled() {
      Ok(OriginalSource::new(generated_source, self.module.identifier().as_str()).boxed())
    } else {
      Ok(RawStringSource::from(generated_source).boxed())
    }
  }

  fn stringify_css_source_for_javascript(&mut self) -> String {
    let css_source = self.generate_css_source_for_module(self.source, self.module);
    self.stringify_css_source_with_inline_map(css_source)
  }

  fn stringify_css_source_for_module(&mut self, source: &BoxSource, module: &dyn Module) -> String {
    let css_source = self.generate_css_source_for_module(source, module);
    self.stringify_css_source_with_inline_map(css_source)
  }

  fn generate_css_source_for_module(
    &mut self,
    source: &BoxSource,
    module: &dyn Module,
  ) -> BoxSource {
    let generate_context = &mut *self.generate_context;
    let mut source = ReplaceSource::new(source.clone());
    let compilation = generate_context.compilation;
    let mut init_fragments = ModuleInitFragments::default();
    let mut context = TemplateContext {
      compilation,
      module,
      runtime: generate_context.runtime,
      init_fragments: &mut init_fragments,
      concatenation_scope: generate_context.concatenation_scope.take(),
      data: generate_context.data,
      runtime_template: generate_context.runtime_template,
    };

    let module_graph = compilation.get_module_graph();
    module.get_dependencies().iter().for_each(|id| {
      let dep = module_graph.dependency_by_id(id);

      if let Some(dependency) = dep.as_dependency_code_generation()
        && let Some(template) = dependency
          .dependency_template()
          .and_then(|template_type| compilation.get_dependency_template(template_type))
      {
        template.render(dependency, &mut source, &mut context)
      }
    });

    for conn in module_graph.get_incoming_connections(&module.identifier()) {
      let dep = module_graph.dependency_by_id(&conn.dependency_id);

      if matches!(dep.dependency_type(), DependencyType::CssImport) {
        let Some(css_import_dep) = dep.downcast_ref::<CssImportDependency>() else {
          panic!(
            "dependency with type DependencyType::CssImport should only be CssImportDependency"
          );
        };

        if let Some(media) = css_import_dep.media() {
          context.data.insert(CssMedia(media.to_string()));
        }

        if let Some(supports) = css_import_dep.supports() {
          context.data.insert(CssSupports(supports.to_string()));
        }

        if let Some(layer) = css_import_dep.layer() {
          context.data.insert(layer.clone());
        }
      }
    }

    if let Some(dependencies) = module.get_presentational_dependencies() {
      dependencies.iter().for_each(|dependency| {
        if let Some(template) = dependency
          .dependency_template()
          .and_then(|dependency_type| compilation.get_dependency_template(dependency_type))
        {
          template.render(dependency.as_ref(), &mut source, &mut context)
        }
      });
    };

    generate_context.concatenation_scope = context.concatenation_scope.take();

    source.boxed()
  }

  fn stringify_css_source_with_inline_map(&self, css_source: BoxSource) -> String {
    let mut css_text = css_source
      .source()
      .into_string_lossy()
      .cow_replace(crate::utils::AUTO_PUBLIC_PATH_PLACEHOLDER, "")
      .into_owned();

    if let Some(source_map) = css_source.map(&ObjectPool::default(), &MapOptions::default()) {
      let base64_map = encode_to_string(source_map.to_json().as_bytes());
      if !css_text.ends_with('\n') {
        css_text.push('\n');
      }
      css_text.push_str("/*# sourceMappingURL=data:application/json;charset=utf-8;base64,");
      css_text.push_str(&base64_map);
      css_text.push_str("*/");
    }

    json_stringify_str(&css_text)
  }

  fn render_css_imports_for_style(&mut self) -> String {
    let mut visited_non_style_modules = HashSet::default();
    self.render_css_imports_for_style_module(self.module, &mut visited_non_style_modules)
  }

  fn render_css_imports_for_style_module(
    &mut self,
    module: &dyn Module,
    visited_non_style_modules: &mut HashSet<rspack_collections::Identifier>,
  ) -> String {
    let compilation = self.generate_context.compilation;
    let module_graph = compilation.get_module_graph();
    let require = self
      .generate_context
      .runtime_template
      .render_runtime_globals(&RuntimeGlobals::REQUIRE);
    let mut code = String::new();

    for dependency_id in module.get_dependencies() {
      let dependency = module_graph.dependency_by_id(dependency_id);
      if !matches!(dependency.dependency_type(), DependencyType::CssImport) {
        continue;
      }

      let Some(imported_module) = module_graph.module_graph_module_by_dependency_id(dependency_id)
      else {
        continue;
      };

      let Some(module_id) = ChunkGraph::get_module_id(
        &compilation.module_ids_artifact,
        imported_module.module_identifier,
      ) else {
        continue;
      };

      let Some(imported_module) =
        module_graph.module_by_identifier(&imported_module.module_identifier)
      else {
        continue;
      };

      if Self::is_style_export_css_module(imported_module.as_ref()) {
        code.push_str(&concat_string!(
          require,
          "(",
          json_stringify(module_id),
          ");\n"
        ));
        continue;
      }

      if !visited_non_style_modules.insert(imported_module.identifier()) {
        continue;
      }

      code.push_str(
        &self
          .render_css_imports_for_style_module(imported_module.as_ref(), visited_non_style_modules),
      );

      let Some(source) = imported_module.source() else {
        continue;
      };
      let css = self.stringify_css_source_for_module(source, imported_module.as_ref());
      code.push_str(&self.render_css_inject_style_by_module_id(json_stringify(module_id), &css));
    }

    code
  }

  fn is_style_export_css_module(module: &dyn Module) -> bool {
    module
      .as_normal_module()
      .and_then(|module| {
        module
          .parser_and_generator()
          .downcast_ref::<crate::parser_and_generator::CssParserAndGenerator>()
      })
      .is_some_and(|parser_and_generator| {
        matches!(
          parser_and_generator.export_type(),
          Some(CssExportType::Style)
        )
      })
  }

  fn render_css_inject_style(&mut self, css: &str) -> String {
    self
      .generate_context
      .runtime_template
      .runtime_requirements_mut()
      .insert(RuntimeGlobals::CSS_INJECT_STYLE);

    let module_id = ChunkGraph::get_module_id(
      &self.generate_context.compilation.module_ids_artifact,
      self.module.identifier(),
    )
    .map(|id| id.to_string())
    .unwrap_or_default();

    self.render_css_inject_style_by_module_id(json_stringify_str(&module_id), css)
  }

  fn render_css_inject_style_by_module_id(&mut self, module_id: String, css: &str) -> String {
    let css_inject_style = self
      .generate_context
      .runtime_template
      .render_runtime_globals(&RuntimeGlobals::CSS_INJECT_STYLE);
    concat_string!(css_inject_style, "(", module_id, ", ", css, ");\n")
  }

  fn generate_css_style_sheet_exports(&mut self, css: &str) -> String {
    self
      .generate_context
      .runtime_template
      .runtime_requirements_mut()
      .insert(RuntimeGlobals::CSS_STYLE_SHEET);

    let module_argument = self.module_argument().to_string();
    let (ns_obj, left, right) = self.get_namespace_object_parts();
    let css_style_sheet = self
      .generate_context
      .runtime_template
      .render_runtime_globals(&RuntimeGlobals::CSS_STYLE_SHEET);
    let sheet_code = concat_string!(
      "var __css_style_sheet = ",
      css_style_sheet,
      "(",
      css,
      ");\n"
    );

    if let Some((decl_name, exports_string)) = self.stringified_used_css_exports() {
      let hmr_code = self.render_exports_hmr(decl_name);
      concat_string!(
        sheet_code,
        exports_string,
        "\n",
        hmr_code,
        "\n",
        ns_obj,
        left,
        module_argument,
        ".exports = Object.assign({}, ",
        decl_name,
        ")",
        right,
        ";\n",
        module_argument,
        ".exports.default = __css_style_sheet;\n"
      )
    } else if self.es_module {
      concat_string!(
        sheet_code,
        ns_obj,
        "(",
        module_argument,
        ".exports = {});\n",
        module_argument,
        ".exports.default = __css_style_sheet;\n",
        self.render_accept_hmr()
      )
    } else {
      concat_string!(
        sheet_code,
        module_argument,
        ".exports = __css_style_sheet;\n",
        self.render_accept_hmr()
      )
    }
  }

  fn generate_css_text_exports(&mut self, css: &str) -> String {
    let module_argument = self.module_argument().to_string();
    let (ns_obj, left, right) = self.get_namespace_object_parts();

    if let Some((decl_name, exports_string)) = self.stringified_used_css_exports() {
      let hmr_code = self.render_exports_hmr(decl_name);
      concat_string!(
        exports_string,
        "\n",
        hmr_code,
        "\n",
        ns_obj,
        left,
        module_argument,
        ".exports = Object.assign({}, ",
        decl_name,
        ")",
        right,
        ";\n",
        module_argument,
        ".exports.default = ",
        css,
        ";\n"
      )
    } else if self.es_module {
      concat_string!(
        ns_obj,
        "(",
        module_argument,
        ".exports = {});\n",
        module_argument,
        ".exports.default = ",
        css,
        ";\n",
        self.render_accept_hmr()
      )
    } else {
      concat_string!(
        module_argument,
        ".exports = ",
        css,
        ";\n",
        self.render_accept_hmr()
      )
    }
  }

  fn stringified_used_css_exports(&mut self) -> Option<(&'static str, String)> {
    let exports = self.module.build_info().css_exports.as_ref()?;

    if let Some(local_names) = &self.module.build_info().css_local_names {
      let unused_exports = get_unused_local_ident(
        exports,
        local_names,
        self.module.identifier(),
        self.generate_context.runtime,
        &self.generate_context.compilation.exports_info_artifact,
      );
      self.generate_context.data.insert(unused_exports);
    }

    let exports = get_used_exports(
      exports,
      self.module.identifier(),
      self.generate_context.runtime,
      &self.generate_context.compilation.exports_info_artifact,
    );

    Some(self.stringified_exports(exports))
  }

  fn get_namespace_object_parts(&mut self) -> (String, String, String) {
    if self.es_module {
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
    }
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
        identifier = Cow::Owned(concat_string!("_", identifier));
      }
      let base_identifier = identifier.clone();
      let mut i = 0;
      while used_identifiers.contains(&identifier) {
        let mut i_buffer = itoa::Buffer::new();
        let i_str = i_buffer.format(i);
        identifier = Cow::Owned(concat_string!(base_identifier, i_str));
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
      concat_string!(module_argument, ".hot.accept();\n")
    } else {
      Default::default()
    }
  }
}
