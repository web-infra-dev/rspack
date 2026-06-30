use std::{borrow::Cow, fmt::Write, sync::Arc};

use async_trait::async_trait;
use cow_utils::CowUtils;
use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsCacheable, AsMap, AsVec},
};
use rspack_collections::{Identifiable, Identifier};
use rspack_core::{
  AsyncDependenciesBlock, AsyncDependenciesBlockIdentifier, BoxDependency, BoxModule, BuildContext,
  BuildInfo, BuildMeta, BuildMetaExportsType, BuildResult, CodeGenerationResult, Compilation,
  Context, DependenciesBlock, Dependency, DependencyId, DependencyRange, FactoryMeta, ImportPhase,
  LibIdentOptions, Module, ModuleCodeGenerationContext, ModuleGraph, ModuleIdentifier, ModuleLayer,
  ModuleType, ReferencedSpecifier, RuntimeSpec, SourceType, contextify, impl_module_meta_info,
  impl_source_map_config, module_update_hash,
  rspack_sources::{BoxSource, RawStringSource, SourceExt},
};
use rspack_error::{Result, impl_empty_diagnosable_trait};
use rspack_hash::{RspackHash, RspackHashDigest};
use rspack_plugin_javascript::dependency::ImportEagerDependency;
use rspack_util::{fx_hash::FxIndexSet, source_map::SourceMapKind};
use rustc_hash::FxHashSet;
use swc_core::ecma::atoms::Atom;

use crate::{
  client_reference_dependency::ClientReferenceDependency,
  constants::LAYERS_NAMES,
  plugin_state::{ClientModuleImport, ClientModulesByServerEntry, CssImportsByServerEntry},
};

#[impl_source_map_config]
#[cacheable]
#[derive(Debug)]
pub struct RscEntryModule {
  blocks: Vec<AsyncDependenciesBlockIdentifier>,
  dependencies: Vec<DependencyId>,
  identifier: ModuleIdentifier,
  lib_ident: String,
  client_modules: Vec<ClientModuleImport>,
  root_client_modules: Vec<ClientModuleImport>,
  #[cacheable(with=AsMap<AsCacheable, AsVec>)]
  client_modules_by_server_entry: ClientModulesByServerEntry,
  #[cacheable(with=AsMap<AsCacheable, AsVec>)]
  css_imports_by_server_entry: CssImportsByServerEntry,
  name: Arc<str>,
  /// When true, client modules are loaded eagerly (not as code-split points).
  is_server_side_rendering: bool,
  factory_meta: Option<FactoryMeta>,
  build_info: BuildInfo,
  build_meta: BuildMeta,
  layer: Option<ModuleLayer>,
}

impl RscEntryModule {
  pub fn new(
    name: Arc<str>,
    client_modules: Vec<ClientModuleImport>,
    root_client_modules: Vec<ClientModuleImport>,
    client_modules_by_server_entry: ClientModulesByServerEntry,
    css_imports_by_server_entry: CssImportsByServerEntry,
    is_server_side_rendering: bool,
  ) -> Self {
    let lib_ident = format!("rspack/rsc-entry?name={}", &name);
    let identifier = create_identifier(
      name.as_ref(),
      &client_modules,
      &root_client_modules,
      &client_modules_by_server_entry,
      &css_imports_by_server_entry,
      is_server_side_rendering,
    );
    let layer = if is_server_side_rendering {
      Some(LAYERS_NAMES.server_side_rendering.to_string())
    } else {
      None
    };

    Self {
      blocks: Vec::new(),
      dependencies: Vec::new(),
      identifier,
      lib_ident,
      client_modules,
      root_client_modules,
      client_modules_by_server_entry,
      css_imports_by_server_entry,
      name,
      is_server_side_rendering,
      factory_meta: None,
      build_info: BuildInfo {
        strict: true,
        top_level_declarations: Some(FxHashSet::default()),
        ..Default::default()
      },
      build_meta: BuildMeta {
        exports_type: BuildMetaExportsType::Namespace,
        ..Default::default()
      },
      source_map_kind: SourceMapKind::empty(),
      layer,
    }
  }

