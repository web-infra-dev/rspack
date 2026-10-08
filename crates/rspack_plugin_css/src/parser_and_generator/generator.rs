use std::{
  borrow::{Borrow, Cow},
  sync::Arc,
};

use concat_string::concat_string;
use rspack_collections::IdentifierSet;
use rspack_core::{
  ChunkGraph, Compilation, Context, CssBuildInfo, CssExportType, CssModuleRenderCondition,
  Dependency, DependencyCodeGeneration, DependencyId, DependencyType, GenerateContext, Module,
  ModuleArgument, ModuleIdentifier, ModuleInitFragments, RESERVED_IDENTIFIER, RuntimeGlobals,
  RuntimeSpec, SourceType, TemplateContext, UsageState, UsedNameItem,
  css_module_render_conditions_identifier, get_runtime_key,
  rspack_sources::{BoxSource, OriginalSource, RawStringSource, ReplaceSource, Source, SourceExt},
  to_identifier,
};
use rspack_error::Result;
use rspack_hash::RspackHashDigest;
use rspack_intern::Atom;
use rspack_util::{fx_hash::FxIndexMap, itoa, json_stringify, json_stringify_str};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use smol_str::SmolStr;

use crate::{
  css_exports::{css_export_name, find_css_export_target, find_export},
  css_syntax::{escape_identifier, unescape_identifier},
  dependency::{
    CssIcssExportDependency, CssIcssImportDependency, CssIcssSymbolDependency, CssIcssSymbolKind,
    CssImportDependency, CssLocalIdentKind,
  },
  parser_and_generator::{
    CodeGenerationDataCssLocalIdents, CodeGenerationDataUnusedLocalIdent, CssExportsRef,
    CssSourceBuilder, get_used_exports,
  },
  utils::{
    LocalIdentOptions, css_generator_options, css_module_export_type,
    replace_css_module_id_placeholder,
  },
};

fn css_javascript_source_map_module_name(module: &dyn Module, context: &Context) -> String {
  concat_string!("css ", module.readable_identifier(context))
}

fn render_dependency_template(
  dependency: &dyn DependencyCodeGeneration,
  source: &mut ReplaceSource,
  context: &mut TemplateContext,
) {
  // Some dependencies only contribute to hashing. ICSS imports, for example,
  // are expanded by this generator and do not have a source template.
  let Some(template_type) = dependency.dependency_template() else {
    return;
  };
  if let Some(template) = context.compilation.get_dependency_template(template_type) {
    template.render(dependency, source, context)
  } else {
    panic!(
      "Can not find dependency template of {:?}",
      dependency.dependency_template()
    );
  }
}

struct CssImportedModule {
  module_identifier: ModuleIdentifier,
  render_conditions: Vec<CssModuleRenderCondition>,
}

// Both representations are collected during the same dependency traversal.
// Each temporary result is moved into its parent when the traversal returns.
#[derive(Default)]
struct CssExportValue<'a> {
  css: Option<Vec<CssExportText<'a>>>,
  js: Option<Vec<Cow<'a, str>>>,
}

// Raw literals borrow dependency data; generated identifiers retain their
// shared storage when a child generator returns its value to the parent.
enum CssExportText<'a> {
  Text(Cow<'a, str>),
  LocalIdent(SmolStr),
}

impl Borrow<str> for CssExportText<'_> {
  fn borrow(&self) -> &str {
    match self {
      Self::Text(text) => text,
      Self::LocalIdent(ident) => ident,
    }
  }
}

#[derive(Default, Clone, Copy)]
struct CssExportRequirements {
  css: bool,
  js: bool,
}

impl<'a> CssExportValue<'a> {
  fn append_text(&mut self, text: CssExportText<'a>, requirements: CssExportRequirements) {
    if requirements.js {
      self
        .js
        .get_or_insert_with(Vec::new)
        .push(Cow::Owned(json_stringify_str(text.borrow())));
    }
    if requirements.css {
      self.css.get_or_insert_with(Vec::new).push(text);
    }
  }
}

pub(crate) struct CssModuleGenerator<'a, 'g> {
  css_output: ReplaceSource,
  js_output: HashMap<DependencyId, String>,
  // Unescaped names shared by this code generation's generators. Module ID
  // placeholders are replaced when rendering CSS occurrences or JS exports.
  local_idents: Arc<HashMap<DependencyId, SmolStr>>,
  module: &'a dyn Module,
  css_build_info: &'a CssBuildInfo,
  generate_context: &'a mut GenerateContext<'g>,
  with_hmr: bool,
  export_type: Option<CssExportType>,
  exports_only: bool,
  es_module: bool,
  module_argument: Option<String>,
  css_inject_style: Option<String>,
  css_style_sheet: Option<String>,
  resolving_css_exports: HashSet<DependencyId>,
  css_output_prepared: bool,
  used_local_idents: HashSet<DependencyId>,
  outputs_prepared: bool,
}

