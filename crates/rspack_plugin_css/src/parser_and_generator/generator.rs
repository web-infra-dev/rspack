use std::{borrow::Cow, sync::LazyLock};

use concat_string::concat_string;
use cow_utils::CowUtils;
use regex::Regex;
use rspack_core::{
  ChunkGraph, CssExport, CssExportType, CssExports, DependencyType, GenerateContext, Module,
  ModuleArgument, RESERVED_IDENTIFIER, RuntimeGlobals, SourceType, UsageState, UsedNameItem,
  rspack_sources::{
    BoxSource, ConcatSource, MapOptions, ObjectPool, OriginalSource, RawStringSource, Source,
    SourceExt,
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
  dependency::{CssImportDependency, CssLayer},
  parser_and_generator::{get_unused_local_ident, get_used_exports, render_css_module_source},
  utils::{replace_css_module_id_placeholder, replace_css_module_id_placeholder_with_id, unescape},
};

static REGEX_CHARSET: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(r#"(?i)@charset\s+("[^"]*"|'[^']*')\s*;\s*"#).expect("Invalid regex")
});

#[derive(Clone, Default)]
struct CssImportConditions {
  media: Option<String>,
  supports: Option<String>,
  layer: Option<CssLayer>,
}

impl CssImportConditions {
  fn from_dependency(dep: &CssImportDependency) -> Self {
    Self {
      media: dep.media().map(|media| media.trim().to_string()),
      supports: dep.supports().map(|supports| supports.trim().to_string()),
      layer: dep.layer().cloned(),
    }
  }

  fn is_empty(&self) -> bool {
    self.media.is_none() && self.supports.is_none() && self.layer.is_none()
  }

  fn cache_key(&self) -> String {
    let layer = match &self.layer {
      Some(CssLayer::Named(layer)) => concat_string!("layer:", layer),
      Some(CssLayer::Anonymous) => "layer:".to_string(),
      None => String::new(),
    };
    concat_string!(
      self.media.as_deref().unwrap_or_default(),
      "|",
      self.supports.as_deref().unwrap_or_default(),
      "|",
      layer
    )
  }
}

#[derive(Clone, Copy)]
enum CssExportRenderMode {
  Standard,
  Concatenation { unescape_referenced_ident: bool },
}