  fn render_debug_comments(&self, compilation: &Compilation) -> String {
    let mut source = String::new();
    let root_chunking = self.debug_chunking("single async block");
    let ungrouped_chunking = self.debug_chunking("one async block per module");
    let server_entry_chunking = self.debug_chunking("one async block per server-entry");

    if self.is_server_side_rendering
      && self.root_client_modules.is_empty()
      && self.client_modules_by_server_entry.is_empty()
      && self.css_imports_by_server_entry.is_empty()
    {
      append_client_modules_debug_section(
        &mut source,
        compilation,
        "ssr-eager",
        "eager import for SSR",
        None,
        &self.client_modules,
      );
      return source;
    }

    append_client_modules_debug_section(
      &mut source,
      compilation,
      "root",
      root_chunking,
      None,
      &self.root_client_modules,
    );
    append_client_modules_debug_section(
      &mut source,
      compilation,
      "ungrouped",
      ungrouped_chunking,
      None,
      &self.client_modules,
    );

    for server_entry in self.debug_server_entries() {
      append_server_entry_debug_section(
        &mut source,
        compilation,
        server_entry.as_str(),
        server_entry_chunking,
        self.css_imports_by_server_entry.get(&server_entry),
        self.client_modules_by_server_entry.get(&server_entry),
      );
    }

    source
  }

  fn debug_chunking(&self, async_chunking: &'static str) -> &'static str {
    if self.is_server_side_rendering {
      "eager import for SSR"
    } else {
      async_chunking
    }
  }

  fn debug_server_entries(&self) -> Vec<String> {
    let mut server_entries = self
      .css_imports_by_server_entry
      .keys()
      .cloned()
      .collect::<Vec<_>>();
    for server_entry in self.client_modules_by_server_entry.keys() {
      if !server_entries.contains(server_entry) {
        server_entries.push(server_entry.clone());
      }
    }
    server_entries.sort_unstable();
    server_entries
  }

  fn all_client_modules(&self) -> Vec<&ClientModuleImport> {
    let mut client_modules = Vec::new();
    client_modules.extend(self.client_modules.iter());
    client_modules.extend(self.root_client_modules.iter());
    for modules in self.client_modules_by_server_entry.values() {
      client_modules.extend(modules.iter());
    }
    client_modules
  }
}

impl Identifiable for RscEntryModule {
  fn identifier(&self) -> Identifier {
    self.identifier
  }
}

impl DependenciesBlock for RscEntryModule {
  fn add_block_id(&mut self, block: AsyncDependenciesBlockIdentifier) {
    self.blocks.push(block)
  }

  fn get_blocks(&self) -> &[AsyncDependenciesBlockIdentifier] {
    &self.blocks
  }

  fn add_dependency_id(&mut self, dependency: DependencyId) {
    self.dependencies.push(dependency)
  }

  fn remove_dependency_id(&mut self, dependency: DependencyId) {
    self.dependencies.retain(|d| d != &dependency)
  }

  fn get_dependencies(&self) -> &[DependencyId] {
    &self.dependencies
  }
}

#[cacheable_dyn]
#[async_trait]
impl Module for RscEntryModule {
  impl_module_meta_info!();

  fn size(&self, _source_type: Option<&SourceType>, _compilation: Option<&Compilation>) -> f64 {
    42.0
  }

  fn module_type(&self) -> &ModuleType {
    &ModuleType::JsDynamic
  }

  fn source_types(&self, _module_graph: &ModuleGraph) -> &[SourceType] {
    &[SourceType::JavaScript]
  }

  fn source(&self) -> Option<&BoxSource> {
    None
  }

  fn readable_identifier(&self, _context: &Context) -> Cow<'_, str> {
    format!("rsc client entry {}", self.name).into()
  }