impl<'a, 'g> CssModuleGenerator<'a, 'g> {
  pub async fn new(
    source: BoxSource,
    module: &'a dyn Module,
    css_build_info: &'a CssBuildInfo,
    generate_context: &'a mut GenerateContext<'g>,
    with_hmr: bool,
    es_module: bool,
  ) -> Result<Self> {
    let generator_options = css_generator_options(generate_context.module_generator_options);
    let local_idents = Self::generate_local_idents(&source, module, generate_context).await?;

    Ok(Self {
      css_output: ReplaceSource::new(source),
      js_output: Default::default(),
      local_idents,
      module,
      css_build_info,
      generate_context,
      with_hmr,
      export_type: css_module_export_type(module),
      exports_only: generator_options
        .exports_only
        .expect("should have exports_only"),
      es_module,
      module_argument: None,
      css_inject_style: None,
      css_style_sheet: None,
      resolving_css_exports: Default::default(),
      css_output_prepared: false,
      used_local_idents: Default::default(),
      outputs_prepared: false,
    })
  }

  async fn generate_local_idents(
    source: &BoxSource,
    module: &dyn Module,
    generate_context: &mut GenerateContext<'_>,
  ) -> Result<Arc<HashMap<DependencyId, SmolStr>>> {
    if let Some(idents) = generate_context
      .data
      .get::<CodeGenerationDataCssLocalIdents>()
      .and_then(|data| data.modules.get(&module.identifier()))
    {
      return Ok(Arc::clone(idents));
    }
    let mut idents = HashMap::default();
    let Some(first_local) = module.get_dependencies().iter().position(|dependency| {
      dependency
        .downcast_ref::<CssIcssExportDependency>()
        .is_some_and(|definition| definition.local_ident.is_some())
    }) else {
      return Ok(Self::store_local_idents(
        module.identifier(),
        idents,
        generate_context,
      ));
    };
    let compilation = generate_context.compilation;
    let normal_module = module
      .as_normal_module()
      .expect("CSS module should be a normal module");
    let hash = Self::cached_module_hash(module.identifier(), generate_context);
    let options = LocalIdentOptions::new(
      normal_module.resource_resolved_data(),
      source.source().into_string_lossy(),
      &compilation.options,
      css_generator_options(normal_module.get_generator_options()),
      hash,
    );
    for dependency in &module.get_dependencies()[first_local..] {
      let Some(definition) = dependency.downcast_ref::<CssIcssExportDependency>() else {
        continue;
      };
      let Some(kind) = definition.local_ident else {
        continue;
      };
      let ident = options.get_local_ident(&definition.value).await?;
      let ident = match kind {
        CssLocalIdentKind::Ident => ident,
        CssLocalIdentKind::DashedIdent => {
          format!("--{}", ident.strip_prefix("_--").unwrap_or(&ident))
        }
      };
      idents.insert(*definition.id(), ident.into());
    }
    Ok(Self::store_local_idents(
      module.identifier(),
      idents,
      generate_context,
    ))
  }

  fn store_local_idents(
    module_identifier: ModuleIdentifier,
    idents: HashMap<DependencyId, SmolStr>,
    generate_context: &mut GenerateContext<'_>,
  ) -> Arc<HashMap<DependencyId, SmolStr>> {
    let idents = Arc::new(idents);
    if !generate_context
      .data
      .contains::<CodeGenerationDataCssLocalIdents>()
    {
      generate_context
        .data
        .insert(CodeGenerationDataCssLocalIdents::default());
    }
    generate_context
      .data
      .get_mut::<CodeGenerationDataCssLocalIdents>()
      .expect("local ident data should have been initialized")
      .modules
      .insert(module_identifier, Arc::clone(&idents));
    idents
  }