enum CssTextFragment {
  Static(String),
  Expression(String),
}

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
        let css = self.stringify_css_source_for_javascript_with_imports();
        let source = self.generate_css_text_exports(&css)?;
        self.concat_source.add(RawStringSource::from(source));
      }
      Some(CssExportType::CssStyleSheet) => {
        let css = self.stringify_css_source_for_javascript_with_imports();
        let source = self.generate_css_style_sheet_exports(&css)?;
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

  fn stringify_css_source_for_javascript_with_imports(&mut self) -> String {
    if !self.has_css_imports(self.module) {
      return self.stringify_css_source_for_javascript();
    }

    let mut seen = HashSet::default();
    let fragments =
      self.css_text_fragments_for_module_with_imports(self.source, self.module, &[], &mut seen);
    self.stringify_css_text_fragments(fragments)
  }

  fn has_css_imports(&self, module: &dyn Module) -> bool {
    let module_graph = self.generate_context.compilation.get_module_graph();
    module.get_dependencies().iter().any(|dependency_id| {
      let dependency = module_graph.dependency_by_id(dependency_id);
      matches!(dependency.dependency_type(), DependencyType::CssImport)
    })
  }

  fn can_inline_css_text_for_module(&self, module: &dyn Module) -> bool {
    let module_graph = self.generate_context.compilation.get_module_graph();
    module.get_dependencies().iter().all(|dependency_id| {
      let dependency = module_graph.dependency_by_id(dependency_id);
      !matches!(
        dependency.dependency_type(),
        DependencyType::CssCompose | DependencyType::CssUrl
      )
    })
  }

  fn stringify_css_text_fragments(&self, fragments: Vec<CssTextFragment>) -> String {
    let mut css_text = String::new();
    let mut expressions = Vec::new();
    let mut static_only = true;

    for fragment in fragments {
      match fragment {
        CssTextFragment::Static(fragment) => {
          if !fragment.is_empty() {
            css_text.push_str(&fragment);
            expressions.push(json_stringify_str(&fragment));
          }
        }
        CssTextFragment::Expression(expression) => {
          static_only = false;
          expressions.push(expression);
        }
      }
    }

    if static_only {
      return json_stringify_str(&normalize_css_charset(css_text));
    }

    if expressions.is_empty() {
      json_stringify_str("")
    } else {
      self.normalize_css_charset_expression(&expressions.join(" + "))
    }
  }

  fn normalize_css_charset_expression(&self, expression: &str) -> String {
    concat_string!(
      "(function(css) { var charset = css.match(/@charset\\s+(\"[^\"]*\"|'[^']*')\\s*;\\s*/i); return charset ? \"@charset \" + charset[1] + \";\\n\" + css.replace(/@charset\\s+(\"[^\"]*\"|'[^']*')\\s*;\\s*/gi, \"\").trimStart() : css; })(",
      expression,
      ")"
    )
  }

  fn css_text_fragments_for_module_with_imports(
    &mut self,
    source: &BoxSource,
    module: &dyn Module,
    import_conditions: &[CssImportConditions],
    seen: &mut HashSet<rspack_collections::Identifier>,
  ) -> Vec<CssTextFragment> {
    if !seen.insert(module.identifier()) {
      return Vec::new();
    }

    let mut fragments =
      self.render_css_import_fragments_for_module(module, import_conditions, seen);
    let css_source = self.generate_css_source_for_module(source, module);
    let mut own_css = self.css_text_from_source(css_source);
    self.wrap_css_source_with_import_conditions(&mut own_css, import_conditions);
    fragments.push(CssTextFragment::Static(own_css));
    seen.remove(&module.identifier());
    fragments
  }

  fn render_css_import_fragments_for_module(
    &mut self,
    module: &dyn Module,
    import_conditions: &[CssImportConditions],
    seen: &mut HashSet<rspack_collections::Identifier>,
  ) -> Vec<CssTextFragment> {
    let compilation = self.generate_context.compilation;
    let module_graph = compilation.get_module_graph();
    let mut imported_modules = Vec::new();

    for dependency_id in module.get_dependencies() {
      let dependency = module_graph.dependency_by_id(dependency_id);
      if !matches!(dependency.dependency_type(), DependencyType::CssImport) {
        continue;
      }
      let Some(css_import_dep) = dependency.downcast_ref::<CssImportDependency>() else {
        panic!("dependency with type DependencyType::CssImport should only be CssImportDependency");
      };
      let Some(imported_module) = module_graph.module_graph_module_by_dependency_id(dependency_id)
      else {
        continue;
      };
      imported_modules.push((
        imported_module.module_identifier,
        CssImportConditions::from_dependency(css_import_dep),
      ));
    }

    let mut fragments = Vec::new();
    for (module_identifier, current_import_conditions) in imported_modules {
      let Some(imported_module) = module_graph.module_by_identifier(&module_identifier) else {
        continue;
      };

      let mut next_import_conditions = import_conditions.to_vec();
      next_import_conditions.push(current_import_conditions);

      if self.can_inline_css_text_for_module(imported_module.as_ref()) {
        let Some(source) = imported_module.source() else {
          continue;
        };
        fragments.extend(self.css_text_fragments_for_module_with_imports(
          source,
          imported_module.as_ref(),
          &next_import_conditions,
          seen,
        ));
        continue;
      }

      let Some(module_id) =
        ChunkGraph::get_module_id(&compilation.module_ids_artifact, module_identifier)
      else {
        continue;
      };
      let expression = self.render_css_import_default_expression(json_stringify(module_id));
      fragments.push(CssTextFragment::Expression(
        self.wrap_css_expression_with_import_conditions(expression, &next_import_conditions),
      ));
    }

    fragments
  }

  fn stringify_css_source_for_module_with_import_conditions(
    &mut self,
    source: &BoxSource,
    module: &dyn Module,
    import_conditions: &[CssImportConditions],
  ) -> String {
    let css_source = self.generate_css_source_for_module(source, module);
    self.stringify_css_source_with_inline_map_and_import_conditions(css_source, import_conditions)
  }

  fn generate_css_source_for_module(
    &mut self,
    source: &BoxSource,
    module: &dyn Module,
  ) -> BoxSource {
    render_css_module_source(source, module, self.generate_context)
  }

  fn stringify_css_source_with_inline_map(&self, css_source: BoxSource) -> String {
    self.stringify_css_source_with_inline_map_and_import_conditions(css_source, &[])
  }

  fn stringify_css_source_with_inline_map_and_import_conditions(
    &self,
    css_source: BoxSource,
    import_conditions: &[CssImportConditions],
  ) -> String {
    let mut css_text = self.css_text_from_source(css_source.clone());
    self.wrap_css_source_with_import_conditions(&mut css_text, import_conditions);

    if import_conditions.is_empty()
      && let Some(source_map) = css_source.map(&ObjectPool::default(), &MapOptions::default())
    {
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

  fn css_text_from_source(&self, css_source: BoxSource) -> String {
    css_source
      .source()
      .into_string_lossy()
      .cow_replace(crate::utils::AUTO_PUBLIC_PATH_PLACEHOLDER, "")
      .into_owned()
  }

  fn wrap_css_source_with_import_conditions(
    &self,
    css_text: &mut String,
    import_conditions: &[CssImportConditions],
  ) {
    for conditions in import_conditions.iter().rev() {
      if let Some(layer) = &conditions.layer {
        let mut wrapped = match layer {
          CssLayer::Named(layer) => concat_string!("@layer ", layer, " {\n"),
          CssLayer::Anonymous => "@layer {\n".to_string(),
        };
        wrapped.push_str(css_text);
        wrapped.push_str("\n}");
        *css_text = wrapped;
      }

      if let Some(supports) = &conditions.supports {
        let mut wrapped = concat_string!("@supports (", supports, ") {\n");
        wrapped.push_str(css_text);
        wrapped.push_str("\n}");
        *css_text = wrapped;
      }

      if let Some(media) = &conditions.media {
        let mut wrapped = concat_string!("@media ", media, "{\n");
        wrapped.push_str(css_text);
        wrapped.push_str("\n}");
        *css_text = wrapped;
      }
    }
  }

  fn wrap_css_expression_with_import_conditions(
    &self,
    mut expression: String,
    import_conditions: &[CssImportConditions],
  ) -> String {
    for conditions in import_conditions.iter().rev() {
      if let Some(layer) = &conditions.layer {
        let header = match layer {
          CssLayer::Named(layer) => concat_string!("@layer ", layer, " {\n"),
          CssLayer::Anonymous => "@layer {\n".to_string(),
        };
        expression = concat_string!(
          json_stringify_str(&header),
          " + (",
          expression,
          ") + ",
          json_stringify_str("\n}")
        );
      }

      if let Some(supports) = &conditions.supports {
        expression = concat_string!(
          json_stringify_str(&concat_string!("@supports (", supports, ") {\n")),
          " + (",
          expression,
          ") + ",
          json_stringify_str("\n}")
        );
      }

      if let Some(media) = &conditions.media {
        expression = concat_string!(
          json_stringify_str(&concat_string!("@media ", media, "{\n")),
          " + (",
          expression,
          ") + ",
          json_stringify_str("\n}")
        );
      }
    }

    expression
  }

  fn render_css_import_default_expression(&mut self, module_id: String) -> String {
    self
      .generate_context
      .runtime_template
      .runtime_requirements_mut()
      .insert(RuntimeGlobals::REQUIRE);
    let require = self
      .generate_context
      .runtime_template
      .render_runtime_globals(&RuntimeGlobals::REQUIRE);
    concat_string!(
      "(function(module) { return module && Object.prototype.hasOwnProperty.call(module, \"default\") ? module.default : module; })(",
      require,
      "(",
      module_id,
      "))"
    )
  }

  fn render_css_imports_for_style(&mut self) -> String {
    let mut visited_inlined_modules = HashSet::default();
    let mut import_conditions = Vec::new();
    self.render_css_imports_for_style_module(
      self.module,
      &mut visited_inlined_modules,
      &mut import_conditions,
    )
  }

  fn render_css_imports_for_style_module(
    &mut self,
    module: &dyn Module,
    visited_inlined_modules: &mut HashSet<String>,
    import_conditions: &mut Vec<CssImportConditions>,
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
      let Some(css_import_dep) = dependency.downcast_ref::<CssImportDependency>() else {
        panic!("dependency with type DependencyType::CssImport should only be CssImportDependency");
      };

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

      let current_import_conditions = CssImportConditions::from_dependency(css_import_dep);
      if Self::is_style_export_css_module(imported_module.as_ref())
        && import_conditions.is_empty()
        && current_import_conditions.is_empty()
      {
        code.push_str(&concat_string!(
          require,
          "(",
          json_stringify(module_id),
          ");\n"
        ));
        continue;
      }

      import_conditions.push(current_import_conditions);
      let import_conditions_key = import_conditions
        .iter()
        .map(CssImportConditions::cache_key)
        .collect::<Vec<_>>()
        .join(";");
      let inlined_module_key = concat_string!(
        imported_module.identifier().as_str(),
        "|",
        import_conditions_key
      );
      if !visited_inlined_modules.insert(inlined_module_key.clone()) {
        import_conditions.pop();
        continue;
      }

      code.push_str(&self.render_css_imports_for_style_module(
        imported_module.as_ref(),
        visited_inlined_modules,
        import_conditions,
      ));

      let Some(source) = imported_module.source() else {
        import_conditions.pop();
        continue;
      };
      let css = self.stringify_css_source_for_module_with_import_conditions(
        source,
        imported_module.as_ref(),
        import_conditions,
      );
      import_conditions.pop();
      let style_module_id = if import_conditions_key.is_empty() {
        module_id.to_string()
      } else {
        concat_string!(module_id.to_string(), "|", import_conditions_key)
      };
      code.push_str(
        &self.render_css_inject_style_by_module_id(json_stringify_str(&style_module_id), &css),
      );
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
    .map_or_else(
      || {
        self
          .module
          .readable_identifier(&self.generate_context.compilation.options.context)
          .into_owned()
      },
      |id| id.to_string(),
    );

    self.render_css_inject_style_by_module_id(json_stringify_str(&module_id), css)
  }

  fn render_css_inject_style_by_module_id(&mut self, module_id: String, css: &str) -> String {
    let css_inject_style = self
      .generate_context
      .runtime_template
      .render_runtime_globals(&RuntimeGlobals::CSS_INJECT_STYLE);
    concat_string!(css_inject_style, "(", module_id, ", ", css, ");\n")
  }

  fn generate_css_style_sheet_exports(&mut self, css: &str) -> Result<String> {
    let css_style_sheet_expr = self.render_css_style_sheet_expression(css);
    if self.generate_context.concatenation_scope.is_some() {
      self.css_modules_exports_to_concatenate_module_string_with_default(Some(
        css_style_sheet_expr,
      ))?;
      return Ok(String::new());
    }

    let sheet_code = concat_string!("var __css_style_sheet = ", css_style_sheet_expr, ";\n");

    Ok(self.generate_css_default_exports(&sheet_code, "__css_style_sheet", false))
  }

  fn generate_css_text_exports(&mut self, css: &str) -> Result<String> {
    if self.generate_context.concatenation_scope.is_some() {
      self.css_modules_exports_to_concatenate_module_string_with_default(Some(css.to_string()))?;
      return Ok(String::new());
    }

    Ok(self.generate_css_default_exports("", css, false))
  }

  fn generate_css_default_exports(
    &mut self,
    prelude: &str,
    default_expr: &str,
    with_exports_hmr: bool,
  ) -> String {
    let module_argument = self.module_argument().to_string();
    let (ns_obj, left, right) = self.get_namespace_object_parts();

    if let Some((decl_name, exports_string)) = self.stringified_used_css_exports() {
      let hmr_code = if with_exports_hmr {
        self.render_exports_hmr(decl_name)
      } else {
        Cow::Borrowed("")
      };
      concat_string!(
        prelude,
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
        default_expr,
        ";\n"
      )
    } else if self.es_module {
      concat_string!(
        prelude,
        ns_obj,
        "(",
        module_argument,
        ".exports = {});\n",
        module_argument,
        ".exports.default = ",
        default_expr,
        ";\n"
      )
    } else {
      concat_string!(prelude, module_argument, ".exports = ", default_expr, ";\n")
    }
  }

  fn render_css_style_sheet_expression(&mut self, css: &str) -> String {
    self
      .generate_context
      .runtime_template
      .runtime_requirements_mut()
      .insert(RuntimeGlobals::CSS_STYLE_SHEET);
    let css_style_sheet = self
      .generate_context
      .runtime_template
      .render_runtime_globals(&RuntimeGlobals::CSS_STYLE_SHEET);
    concat_string!(css_style_sheet, "(", css, ")")
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
    self.css_modules_exports_to_concatenate_module_string_inner(None, Some(exports))
  }

  fn css_modules_exports_to_concatenate_module_string_with_default(
    &mut self,
    default_expr: Option<String>,
  ) -> Result<()> {
    let exports = self
      .module
      .build_info()
      .css_exports
      .as_ref()
      .map(|exports| {
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

        get_used_exports(
          exports,
          self.module.identifier(),
          self.generate_context.runtime,
          &self.generate_context.compilation.exports_info_artifact,
        )
      });
    self.css_modules_exports_to_concatenate_module_string_inner(default_expr, exports)
  }

  fn css_modules_exports_to_concatenate_module_string_inner<'b>(
    &mut self,
    default_expr: Option<String>,
    exports: Option<FxIndexMap<&'b str, &'b FxIndexSet<CssExport>>>,
  ) -> Result<()> {
    let module = self.module;
    if self.generate_context.concatenation_scope.is_none() {
      return Ok(());
    }
    let compilation = self.generate_context.compilation;
    let runtime = self.generate_context.runtime;
    let mut used_identifiers = HashSet::default();
    let exports_info = compilation
      .exports_info_artifact
      .get_exports_info_data(&module.identifier());

    if let Some(default_expr) = default_expr {
      self.register_concatenated_css_export(
        "default",
        &default_expr,
        &mut used_identifiers,
        exports_info,
        runtime,
      );
    }

    let Some(exports) = exports else {
      return Ok(());
    };

    for (key, elements) in exports {
      let export_info = exports_info.get_read_only_export_info(&Atom::from(key));
      let used_name = export_info.get_used_name(None, runtime);
      let used_name: Cow<'_, str> = match used_name {
        Some(UsedNameItem::Str(name)) => Cow::Owned(name.to_string()),
        _ => Cow::Borrowed(key),
      };

      let content = self.render_css_export_content(
        elements,
        CssExportRenderMode::Concatenation {
          unescape_referenced_ident: false,
        },
      );
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
      let Some(ref mut scope) = self.generate_context.concatenation_scope else {
        unreachable!();
      };
      scope.register_export(key.into(), identifier.into_owned());
    }
    Ok(())
  }

  fn register_concatenated_css_export(
    &mut self,
    key: &str,
    content: &str,
    used_identifiers: &mut HashSet<Cow<'_, str>>,
    exports_info: &rspack_core::ExportsInfoData,
    runtime: Option<&rspack_core::RuntimeSpec>,
  ) {
    let export_info = exports_info.get_read_only_export_info(&Atom::from(key));
    let Some(UsedNameItem::Str(used_name)) = export_info.get_used_name(None, runtime) else {
      return;
    };

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

    let export_source = concat_string!("var ", identifier, " = ", content, ";\n");
    self.concat_source.add(RawStringSource::from(export_source));
    used_identifiers.insert(identifier.clone());
    let Some(ref mut scope) = self.generate_context.concatenation_scope else {
      unreachable!();
    };
    scope.register_export(key.into(), identifier.into_owned());
  }

  fn resolve_static_css_export(
    &self,
    module: &dyn Module,
    export_name: &str,
    seen: &mut HashSet<(rspack_core::ModuleIdentifier, String)>,
  ) -> Option<String> {
    let compilation = self.generate_context.compilation;
    let module_graph = compilation.get_module_graph();
    let module_identifier = module.identifier();
    if !seen.insert((module_identifier, export_name.to_string())) {
      return None;
    }

    let exports = module.build_info().css_exports.as_ref()?;
    let values = exports
      .get(export_name)?
      .iter()
      .filter_map(|css_export| match css_export.from.as_deref() {
        None => Some(
          self
            .replace_css_module_id_placeholder_for_concatenation(&css_export.ident, module)
            .into_owned(),
        ),
        Some(from_request) => {
          let target_module = css_export
            .id
            .as_ref()
            .and_then(|id| module_graph.get_module_by_dependency_id(id))
            .or_else(|| {
              module.get_dependencies().iter().find_map(|id| {
                let dependency = module_graph.dependency_by_id(id);
                let request = dependency
                  .as_module_dependency()
                  .map(|dep| dep.request())
                  .or_else(|| dependency.as_context_dependency().map(|dep| dep.request()));
                (request == Some(from_request))
                  .then(|| module_graph.get_module_by_dependency_id(id))?
              })
            })?;
          self.resolve_static_css_export(target_module.as_ref(), &css_export.ident, seen)
        }
      })
      .collect::<Vec<_>>();

    if values.is_empty() {
      None
    } else {
      Some(values.join(" "))
    }
  }

  fn replace_css_module_id_placeholder_for_concatenation<'b>(
    &self,
    local_ident: &'b str,
    module: &dyn Module,
  ) -> Cow<'b, str> {
    if !local_ident.contains(crate::utils::CSS_MODULE_ID_PLACEHOLDER) {
      return Cow::Borrowed(local_ident);
    }

    let compilation = self.generate_context.compilation;
    let module_id =
      ChunkGraph::get_module_id(&compilation.module_ids_artifact, module.identifier()).map_or_else(
        || {
          module
            .readable_identifier(&compilation.options.context)
            .into_owned()
        },
        |id| id.to_string(),
      );
    replace_css_module_id_placeholder_with_id(local_ident, &module_id)
  }

  fn render_css_export_content(
    &mut self,
    elements: &FxIndexSet<CssExport>,
    mode: CssExportRenderMode,
  ) -> String {
    let compilation = self.generate_context.compilation;
    let module = self.module;
    let runtime = self.generate_context.runtime;
    let module_graph = compilation.get_module_graph();

    elements
      .iter()
      .map(
        |CssExport {
           ident,
           from,
           id,
           orig_name: _,
         }| match from {
          None => {
            let ident = match mode {
              CssExportRenderMode::Standard => {
                replace_css_module_id_placeholder(ident, compilation, module)
              }
              CssExportRenderMode::Concatenation { .. } => {
                self.replace_css_module_id_placeholder_for_concatenation(ident, module)
              }
            };
            json_stringify_str(&ident)
          }
          Some(from_name) => {
            if matches!(mode, CssExportRenderMode::Standard) {
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
                .get_used_name(None, runtime)
              {
                Some(UsedNameItem::Str(name)) => json_stringify_str(&unescape(name.as_str())),
                _ => json_stringify_str(&unescape(ident)),
              };

              let from = json_stringify(
                ChunkGraph::get_module_id(&compilation.module_ids_artifact, from.module_identifier)
                  .expect("should have module"),
              );
              return concat_string!(
                self
                  .generate_context
                  .runtime_template
                  .render_runtime_globals(&RuntimeGlobals::REQUIRE),
                "(",
                from,
                ")[",
                from_used_name,
                "]"
              );
            }

            let CssExportRenderMode::Concatenation {
              unescape_referenced_ident,
            } = mode
            else {
              unreachable!();
            };
            let current_module_identifier = module.identifier();
            let chunk_graph = &compilation.build_chunk_graph_artifact.chunk_graph;
            let current_module_chunks =
              if chunk_graph.get_number_of_module_chunks(current_module_identifier) > 0 {
                Some(chunk_graph.get_module_chunks(current_module_identifier))
              } else {
                None
              };
            let candidate_priority = |target: &dyn Module| {
              let target_identifier = target.identifier();
              let supports_javascript = target
                .source_types(module_graph)
                .contains(&SourceType::JavaScript);
              let shares_chunk = current_module_chunks.is_some_and(|current_chunks| {
                chunk_graph.get_number_of_module_chunks(target_identifier) > 0
                  && chunk_graph
                    .get_module_chunks(target_identifier)
                    .iter()
                    .any(|chunk| current_chunks.contains(chunk))
              });
              (
                supports_javascript,
                shares_chunk,
                ChunkGraph::get_module_id(&compilation.module_ids_artifact, target_identifier)
                  .is_some(),
              )
            };
            let find_target_module = |dep_id: &rspack_core::DependencyId| {
              module_graph
                .get_module_by_dependency_id(dep_id)
                .map(|target| {
                  let priority = candidate_priority(target.as_ref());
                  (target, priority)
                })
            };
            let from = id
              .as_ref()
              .and_then(find_target_module)
              .or_else(|| {
                module
                  .get_dependencies()
                  .iter()
                  .filter(|dep_id| {
                    let dependency = module_graph.dependency_by_id(dep_id);
                    let request = if let Some(d) = dependency.as_module_dependency() {
                      Some(d.request())
                    } else {
                      dependency.as_context_dependency().map(|d| d.request())
                    };
                    request == Some(from_name.as_str())
                  })
                  .filter_map(find_target_module)
                  .max_by_key(|(_, priority)| *priority)
              })
              .map(|(target, _)| target)
              .and_then(|target| {
                if target
                  .source_types(module_graph)
                  .contains(&SourceType::JavaScript)
                {
                  Some(target)
                } else {
                  let target_name_for_condition = target.name_for_condition();
                  module_graph
                    .modules()
                    .filter_map(|(_, candidate)| {
                      (candidate.name_for_condition() == target_name_for_condition
                        && candidate
                          .source_types(module_graph)
                          .contains(&SourceType::JavaScript))
                      .then_some(candidate)
                    })
                    .max_by_key(|candidate| candidate_priority(candidate.as_ref()))
                    .or(Some(target))
                }
              })
              .expect("should have css from module");

            let from_exports_info = compilation
              .exports_info_artifact
              .get_exports_info_data(&from.identifier());
            if !from
              .source_types(module_graph)
              .contains(&SourceType::JavaScript)
            {
              let ident = if unescape_referenced_ident {
                unescape(ident)
              } else {
                Cow::Borrowed(ident.as_str())
              };
              let mut seen = HashSet::default();
              let resolved = self
                .resolve_static_css_export(from.as_ref(), ident.as_ref(), &mut seen)
                .expect("should resolve static css export");
              return json_stringify_str(&resolved);
            }
            let from_used_name = match from_exports_info
              .get_read_only_export_info(&Atom::from(ident.as_str()))
              .get_used_name(None, runtime)
            {
              Some(UsedNameItem::Str(name)) => {
                let name = if unescape_referenced_ident {
                  Cow::Owned(unescape(name.as_str()).into_owned())
                } else {
                  Cow::Borrowed(name.as_str())
                };
                json_stringify_str(name.as_ref())
              }
              _ => {
                let ident = if unescape_referenced_ident {
                  unescape(ident)
                } else {
                  Cow::Borrowed(ident.as_str())
                };
                json_stringify_str(ident.as_ref())
              }
            };

            let from = json_stringify(
              ChunkGraph::get_module_id(&compilation.module_ids_artifact, from.identifier())
                .expect("should have module"),
            );
            concat_string!(
              self
                .generate_context
                .runtime_template
                .render_runtime_globals(&RuntimeGlobals::REQUIRE),
              "(",
              from,
              ")[",
              from_used_name,
              "]"
            )
          }
        },
      )
      .collect::<Vec<_>>()
      .join(" + \" \" + ")
  }

  fn stringified_exports<'b>(
    &mut self,
    exports: FxIndexMap<&'b str, &'b FxIndexSet<CssExport>>,
  ) -> (&'static str, String) {
    let module = self.module;
    let mut stringified_exports = String::new();

    for (key, elements) in exports {
      let used_name: Cow<'_, str> = {
        let exports_info = self
          .generate_context
          .compilation
          .exports_info_artifact
          .get_exports_info_data(&module.identifier());
        let export_info = exports_info.get_read_only_export_info(&Atom::from(key));
        match export_info.get_used_name(None, self.generate_context.runtime) {
          Some(UsedNameItem::Str(name)) => Cow::Owned(name.to_string()),
          _ => Cow::Borrowed(key),
        }
      };

      stringified_exports.push_str("  ");
      stringified_exports.push_str(&json_stringify_str(&used_name));
      stringified_exports.push_str(": ");
      stringified_exports
        .push_str(&self.render_css_export_content(elements, CssExportRenderMode::Standard));

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

fn normalize_css_charset(css_text: String) -> String {
  let Some(caps) = REGEX_CHARSET.captures(&css_text) else {
    return css_text;
  };
  let Some(charset) = caps.get(1) else {
    return css_text;
  };
  let without_charsets = REGEX_CHARSET.replace_all(&css_text, "");
  concat_string!(
    "@charset ",
    charset.as_str(),
    ";\n",
    without_charsets.trim_start()
  )
}