  fn lib_ident(&self, _options: LibIdentOptions) -> Option<Cow<'_, str>> {
    Some(self.lib_ident.as_str().into())
  }

  fn get_layer(&self) -> Option<&ModuleLayer> {
    self.layer.as_ref()
  }

  async fn build(
    mut self: Box<Self>,
    _build_context: BuildContext,
    _: Option<&Compilation>,
  ) -> Result<BuildResult> {
    if self.is_server_side_rendering {
      // Eager: no code-split points; use ImportEagerDependency (CSS filtering done at call site).
      let all_client_modules = self.all_client_modules();
      let mut dependencies: Vec<BoxDependency> = Vec::with_capacity(all_client_modules.len());
      for client_module in all_client_modules {
        let referenced_specifiers = create_referenced_specifiers(&client_module.ids);
        let mut dep = ImportEagerDependency::new(
          Atom::from(client_module.request.as_str()),
          DependencyRange { start: 0, end: 0 },
          None,
          ImportPhase::Evaluation,
        );
        if let Some(referenced_specifiers) = referenced_specifiers {
          dep.set_referenced_specifiers(referenced_specifiers, true);
        }
        dependencies.push(Box::new(dep));
      }
      Ok(BuildResult {
        module: BoxModule::new(self),
        dependencies,
        blocks: vec![],
        optimization_bailouts: vec![],
      })
    } else {
      // Non-eager: code-split points; use AsyncDependenciesBlock + ClientReferenceDependency.
      let mut blocks = Vec::with_capacity(
        self.client_modules.len()
          + self.css_imports_by_server_entry.len()
          + self.client_modules_by_server_entry.len()
          + usize::from(!self.root_client_modules.is_empty()),
      );
      let dependencies: Vec<BoxDependency> = vec![];

      let mut server_entries = self
        .css_imports_by_server_entry
        .keys()
        .cloned()
        .collect::<Vec<_>>();
      for server_entry in self.client_modules_by_server_entry.keys() {
        if !server_entries.contains(server_entry) {
          server_entries.push(server_entry.clone());
        }
      }
      server_entries.sort_unstable();

      for server_entry in server_entries {
        let mut block_dependencies: Vec<BoxDependency> = Vec::new();

        if let Some(css_imports) = self.css_imports_by_server_entry.get(&server_entry) {
          block_dependencies.extend(css_imports.iter().map(|request| {
            Box::new(ClientReferenceDependency::new(
              request.clone(),
              Default::default(),
              self.is_server_side_rendering,
            )) as Box<dyn Dependency>
          }));
        }

        if let Some(client_modules) = self.client_modules_by_server_entry.get(&server_entry) {
          block_dependencies.extend(client_modules.iter().map(|client_module| {
            Box::new(ClientReferenceDependency::new(
              client_module.request.clone(),
              client_module.ids.clone(),
              self.is_server_side_rendering,
            )) as Box<dyn Dependency>
          }));
        }

        if block_dependencies.is_empty() {
          continue;
        }

        let block_modifier = format!("server-entry={server_entry}");
        let block = AsyncDependenciesBlock::new(
          self.identifier,
          None,
          Some(&block_modifier),
          block_dependencies,
          Some(server_entry.clone()),
        );
        blocks.push(Box::new(block));
      }

      if !self.root_client_modules.is_empty() {
        let dependencies = self
          .root_client_modules
          .iter()
          .map(|client_module| {
            Box::new(ClientReferenceDependency::new(
              client_module.request.clone(),
              client_module.ids.clone(),
              self.is_server_side_rendering,
            )) as Box<dyn Dependency>
          })
          .collect::<Vec<_>>();

        let block = AsyncDependenciesBlock::new(
          self.identifier,
          None,
          None,
          dependencies,
          Some(format!("{}#root-client", self.name)),
        );
        blocks.push(Box::new(block));
      }

      for client_module in &self.client_modules {
        let dep = ClientReferenceDependency::new(
          client_module.request.clone(),
          client_module.ids.clone(),
          self.is_server_side_rendering,
        );
        let block = AsyncDependenciesBlock::new(
          self.identifier,
          None,
          None,
          vec![Box::new(dep) as Box<dyn Dependency>],
          Some(client_module.request.clone()),
        );
        blocks.push(Box::new(block));
      }

      Ok(BuildResult {
        module: BoxModule::new(self),
        dependencies,
        blocks,
        optimization_bailouts: vec![],
      })
    }
  }

  // RscEntryModule is the bridge injected by the Server Compiler into the
  // Client Compiler to connect Client Component and CSS module graphs.
  // It never emits runtime code; code generation only writes debug comments to
  // help diagnose RSC entry composition issues.
  async fn code_generation(
    &self,
    code_generation_context: &mut ModuleCodeGenerationContext,
  ) -> Result<CodeGenerationResult> {
    let compilation = code_generation_context.compilation;
    let source = self.render_debug_comments(compilation);

    Ok(CodeGenerationResult::default().with_javascript(RawStringSource::from(source).boxed()))
  }

  async fn get_runtime_hash(
    &self,
    compilation: &Compilation,
    runtime: Option<&RuntimeSpec>,
  ) -> Result<RspackHashDigest> {
    let mut hasher = RspackHash::from(&compilation.options.output);
    module_update_hash(self, &mut hasher, compilation, runtime);
    Ok(hasher.digest(&compilation.options.output.hash_digest))
  }
}