  fn cached_module_hash<'c>(
    module_identifier: ModuleIdentifier,
    generate_context: &GenerateContext<'c>,
  ) -> &'c RspackHashDigest {
    let compilation = generate_context.compilation;
    let hashes = compilation
      .cgm_hash_artifact
      .get_runtime_map(&module_identifier)
      .expect("CSS module hashes should be stored before code generation");
    generate_context
      .runtime
      .and_then(|runtime| ChunkGraph::get_module_hash(compilation, module_identifier, runtime))
      // A child without a chunk has its runtime-independent hash stored here.
      .or_else(|| hashes.get(&RuntimeSpec::default()))
      // A child can belong to a shared chunk whose runtime includes the parent.
      .or_else(|| {
        let runtime = generate_context.runtime?;
        let child_runtime = compilation
          .build_chunk_graph_artifact
          .chunk_graph
          .get_module_runtimes_iter(
            module_identifier,
            &compilation.build_chunk_graph_artifact.chunk_by_ukey,
          )
          .filter(|child_runtime| runtime.is_subset(child_runtime))
          .min_by(|a, b| {
            a.len()
              .cmp(&b.len())
              .then_with(|| get_runtime_key(a).cmp(get_runtime_key(b)))
          })?;
        ChunkGraph::get_module_hash(compilation, module_identifier, child_runtime)
      })
      // Without a matching runtime, a common digest is still unambiguous.
      .or_else(|| {
        let mut values = hashes.values();
        let hash = values.next()?;
        values.all(|value| value == hash).then_some(hash)
      })
      .expect("CSS module should have a hash for its code generation runtime")
  }

  fn module_argument(&mut self) -> &str {
    self.module_argument.get_or_insert_with(|| {
      self
        .generate_context
        .runtime_template
        .render_module_argument(ModuleArgument::Module)
    })
  }

  fn css_inject_style(&mut self) -> &str {
    self.css_inject_style.get_or_insert_with(|| {
      self
        .generate_context
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::CSS_INJECT_STYLE)
    })
  }

  fn css_style_sheet(&mut self) -> &str {
    self.css_style_sheet.get_or_insert_with(|| {
      self
        .generate_context
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::CSS_STYLE_SHEET)
    })
  }

  fn collect_used_css_exports<'b>(&self) -> CssExportsRef<'b>
  where
    'a: 'b,
    'g: 'b,
  {
    let identifier = self.module.identifier();
    let runtime = self.generate_context.runtime;
    let exports_info_artifact = &self.generate_context.compilation.exports_info_artifact;
    get_used_exports(
      self.css_build_info,
      self.generate_context.compilation.get_module_graph(),
      identifier,
      runtime,
      exports_info_artifact,
    )
    .unwrap_or_default()
  }

  pub(crate) async fn generate_css_source(mut self) -> Result<BoxSource> {
    self
      .generate_context
      .runtime_template
      .runtime_requirements_mut()
      .insert(RuntimeGlobals::HAS_CSS_MODULES);
    let source = self.render_css_module_source().await?;
    self.store_unused_local_idents();
    Ok(source)
  }

  pub async fn generate_javascript_source(mut self) -> Result<BoxSource> {
    let generated_source = match self.export_type {
      Some(CssExportType::Text) => {
        let css = self.css_text_expr_with_imports().await?;
        self.generate_css_text_exports(&css)?
      }
      Some(CssExportType::CssStyleSheet) => {
        let css = self.css_text_expr_with_imports().await?;
        self.generate_css_style_sheet_exports(&css)?
      }
      Some(CssExportType::Style) if !self.exports_only => {
        let mut visited_inlined_modules = HashSet::default();
        let imports = self
          .render_style_imports(&mut visited_inlined_modules)
          .await?;
        let css_source = self.render_css_module_source().await?;
        let css = self.css_text_expr(css_source, &[]);
        let inject_style = self.render_css_inject_style(&css);
        let exports = self.generate_js_exports().await?;
        concat_string!(imports, inject_style, exports)
      }
      _ => self.generate_js_exports().await?,
    };
    self.store_unused_local_idents();
    if self.module.get_source_map_kind().enabled() {
      let source_name = css_javascript_source_map_module_name(
        self.module,
        &self.generate_context.compilation.options.context,
      );
      Ok(OriginalSource::new(generated_source, source_name).boxed())
    } else {
      Ok(RawStringSource::from(generated_source).boxed())
    }
  }

  async fn generate_js_exports(&mut self) -> Result<String> {
    self.prepare_outputs(false).await?;
    if self.generate_context.concatenation_scope.is_some() {
      let exports = self.collect_used_css_exports();
      return self.concat_css_exports_inner(None, exports);
    }

    let (ns_obj, left, right) = self.render_namespace_object_parts();

    let used_exports = self.collect_used_css_exports();
    let exports_str = if !used_exports.is_empty() {
      let (decl_name, exports_string) = self.stringified_exports(used_exports);
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

    Ok(exports_str)
  }

  async fn child_generator<'b>(
    &'b mut self,
    source: BoxSource,
    module: &'b dyn Module,
    css_build_info: &'b CssBuildInfo,
  ) -> Result<CssModuleGenerator<'b, 'g>> {
    CssModuleGenerator::new(
      source,
      module,
      css_build_info,
      self.generate_context,
      self.with_hmr,
      self.es_module,
    )
    .await
  }

  pub(crate) async fn render_css_module_source(&mut self) -> Result<BoxSource> {
    self.prepare_outputs(true).await?;
    Ok(
      std::mem::replace(
        &mut self.css_output,
        ReplaceSource::new(RawStringSource::from("")),
      )
      .boxed(),
    )
  }

  async fn prepare_outputs(&mut self, render_css: bool) -> Result<()> {
    if self.outputs_prepared && (!render_css || self.css_output_prepared) {
      return Ok(());
    }
    self.outputs_prepared = true;

    let roots = self.collect_css_export_roots(render_css);
    let css_values = self.resolve_css_exports(roots).await?;
    if render_css {
      self.render_css_dependencies(&css_values);
      self.css_output_prepared = true;
    }
    Ok(())
  }

  fn collect_css_export_roots(
    &self,
    render_css: bool,
  ) -> FxIndexMap<DependencyId, CssExportRequirements> {
    let emit_js = self.generate_context.requested_source_type == SourceType::JavaScript;
    let mut roots = FxIndexMap::default();
    // Used exports keep local identifiers alive without needing their CSS value.
    // Only ICSS references that rewrite CSS request the expanded CSS value.
    for dependency in self.collect_used_css_exports().values() {
      roots.insert(
        *dependency.id(),
        CssExportRequirements {
          css: false,
          js: emit_js,
        },
      );
    }
    for dependency in self.module.get_dependencies() {
      if let Some(symbol) = dependency.downcast_ref::<CssIcssSymbolDependency>()
        && symbol.kind != CssIcssSymbolKind::LocalDeclaration
      {
        // Local symbols such as `animation: spin` keep their definitions alive
        // even though rendering the use only needs the generated identifier.
        roots.entry(symbol.target).or_default().css |=
          render_css && symbol.kind == CssIcssSymbolKind::IcssReference;
      }
    }
    roots
  }

  async fn resolve_css_exports(
    &mut self,
    roots: FxIndexMap<DependencyId, CssExportRequirements>,
  ) -> Result<HashMap<DependencyId, Option<String>>> {
    let mut css_values = HashMap::default();
    for (id, requirements) in roots {
      let mut value = self
        .resolve_css_export_dependency(id, requirements, true)
        .await?;
      if requirements.js {
        if value.js.is_none() {
          // A pure self-import cycle still needs a valid runtime expression.
          value.js = self
            .resolve_css_export_dependency(
              id,
              CssExportRequirements {
                css: false,
                js: true,
              },
              false,
            )
            .await?
            .js;
        }
        self.js_output.insert(
          id,
          value
            .js
            .map_or_else(|| json_stringify_str(""), |parts| parts.join(" + ")),
        );
      }
      if requirements.css {
        css_values.insert(id, value.css.map(|parts| parts.join("")));
      }
    }
    Ok(css_values)
  }

  fn render_local_ident(compilation: &Compilation, module: &dyn Module, ident: &str) -> String {
    escape_identifier(&replace_css_module_id_placeholder(
      ident,
      compilation,
      module,
    ))
    .into_owned()
  }

  fn render_css_dependencies(&mut self, css_values: &HashMap<DependencyId, Option<String>>) {
    let mut escaped_local_idents = HashMap::default();
    let mut init_fragments = ModuleInitFragments::default();
    let mut context = TemplateContext {
      compilation: self.generate_context.compilation,
      module: self.module,
      runtime: self.generate_context.runtime,
      init_fragments: &mut init_fragments,
      concatenation_scope: self.generate_context.concatenation_scope.take(),
      data: self.generate_context.data,
      runtime_template: self.generate_context.runtime_template,
    };
    // Keep each source occurrence in parser order, including imports and URLs.
    // Grouping replacements by definition makes ReplaceSource repeatedly shift
    // its sorted replacement vector for interleaved occurrences.
    for dependency in self.module.get_dependencies() {
      if let Some(symbol) = dependency.downcast_ref::<CssIcssSymbolDependency>() {
        let value = match symbol.kind {
          CssIcssSymbolKind::LocalDeclaration | CssIcssSymbolKind::LocalReference => {
            let value = escaped_local_idents
              .entry(symbol.target)
              .or_insert_with(|| {
                let ident = self
                  .local_idents
                  .get(&symbol.target)
                  .expect("local identifier should be generated before CSS rendering");
                Self::render_local_ident(context.compilation, context.module, ident)
              });
            Some(value.as_str())
          }
          CssIcssSymbolKind::IcssReference => {
            css_values.get(&symbol.target).and_then(Option::as_deref)
          }
        };
        if let Some(value) = value {
          let range = symbol
            .range()
            .expect("CSS symbol should have a source range");
          self
            .css_output
            .replace(range.start, range.end, value.to_owned(), None);
        }
      } else if dependency.dependency_type() != &DependencyType::CssIcssExport
        && let Some(dependency) = dependency.as_dependency_code_generation()
      {
        render_dependency_template(dependency, &mut self.css_output, &mut context);
      }
    }
    if let Some(dependencies) = self.module.get_presentational_dependencies() {
      for dependency in dependencies {
        render_dependency_template(dependency.as_ref(), &mut self.css_output, &mut context);
      }
    }
    self.generate_context.concatenation_scope = context.concatenation_scope.take();
  }

  fn store_unused_local_idents(&mut self) {
    let Some(local_names) = self.css_build_info.local_names() else {
      return;
    };
    let idents = local_names
      .values()
      .filter(|id| !self.used_local_idents.contains(id))
      .map(|id| {
        self
          .local_idents
          .get(id)
          .expect("declared local should have a generated identifier")
          .clone()
      })
      .collect();
    // Only the top-level generator publishes data; children share its context
    // but track identifiers belonging to their own module.
    self
      .generate_context
      .data
      .insert(CodeGenerationDataUnusedLocalIdent { idents });
  }

  async fn css_text_expr_with_imports(&mut self) -> Result<String> {
    let has_css_imports = self
      .module
      .get_dependencies()
      .iter()
      .any(|dependency| matches!(dependency.dependency_type(), DependencyType::CssImport));
    if !has_css_imports {
      let css_source = self.render_css_module_source().await?;
      return Ok(self.css_text_expr(css_source, &[]));
    }

    let mut seen = IdentifierSet::default();
    let mut builder = self.css_source_builder(false);
    let render_conditions = self
      .css_build_info
      .render_conditions()
      .cloned()
      .collect::<Vec<_>>();
    self
      .render_ordered_css_sources(&mut builder, &render_conditions, &mut seen)
      .await?;
    Ok(json_stringify_str(&builder.into_css_text()))
  }

  async fn render_ordered_css_sources(
    &mut self,
    builder: &mut CssSourceBuilder,
    render_conditions: &[CssModuleRenderCondition],
    seen: &mut IdentifierSet,
  ) -> Result<()> {
    let module = self.module;
    if !seen.insert(module.identifier()) {
      return Ok(());
    }

    self.render_css_import_sources(builder, seen).await?;
    let css_source = self.render_css_module_source().await?;
    if !css_source.source().is_empty() {
      if self.css_build_info.has_charset {
        builder.set_has_charset();
      }
      builder.push_css_source(
        css_source,
        render_conditions,
        self.css_build_info.has_charset,
      );
    }
    seen.remove(&module.identifier());
    Ok(())
  }

  async fn render_css_import_sources(
    &mut self,
    builder: &mut CssSourceBuilder,
    seen: &mut IdentifierSet,
  ) -> Result<()> {
    let compilation = self.generate_context.compilation;
    let module_graph = compilation.get_module_graph();

    for css_import in self.css_import_modules() {
      let Some(imported_module) = module_graph.module_by_identifier(&css_import.module_identifier)
      else {
        continue;
      };
      let Some(imported_source) = imported_module.source() else {
        continue;
      };

      let build_info = imported_module.build_info();
      let css_build_info = build_info
        .css
        .as_deref()
        .expect("imported CSS module should have build info");
      let mut child = self
        .child_generator(
          imported_source.clone(),
          imported_module.as_ref(),
          css_build_info,
        )
        .await?;
      Box::pin(child.render_ordered_css_sources(builder, &css_import.render_conditions, seen))
        .await?;
    }
    Ok(())
  }

  fn css_import_modules(&self) -> impl Iterator<Item = CssImportedModule> + 'a {
    let compilation = self.generate_context.compilation;
    let module_graph = compilation.get_module_graph();

    self
      .module
      .get_dependencies()
      .iter()
      .filter_map(move |dependency| {
        if !matches!(dependency.dependency_type(), DependencyType::CssImport) {
          return None;
        }
        let Some(css_import_dep) = dependency.downcast_ref::<CssImportDependency>() else {
          panic!(
            "dependency with type DependencyType::CssImport should only be CssImportDependency"
          );
        };
        let imported_module = module_graph.module_graph_module_by_dependency_id(dependency.id())?;

        Some(CssImportedModule {
          module_identifier: imported_module.module_identifier,
          render_conditions: css_import_dep.render_conditions().cloned().collect(),
        })
      })
  }

  fn css_text_expr(
    &self,
    css_source: BoxSource,
    render_conditions: &[CssModuleRenderCondition],
  ) -> String {
    let mut builder = self.css_source_builder(self.css_build_info.has_charset);
    builder.push_css_source(
      css_source,
      render_conditions,
      self.css_build_info.has_charset,
    );
    json_stringify_str(&builder.into_css_text())
  }

  fn css_source_builder(&self, with_charset: bool) -> CssSourceBuilder {
    CssSourceBuilder::new(
      with_charset,
      !self.module.get_source_map_kind().no_sources(),
      self.generate_context.compilation.options.context.clone(),
    )
  }

  fn render_require_call_parts(&mut self) -> (String, &'static str, &'static str) {
    (
      self
        .generate_context
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::REQUIRE),
      "(",
      ")",
    )
  }

  fn render_namespace_object_parts(&mut self) -> (String, &'static str, &'static str) {
    let exports_info = self
      .generate_context
      .compilation
      .exports_info_artifact
      .get_exports_info_data(&self.module.identifier());
    if !self.es_module
      || exports_info
        .other_exports_info()
        .get_used(self.generate_context.runtime)
        == UsageState::Unused
    {
      return (String::new(), "", "");
    }

    (
      self
        .generate_context
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::MAKE_NAMESPACE_OBJECT),
      "(",
      ")",
    )
  }

  fn render_require_property_access(
    &mut self,
    module_identifier: ModuleIdentifier,
    property: &str,
  ) -> String {
    let module_id = json_stringify(
      ChunkGraph::get_module_id(
        &self.generate_context.compilation.module_ids_artifact,
        module_identifier,
      )
      .expect("should have module"),
    );
    let (require, require_left, require_right) = self.render_require_call_parts();
    concat_string!(
      require,
      require_left,
      module_id,
      require_right,
      "[",
      property,
      "]"
    )
  }

  fn stringified_used_export_name(
    &self,
    module_identifier: ModuleIdentifier,
    ident: &str,
    should_unescape: bool,
  ) -> String {
    let exports_info = self
      .generate_context
      .compilation
      .exports_info_artifact
      .get_exports_info_data(&module_identifier);
    let used_name = exports_info
      .get_read_only_export_info(&Atom::from(ident))
      .get_used_name(None, self.generate_context.runtime);
    match used_name {
      Some(UsedNameItem::Str(name)) if should_unescape => {
        json_stringify_str(&unescape_identifier(name.as_str()))
      }
      Some(UsedNameItem::Str(name)) => json_stringify_str(name.as_str()),
      _ if should_unescape => json_stringify_str(&unescape_identifier(ident)),
      _ => json_stringify_str(ident),
    }
  }

  async fn render_style_imports(
    &mut self,
    visited_inlined_modules: &mut HashSet<String>,
  ) -> Result<String> {
    let compilation = self.generate_context.compilation;
    let module_graph = compilation.get_module_graph();
    let (require, require_left, require_right) = self.render_require_call_parts();
    let mut code = String::new();

    let has_render_condition = !self.css_build_info.has_render_conditions();

    for css_import in self.css_import_modules() {
      let Some(module_id) = ChunkGraph::get_module_id(
        &compilation.module_ids_artifact,
        css_import.module_identifier,
      ) else {
        continue;
      };

      let Some(imported_module) = module_graph.module_by_identifier(&css_import.module_identifier)
      else {
        continue;
      };

      if matches!(
        css_module_export_type(imported_module.as_ref()),
        Some(CssExportType::Style)
      ) && has_render_condition
        && css_import.render_conditions.is_empty()
      {
        let is_concatenated_import = self
          .generate_context
          .concatenation_scope
          .as_ref()
          .is_some_and(|scope| {
            scope.is_module_in_scope(&css_import.module_identifier)
              && scope.is_module_concatenated(&css_import.module_identifier)
          });
        if !is_concatenated_import {
          code.push_str(&concat_string!(
            require,
            require_left,
            json_stringify(module_id),
            require_right,
            ";\n"
          ));
        }
        continue;
      }

      let Some(source) = imported_module.source() else {
        continue;
      };
      let render_conditions_key =
        css_module_render_conditions_identifier(&css_import.render_conditions).unwrap_or_default();
      let inlined_module_key = concat_string!(
        imported_module.identifier().as_str(),
        "|",
        render_conditions_key
      );
      if !visited_inlined_modules.insert(inlined_module_key) {
        continue;
      }

      let build_info = imported_module.build_info();
      let css_build_info = build_info
        .css
        .as_deref()
        .expect("imported CSS module should have build info");
      let mut child = self
        .child_generator(source.clone(), imported_module.as_ref(), css_build_info)
        .await?;
      code.push_str(&Box::pin(child.render_style_imports(visited_inlined_modules)).await?);
      let css_source = child.render_css_module_source().await?;
      let css = child.css_text_expr(css_source, &css_import.render_conditions);
      let style_module_id = if render_conditions_key.is_empty() {
        module_id.to_string()
      } else {
        concat_string!(module_id.to_string(), "|", render_conditions_key)
      };
      code.push_str(&self.render_inject_style_call(json_stringify_str(&style_module_id), &css));
    }

    Ok(code)
  }

  fn render_css_inject_style(&mut self, css: &str) -> String {
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

    self.render_inject_style_call(json_stringify_str(&module_id), css)
  }

  fn render_inject_style_call(&mut self, module_id: String, css: &str) -> String {
    let css_inject_style = self.css_inject_style();
    concat_string!(css_inject_style, "(", module_id, ", ", css, ");\n")
  }

  fn generate_css_style_sheet_exports(&mut self, css: &str) -> Result<String> {
    let css_style_sheet = self.css_style_sheet();
    let css_style_sheet_expr = concat_string!(css_style_sheet, "(", css, ")");
    if self.generate_context.concatenation_scope.is_some() {
      return self.concat_css_exports_with_default(Some(css_style_sheet_expr));
    }

    let sheet_code = concat_string!("var __css_style_sheet = ", css_style_sheet_expr, ";\n");

    Ok(self.generate_css_default_exports(&sheet_code, "__css_style_sheet"))
  }

  fn generate_css_text_exports(&mut self, css: &str) -> Result<String> {
    if self.generate_context.concatenation_scope.is_some() {
      return self.concat_css_exports_with_default(Some(css.to_string()));
    }

    Ok(self.generate_css_default_exports("", css))
  }

  fn generate_css_default_exports(&mut self, prelude: &str, default_expr: &str) -> String {
    let module_argument = self.module_argument().to_string();
    let (ns_obj, left, right) = self.render_namespace_object_parts();

    let used_exports = self.collect_used_css_exports();

    if !used_exports.is_empty() {
      let (decl_name, exports_string) = self.stringified_exports(used_exports);
      concat_string!(
        prelude,
        exports_string,
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
        left,
        module_argument,
        ".exports = {\n\t\"default\": ",
        default_expr,
        "\n}",
        right,
        ";\n"
      )
    } else {
      concat_string!(prelude, module_argument, ".exports = ", default_expr, ";\n")
    }
  }

  fn concat_css_exports_with_default(&mut self, default_expr: Option<String>) -> Result<String> {
    let exports = self.collect_used_css_exports();
    self.concat_css_exports_inner(default_expr, exports)
  }

  fn concat_css_exports_inner<'b>(
    &mut self,
    default_expr: Option<String>,
    exports: FxIndexMap<&'b str, &'b CssIcssExportDependency>,
  ) -> Result<String> {
    if self.generate_context.concatenation_scope.is_none() {
      return Ok(String::new());
    }

    let module = self.module;
    let compilation = self.generate_context.compilation;
    let runtime = self.generate_context.runtime;
    let exports_info = compilation
      .exports_info_artifact
      .get_exports_info_data(&module.identifier());
    let mut used_identifiers = HashSet::default();
    let mut source = String::new();

    if let Some(default_expr) = default_expr {
      let export_info = exports_info.get_read_only_export_info(&Atom::from("default"));
      if let Some(UsedNameItem::Str(used_name)) = export_info.get_used_name(None, runtime) {
        source.push_str(&self.register_concat_export(
          "default",
          &default_expr,
          &used_name,
          &mut used_identifiers,
        ));
      }
    }

    for (key, _) in exports {
      let export_info = exports_info.get_read_only_export_info(&Atom::from(key));
      let used_name = export_info.get_used_name(None, runtime);
      let used_name: Cow<'_, str> = match used_name {
        Some(UsedNameItem::Str(name)) => Cow::Owned(name.to_string()),
        _ => Cow::Borrowed(key),
      };

      let content = self.render_css_export_content(key);
      source.push_str(&self.register_concat_export(
        key,
        &content,
        &used_name,
        &mut used_identifiers,
      ));
    }

    Ok(source)
  }

  fn register_concat_export(
    &mut self,
    key: &str,
    content: &str,
    used_name: &str,
    used_identifiers: &mut HashSet<SmolStr>,
  ) -> String {
    let mut identifier = to_identifier(used_name).into_owned();
    if identifier.is_empty() || RESERVED_IDENTIFIER.contains(identifier.as_str()) {
      identifier = concat_string!("_", identifier);
    }
    let base_identifier = identifier.clone();
    let mut i = 0;
    while used_identifiers.contains(identifier.as_str()) {
      let mut i_buffer = itoa::Buffer::new();
      let i_str = i_buffer.format(i);
      identifier = concat_string!(base_identifier, i_str);
      i += 1;
    }

    let export_source = concat_string!("var ", identifier, " = ", content, ";\n");
    used_identifiers.insert(SmolStr::new(&identifier));
    let Some(ref mut scope) = self.generate_context.concatenation_scope else {
      unreachable!();
    };
    scope.register_export(key.into(), identifier);
    export_source
  }

  fn render_css_export_content(&self, name: &str) -> String {
    let Some(id) = find_export(self.module, name) else {
      return String::new();
    };
    self
      .js_output
      .get(&id)
      .expect("CSS export should be collected before rendering JavaScript")
      .clone()
  }

  async fn resolve_css_export_dependency(
    &mut self,
    id: DependencyId,
    requirements: CssExportRequirements,
    follow_self_imports: bool,
  ) -> Result<CssExportValue<'g>> {
    if !self.resolving_css_exports.insert(id) {
      return Ok(CssExportValue::default());
    }
    let graph = self.generate_context.compilation.get_module_graph();
    let dependency = graph.dependency_by_id(&id);
    let value = if let Some(import) = dependency.downcast_ref::<CssIcssImportDependency>() {
      self
        .resolve_css_import(import, requirements, follow_self_imports)
        .await?
    } else if let Some(export) = dependency.downcast_ref::<CssIcssExportDependency>() {
      if export.local_ident.is_some() {
        self.used_local_idents.insert(id);
      }
      self
        .resolve_css_export_definition(export, requirements, follow_self_imports)
        .await?
    } else {
      CssExportValue::default()
    };
    self.resolving_css_exports.remove(&id);
    Ok(value)
  }

  async fn resolve_css_import(
    &mut self,
    import: &CssIcssImportDependency,
    requirements: CssExportRequirements,
    follow_self_imports: bool,
  ) -> Result<CssExportValue<'g>> {
    let compilation = self.generate_context.compilation;
    let id = *import.id();
    let Some(target) = find_css_export_target(compilation, &id) else {
      // Factorization already reported the unresolved import. Code generation
      // must still finish without trying to render a reference to its target.
      return Ok(CssExportValue::default());
    };
    let target_export = find_export(target, import.import_name());
    let is_self_import = target.identifier() == self.module.identifier();
    let mut value = if let Some(target_export) = target_export {
      if is_self_import {
        Box::pin(self.resolve_css_export_dependency(
          target_export,
          CssExportRequirements {
            css: requirements.css,
            js: requirements.js && follow_self_imports,
          },
          follow_self_imports,
        ))
        .await?
      } else if requirements.css {
        self
          .resolve_imported_css_export(target, target_export)
          .await?
      } else {
        CssExportValue::default()
      }
    } else {
      CssExportValue::default()
    };

    if requirements.js && (!is_self_import || !follow_self_imports) {
      value.js = Some(vec![Cow::Owned(
        self
          .render_css_import_expression(id, import.import_name(), &mut value.css)
          .await?,
      )]);
    }
    if !requirements.css {
      value.css = None;
    }
    Ok(value)
  }

  async fn resolve_imported_css_export(
    &mut self,
    target: &dyn Module,
    id: DependencyId,
  ) -> Result<CssExportValue<'g>> {
    let build_info = target.build_info();
    let Some(css_build_info) = build_info.css.as_deref() else {
      return Ok(CssExportValue::default());
    };
    // Share cycle detection across modules while keeping local usage in each
    // generator. A child's identifiers must not enter its parent's used set.
    let active = std::mem::take(&mut self.resolving_css_exports);
    let mut child = self
      .child_generator(
        target
          .source()
          .expect("CSS module should have a source")
          .clone(),
        target,
        css_build_info,
      )
      .await?;
    child.resolving_css_exports = active;
    let value = Box::pin(child.resolve_css_export_dependency(
      id,
      CssExportRequirements {
        css: true,
        js: false,
      },
      true,
    ))
    .await?;
    let active = std::mem::take(&mut child.resolving_css_exports);
    drop(child);
    self.resolving_css_exports = active;
    Ok(value)
  }

  async fn resolve_css_export_definition(
    &mut self,
    export: &'g CssIcssExportDependency,
    requirements: CssExportRequirements,
    follow_self_imports: bool,
  ) -> Result<CssExportValue<'g>> {
    let compilation = self.generate_context.compilation;
    let mut resolved = CssExportValue::default();
    if export.local_ident.is_some() {
      let ident = self
        .local_idents
        .get(export.id())
        .expect("local identifier should be generated before export rendering")
        .clone();
      let text = match replace_css_module_id_placeholder(&ident, compilation, self.module) {
        Cow::Borrowed(_) => CssExportText::LocalIdent(ident),
        Cow::Owned(text) => CssExportText::Text(Cow::Owned(text)),
      };
      resolved.append_text(text, requirements);
    } else {
      let raw_value = &export.value;
      let mut cursor = 0;
      for reference in &export.references {
        let text = &raw_value[cursor..reference.range.start as usize];
        if (requirements.css || requirements.js) && !text.is_empty() {
          let text = replace_css_module_id_placeholder(text, compilation, self.module);
          resolved.append_text(CssExportText::Text(text), requirements);
        }
        let value = Box::pin(self.resolve_css_export_dependency(
          reference.dependency_id,
          requirements,
          follow_self_imports,
        ))
        .await?;
        if let Some(parts) = value.css {
          resolved.css.get_or_insert_with(Vec::new).extend(parts);
        }
        if let Some(parts) = value.js {
          resolved.js.get_or_insert_with(Vec::new).extend(parts);
        }
        cursor = reference.range.end as usize;
      }

      let tail = &raw_value[cursor..];
      if (requirements.css || requirements.js) && (!tail.is_empty() || export.references.is_empty())
      {
        let text = replace_css_module_id_placeholder(tail, compilation, self.module);
        resolved.append_text(CssExportText::Text(text), requirements);
      }
    }
    for compose in &export.composes {
      let value =
        Box::pin(self.resolve_css_export_dependency(*compose, requirements, follow_self_imports))
          .await?;
      if let Some(parts) = value.css {
        if let Some(parts) = &mut resolved.css {
          parts.push(CssExportText::Text(Cow::Borrowed(" ")));
        }
        resolved.css.get_or_insert_with(Vec::new).extend(parts);
      }
      if let Some(parts) = value.js {
        if let Some(parts) = &mut resolved.js {
          parts.push(Cow::Owned(json_stringify_str(" ")));
        }
        resolved.js.get_or_insert_with(Vec::new).extend(parts);
      }
    }
    Ok(resolved)
  }

  async fn render_css_import_expression(
    &mut self,
    id: DependencyId,
    import_name: &str,
    css_value: &mut Option<Vec<CssExportText<'g>>>,
  ) -> Result<String> {
    if self.generate_context.concatenation_scope.is_some() {
      return self
        .render_concat_reexport(import_name, &id, css_value)
        .await;
    }

    let (target_identifier, export_name) = {
      let compilation = self.generate_context.compilation;
      let target =
        find_css_export_target(compilation, &id).expect("CSS import should resolve to a module");
      (target.identifier(), css_export_name(target, import_name))
    };
    let used_name = self.stringified_used_export_name(target_identifier, &export_name, true);
    Ok(self.render_require_property_access(target_identifier, &used_name))
  }

  async fn render_concat_reexport(
    &mut self,
    ident: &str,
    id: &DependencyId,
    css_value: &mut Option<Vec<CssExportText<'g>>>,
  ) -> Result<String> {
    let compilation = self.generate_context.compilation;
    let module = self.module;
    let module_graph = compilation.get_module_graph();
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
        ChunkGraph::get_module_id(&compilation.module_ids_artifact, target_identifier).is_some(),
      )
    };
    let from = module_graph
      .get_module_by_dependency_id(id)
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

    if !from
      .source_types(module_graph)
      .contains(&SourceType::JavaScript)
    {
      // Only a static fallback needs the target's CSS value. Runtime module
      // references must not recursively expand an unused representation.
      if css_value.is_none() {
        let export = find_export(from.as_ref(), ident).expect("should have CSS export");
        *css_value = self
          .resolve_imported_css_export(from.as_ref(), export)
          .await?
          .css;
      }
      Ok(json_stringify_str(
        &css_value
          .as_ref()
          .expect("should resolve static css export")
          .join(""),
      ))
    } else {
      let from_used_name = self.stringified_used_export_name(
        from.identifier(),
        &crate::css_exports::css_export_name(from.as_ref(), ident),
        false,
      );
      Ok(self.render_require_property_access(from.identifier(), &from_used_name))
    }
  }

  fn stringified_exports<'b>(
    &mut self,
    exports: FxIndexMap<&'b str, &'b CssIcssExportDependency>,
  ) -> (&'static str, String) {
    let module = self.module;
    let mut stringified_exports = String::new();

    for (key, _) in exports {
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
      stringified_exports.push_str(&self.render_css_export_content(key));

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