impl_empty_diagnosable_trait!(RscEntryModule);

fn create_identifier(
  name: &str,
  client_modules: &[ClientModuleImport],
  root_client_modules: &[ClientModuleImport],
  client_modules_by_server_entry: &ClientModulesByServerEntry,
  css_imports_by_server_entry: &CssImportsByServerEntry,
  is_server_side_rendering: bool,
) -> ModuleIdentifier {
  let mut identifier = String::from("rsc entry|");
  push_value(&mut identifier, name);
  identifier.push('|');
  identifier.push(if is_server_side_rendering { '1' } else { '0' });
  identifier.push('|');

  identifier.push_str("ungrouped[");
  push_client_modules(&mut identifier, client_modules);
  identifier.push_str("]|root[");
  push_client_modules(&mut identifier, root_client_modules);
  identifier.push_str("]|server[");
  let mut client_modules_by_server_entry =
    client_modules_by_server_entry.iter().collect::<Vec<_>>();
  client_modules_by_server_entry.sort_unstable_by_key(|(server_entry, _)| *server_entry);
  for (server_entry, client_modules) in client_modules_by_server_entry {
    push_value(&mut identifier, server_entry);
    identifier.push('[');
    push_client_modules(&mut identifier, client_modules);
    identifier.push(']');
  }
  identifier.push(']');

  identifier.push('|');
  let mut css_imports_by_server_entry = css_imports_by_server_entry.iter().collect::<Vec<_>>();
  css_imports_by_server_entry.sort_unstable_by_key(|(a, _)| *a);
  for (server_entry, css_imports) in css_imports_by_server_entry {
    push_value(&mut identifier, server_entry);
    identifier.push('[');

    let css_imports = sorted_strs(css_imports.iter().map(String::as_str));
    for css_import in css_imports {
      push_value(&mut identifier, css_import);
    }
    identifier.push(']');
  }

  ModuleIdentifier::from(identifier)
}

fn push_client_modules(identifier: &mut String, client_modules: &[ClientModuleImport]) {
  let mut client_modules = client_modules.iter().collect::<Vec<_>>();
  client_modules.sort_unstable_by(|a, b| a.request.cmp(&b.request));
  for client_module in client_modules {
    push_value(identifier, &client_module.request);
    identifier.push('[');

    let ids = sorted_strs(client_module.ids.iter().map(|id| id.as_str()));
    for id in ids {
      push_value(identifier, id);
    }
    identifier.push(']');
  }
}

fn sorted_strs<'a>(values: impl Iterator<Item = &'a str>) -> Vec<&'a str> {
  let mut values = values.collect::<Vec<_>>();
  values.sort_unstable();
  values
}

fn push_value(identifier: &mut String, value: &str) {
  write!(identifier, "{}:", value.len())
    .expect("writing RSC entry module identifier should not fail");
  identifier.push_str(value);
}

fn create_referenced_specifiers(ids: &FxIndexSet<Atom>) -> Option<Vec<ReferencedSpecifier>> {
  if ids.is_empty() || ids.iter().any(|id| id == "*") {
    return None;
  }

  Some(
    ids
      .iter()
      .map(|id| ReferencedSpecifier::new(vec![Atom::from(id.as_str())]))
      .collect(),
  )
}

fn append_client_modules_debug_section(
  source: &mut String,
  compilation: &Compilation,
  chunk_group: &str,
  chunking: &str,
  server_entry: Option<&str>,
  client_modules: &[ClientModuleImport],
) {
  if client_modules.is_empty() {
    return;
  }

  let mut client_modules = client_modules.iter().collect::<Vec<_>>();
  client_modules.sort_unstable_by(|a, b| a.request.cmp(&b.request));
  append_debug_section_header(
    source,
    chunk_group,
    chunking,
    server_entry
      .map(|server_entry| contextify(compilation.options.context.as_path(), server_entry)),
  );
  for client_module in client_modules {
    let request = contextify(
      compilation.options.context.as_path(),
      client_module.request.as_str(),
    );
    append_debug_module_line(source, &request, &format_referenced_exports(client_module));
  }
  append_debug_section_end(source);
}

fn append_server_entry_debug_section(
  source: &mut String,
  compilation: &Compilation,
  server_entry: &str,
  chunking: &str,
  css_imports: Option<&FxIndexSet<String>>,
  client_modules: Option<&Vec<ClientModuleImport>>,
) {
  if css_imports.is_none_or(|css_imports| css_imports.is_empty())
    && client_modules.is_none_or(|client_modules| client_modules.is_empty())
  {
    return;
  }

  append_debug_section_header(
    source,
    "server-entry",
    chunking,
    Some(contextify(
      compilation.options.context.as_path(),
      server_entry,
    )),
  );

  if let Some(css_imports) = css_imports {
    for css_import in sorted_strs(css_imports.iter().map(String::as_str)) {
      let request = contextify(compilation.options.context.as_path(), css_import);
      append_debug_module_line(source, &request, "side-effect");
    }
  }

  if let Some(client_modules) = client_modules {
    let mut client_modules = client_modules.iter().collect::<Vec<_>>();
    client_modules.sort_unstable_by(|a, b| a.request.cmp(&b.request));
    for client_module in client_modules {
      let request = contextify(
        compilation.options.context.as_path(),
        client_module.request.as_str(),
      );
      append_debug_module_line(source, &request, &format_referenced_exports(client_module));
    }
  }

  append_debug_section_end(source);
}

fn append_debug_section_header(
  source: &mut String,
  chunk_group: &str,
  chunking: &str,
  server_entry: Option<String>,
) {
  if !source.is_empty() {
    source.push('\n');
  }

  let chunk_group = sanitize_comment_part(chunk_group);
  let chunking = sanitize_comment_part(chunking);
  write!(
    source,
    "/*!\n * chunk group: {chunk_group}\n * chunking: {chunking}\n"
  )
  .expect("writing debug comments to String should not fail");

  if let Some(server_entry) = server_entry {
    let server_entry = sanitize_comment_part(&server_entry);
    writeln!(source, " * server-entry: {server_entry}")
      .expect("writing debug comments to String should not fail");
  }

  source.push_str(" * modules:\n");
}

fn append_debug_module_line(source: &mut String, request: &str, exports: &str) {
  let request = sanitize_comment_part(request);
  let exports = sanitize_comment_part(exports);
  writeln!(source, " * - {request} (exports: {exports})")
    .expect("writing debug comments to String should not fail");
}

fn append_debug_section_end(source: &mut String) {
  source.push_str(" */");
}

fn sanitize_comment_part(value: &str) -> Cow<'_, str> {
  if value.contains("*/") {
    value.cow_replace("*/", "* /")
  } else {
    Cow::Borrowed(value)
  }
}

fn format_referenced_exports(client_module: &ClientModuleImport) -> String {
  if client_module.ids.is_empty() {
    return "side-effect".to_string();
  }

  if client_module.ids.iter().any(|id| id == "*") {
    return "*".to_string();
  }

  let mut exports = String::new();
  for id in &client_module.ids {
    if !exports.is_empty() {
      exports.push_str(", ");
    }
    exports.push_str(id.as_str());
  }
  exports
}
