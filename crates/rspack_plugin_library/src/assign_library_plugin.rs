use std::{hash::Hash, sync::LazyLock};

use futures::future::join_all;
use regex::Regex;
use rspack_collections::DatabaseItem;
use rspack_core::{
  AsyncModulesArtifact, BoxModule, CanInlineUse, Chunk, ChunkUkey,
  CodeGenerationDataTopLevelDeclarations, Compilation,
  CompilationAdditionalChunkRuntimeRequirements, CompilationFinishModules, CompilationParams,
  CompilerCompilation, EntryData, ExportProvided, Filename, LibraryExport, LibraryName,
  LibraryNonUmdObject, LibraryOptions, ModuleIdentifier, PathData, Plugin, PrefetchExportsInfoMode,
  RuntimeGlobals, RuntimeModule, RuntimeVariable, SourceType, UsageState, get_entry_runtime,
  property_access,
  rspack_sources::{ConcatSource, RawStringSource, SourceExt},
  to_identifier,
};
use rspack_error::{Result, ToStringResultToRspackResultExt, error, error_bail};
use rspack_hash::RspackHash;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::{
  JavascriptModulesChunkHash, JavascriptModulesEmbedInRuntimeBailout, JavascriptModulesRender,
  JavascriptModulesRenderStartup, JavascriptModulesStrictRuntimeBailout, JsPlugin, RenderSource,
};
use swc_core::atoms::Atom;

use crate::utils::{COMMON_LIBRARY_NAME_MESSAGE, get_options_for_chunk};

const PLUGIN_NAME: &str = "rspack.AssignLibraryPlugin";

#[derive(Debug)]
pub enum Unnamed {
  Error,
  Static,
  Copy,
  Assign,
}

#[derive(Debug)]
pub enum Named {
  Copy,
  Assign,
}

#[derive(Debug)]
pub enum Prefix {
  Global,
  Array(Vec<String>),
}

impl Prefix {
  pub fn value(&self, compilation: &Compilation) -> Vec<String> {
    match self {
      Prefix::Global => vec![compilation.options.output.global_object.clone()],
      Prefix::Array(v) => v.clone(),
    }
  }

  pub fn len(&self) -> usize {
    match self {
      Prefix::Global => 1,
      Prefix::Array(v) => v.len(),
    }
  }

  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }
}

#[derive(Debug)]
pub struct AssignLibraryPluginOptions {
  pub library_type: String,
  pub prefix: Prefix,
  pub declare: bool,
  pub unnamed: Unnamed,
  pub named: Option<Named>,
}

#[derive(Debug)]
struct AssignLibraryPluginParsed<'a> {
  name: Option<&'a LibraryNonUmdObject>,
  export: Option<&'a LibraryExport>,
}

#[plugin]
#[derive(Debug)]
pub struct AssignLibraryPlugin {
  options: AssignLibraryPluginOptions,
}

impl AssignLibraryPlugin {
  pub fn new(options: AssignLibraryPluginOptions) -> Self {
    Self::new_inner(options)
  }

  fn parse_options<'a>(
    &self,
    library: &'a LibraryOptions,
  ) -> Result<AssignLibraryPluginParsed<'a>> {
    if matches!(self.options.unnamed, Unnamed::Error) {
      if !matches!(
        library.name,
        Some(LibraryName::NonUmdObject(
          LibraryNonUmdObject::Array(_) | LibraryNonUmdObject::String(_)
        ))
      ) {
        error_bail!("Library name must be a string or string array. {COMMON_LIBRARY_NAME_MESSAGE}")
      }
    } else if let Some(name) = &library.name
      && !matches!(
        name,
        LibraryName::NonUmdObject(LibraryNonUmdObject::Array(_) | LibraryNonUmdObject::String(_))
      )
    {
      error_bail!(
        "Library name must be a string, string array or unset. {COMMON_LIBRARY_NAME_MESSAGE}"
      )
    }
    Ok(AssignLibraryPluginParsed {
      name: library.name.as_ref().map(|n| match n {
        LibraryName::NonUmdObject(n) => n,
        _ => unreachable!("Library name must be a string, string array or unset."),
      }),
      export: library.export.as_ref(),
    })
  }

  fn get_options_for_chunk<'a>(
    &self,
    compilation: &'a Compilation,
    chunk_ukey: &ChunkUkey,
  ) -> Result<Option<AssignLibraryPluginParsed<'a>>> {
    get_options_for_chunk(compilation, chunk_ukey)
      .filter(|library| library.library_type == self.options.library_type)
      .map(|library| self.parse_options(library))
      .transpose()
  }

  fn is_copy(&self, options: &AssignLibraryPluginParsed) -> bool {
    if options.name.is_some() {
      matches!(self.options.named, Some(Named::Copy))
    } else {
      matches!(self.options.unnamed, Unnamed::Copy)
    }
  }

  async fn get_resolved_full_name(
    &self,
    options: &AssignLibraryPluginParsed<'_>,
    compilation: &Compilation,
    chunk: &Chunk,
  ) -> Result<Vec<String>> {
    if let Some(name) = options.name {
      let mut prefix = self.options.prefix.value(compilation);
      let get_path = async |v: &str| {
        compilation
          .get_path(
            &Filename::from(v),
            PathData::default()
              .chunk_id_optional(chunk.id().map(|id| id.as_str()))
              .chunk_hash_optional(chunk.rendered_hash(
                &compilation.chunk_hashes_artifact,
                compilation.options.output.hash_digest_length,
              ))
              .chunk_name_optional(chunk.name_for_filename_template())
              .content_hash_optional(chunk.rendered_content_hash_by_source_type(
                &compilation.chunk_hashes_artifact,
                &SourceType::JavaScript,
                compilation.options.output.hash_digest_length,
              )),
          )
          .await
      };
      match name {
        LibraryNonUmdObject::Array(arr) => {
          let paths = join_all(arr.iter().map(|s| get_path(s)))
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()?;
          prefix.extend(paths);
        }
        LibraryNonUmdObject::String(s) => prefix.push(get_path(s).await?),
      };
      Ok(prefix)
    } else {
      Ok(self.options.prefix.value(compilation))
    }
  }
}

#[plugin_hook(CompilerCompilation for AssignLibraryPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  let hooks = JsPlugin::get_compilation_hooks_mut(compilation.id());
  let mut hooks = hooks.write().await;
  hooks.render.tap(render::new(self));
  hooks.render_startup.tap(render_startup::new(self));
  hooks.chunk_hash.tap(js_chunk_hash::new(self));
  hooks
    .embed_in_runtime_bailout
    .tap(embed_in_runtime_bailout::new(self));
  hooks
    .strict_runtime_bailout
    .tap(strict_runtime_bailout::new(self));
  Ok(())
}

#[plugin_hook(JavascriptModulesRender for AssignLibraryPlugin)]
async fn render(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  render_source: &mut RenderSource,
) -> Result<()> {
  let Some(options) = self.get_options_for_chunk(compilation, chunk_ukey)? else {
    return Ok(());
  };
  if self.options.declare {
    let chunk = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .expect_get(chunk_ukey);
    let base = &self
      .get_resolved_full_name(&options, compilation, chunk)
      .await?[0];
    if !is_name_valid(base) {
      let base_identifier = to_identifier(base);
      return Err(error!(
        "Library name base ({base}) must be a valid identifier when using a var declaring library type. Either use a valid identifier (e. g. {base_identifier}) or use a different library type (e. g. `type: 'global'`, which assign a property on the global scope instead of declaring a variable). {COMMON_LIBRARY_NAME_MESSAGE}"
      ));
    }
    let mut source = ConcatSource::default();
    source.add(RawStringSource::from(format!("var {base};\n")));
    source.add(render_source.source.clone());
    render_source.source = source.boxed();
    return Ok(());
  }
  Ok(())
}

#[plugin_hook(JavascriptModulesRenderStartup for AssignLibraryPlugin)]
async fn render_startup(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  module: &ModuleIdentifier,
  render_source: &mut RenderSource,
) -> Result<()> {
  let Some(options) = self.get_options_for_chunk(compilation, chunk_ukey)? else {
    return Ok(());
  };
  let mut source = ConcatSource::default();
  source.add(render_source.source.clone());
  let chunk = compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .expect_get(chunk_ukey);
  let full_name_resolved = self
    .get_resolved_full_name(&options, compilation, chunk)
    .await?;
  let export_access = options
    .export
    .map(|e| property_access(e, 0))
    .unwrap_or_default();
  let exports_name = compilation
    .runtime_template
    .render_runtime_variable(&RuntimeVariable::Exports);
  if matches!(self.options.unnamed, Unnamed::Static) {
    let export_target = access_with_init(&full_name_resolved, self.options.prefix.len(), true);
    let module_graph = compilation.get_module_graph();
    let exports_info =
      module_graph.get_prefetched_exports_info(module, PrefetchExportsInfoMode::Default);
    let mut provided = vec![];
    let exports_name = compilation
      .runtime_template
      .render_runtime_variable(&RuntimeVariable::Exports);
    for (_, export_info) in exports_info.exports() {
      if matches!(export_info.provided(), Some(ExportProvided::NotProvided)) {
        continue;
      }
      let export_info_name = export_info.name().expect("should have name").to_string();
      provided.push(export_info_name.clone());
      let name_access = property_access([export_info_name], 0);
      source.add(RawStringSource::from(format!(
        "{export_target}{name_access} = {exports_name}{export_access}{name_access};\n"
      )));
    }

    let mut exports = exports_name.as_str();
    let exports_assign_export = "__rspack_exports_export";
    if !export_access.is_empty() {
      source.add(RawStringSource::from(format!(
        "var {exports_assign_export} = {exports_name}{export_access};\n"
      )));
      exports = exports_assign_export;
    }
    source.add(RawStringSource::from(format!(
      "for(var __rspack_i in {exports}) {{\n"
    )));
    let has_provided = !provided.is_empty();
    if has_provided {
      source.add(RawStringSource::from(format!(
        "  if({}.indexOf(__rspack_i) === -1) {{\n",
        serde_json::to_string(&provided).to_rspack_result()?
      )));
    }
    source.add(RawStringSource::from(format!(
      "{}  {export_target}[__rspack_i] = {exports}[__rspack_i];\n",
      match has_provided {
        true => "  ",
        false => "",
      }
    )));

    source.add(RawStringSource::from(if has_provided {
      "  }\n}\n"
    } else {
      "}\n"
    }));

    source.add(RawStringSource::from(format!(
      "Object.defineProperty({export_target}, '__esModule', {{ value: true }});\n",
    )));
  } else if self.is_copy(&options) {
    let exports_assign = "__rspack_exports_target";
    source.add(RawStringSource::from(format!(
      "var {exports_assign} = {};\n",
      access_with_init(&full_name_resolved, self.options.prefix.len(), true)
    )));
    let mut exports = exports_name.as_str();
    let exports_assign_export = "__rspack_exports_export";
    if !export_access.is_empty() {
      source.add(RawStringSource::from(format!(
        "var {exports_assign_export} = {exports_name}{export_access};\n"
      )));
      exports = exports_assign_export;
    }
    source.add(RawStringSource::from(format!(
      "for(var __rspack_i in {exports}) {exports_assign}[__rspack_i] = {exports}[__rspack_i];\n"
    )));
    source.add(RawStringSource::from(format!(
      "if({exports}.__esModule) Object.defineProperty({exports_assign}, '__esModule', {{ value: true }});\n"
    )));
  } else {
    source.add(RawStringSource::from(format!(
      "{} = {exports_name}{export_access};\n",
      access_with_init(&full_name_resolved, self.options.prefix.len(), false)
    )));
  }
  render_source.source = source.boxed();
  Ok(())
}

#[plugin_hook(JavascriptModulesChunkHash for AssignLibraryPlugin)]
async fn js_chunk_hash(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  hasher: &mut RspackHash,
) -> Result<()> {
  let Some(options) = self.get_options_for_chunk(compilation, chunk_ukey)? else {
    return Ok(());
  };
  PLUGIN_NAME.hash(hasher);
  let chunk = compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .expect_get(chunk_ukey);
  let full_resolved_name = self
    .get_resolved_full_name(&options, compilation, chunk)
    .await?;
  if self.is_copy(&options) {
    "copy".hash(hasher);
  }
  if self.options.declare {
    self.options.declare.hash(hasher);
  }
  full_resolved_name.join(".").hash(hasher);
  if let Some(export) = options.export {
    export.hash(hasher);
  }
  Ok(())
}

#[plugin_hook(JavascriptModulesEmbedInRuntimeBailout for AssignLibraryPlugin)]
async fn embed_in_runtime_bailout(
  &self,
  compilation: &Compilation,
  module: &BoxModule,
  chunk: &Chunk,
) -> Result<Option<String>> {
  let Some(options) = self.get_options_for_chunk(compilation, &chunk.ukey())? else {
    return Ok(None);
  };
  let codegen = compilation
    .code_generation_results
    .get(&module.identifier(), Some(chunk.runtime()));
  let top_level_decls = codegen
    .data
    .get::<CodeGenerationDataTopLevelDeclarations>()
    .map(|d| d.inner())
    .or_else(|| module.build_info().top_level_declarations.as_ref());
  if let Some(top_level_decls) = top_level_decls {
    let full_name = self
      .get_resolved_full_name(&options, compilation, chunk)
      .await?;
    if let Some(base) = full_name.first()
      && top_level_decls.contains(&Atom::new(base.as_str()))
    {
      return Ok(Some(format!(
        "it declares '{base}' on top-level, which conflicts with the current library output."
      )));
    }
    return Ok(None);
  }
  Ok(Some(
    "it doesn't tell about top level declarations.".to_string(),
  ))
}

#[plugin_hook(JavascriptModulesStrictRuntimeBailout for AssignLibraryPlugin)]
async fn strict_runtime_bailout(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
) -> Result<Option<String>> {
  let Some(options) = self.get_options_for_chunk(compilation, chunk_ukey)? else {
    return Ok(None);
  };
  if self.options.declare
    || matches!(self.options.prefix, Prefix::Global)
    || !self.options.prefix.is_empty()
    || options.name.is_none()
  {
    return Ok(None);
  }
  Ok(Some(
    "a global variable is assign and maybe created".to_string(),
  ))
}

#[plugin_hook(CompilationFinishModules for AssignLibraryPlugin)]
async fn finish_modules(
  &self,
  compilation: &mut Compilation,
  _async_modules_artifact: &mut AsyncModulesArtifact,
) -> Result<()> {
  let mut runtime_info = Vec::with_capacity(compilation.entries.len());
  for (entry_name, entry) in compilation.entries.iter() {
    let EntryData {
      dependencies,
      options,
      ..
    } = entry;
    let runtime = get_entry_runtime(entry_name, options, &compilation.entries);
    let library_options = options
      .library
      .as_ref()
      .or_else(|| compilation.options.output.library.as_ref());
    let module_graph = compilation.get_module_graph();
    let module_of_last_dep = dependencies
      .last()
      .and_then(|dep| module_graph.get_module_by_dependency_id(dep));
    let Some(module_of_last_dep) = module_of_last_dep else {
      continue;
    };
    let Some(library_options) = library_options else {
      continue;
    };
    if let Some(export) = library_options
      .export
      .as_ref()
      .and_then(|item| item.first())
    {
      runtime_info.push((
        runtime,
        Some(export.clone()),
        module_of_last_dep.identifier(),
      ));
    } else {
      runtime_info.push((runtime, None, module_of_last_dep.identifier()));
    }
  }

  for (runtime, export, module_identifier) in runtime_info {
    let module_graph = compilation
      .build_module_graph_artifact
      .get_module_graph_mut();
    if let Some(export) = export {
      let export_info = module_graph
        .get_exports_info_data_mut(&module_identifier)
        .ensure_export_info(&(export.as_str()).into());
      let info = export_info.as_data_mut(module_graph);
      info.set_used(UsageState::Used, Some(&runtime));
      info.set_can_mangle_use(Some(false));
      info.set_can_inline_use(Some(CanInlineUse::No));
    } else {
      module_graph
        .get_exports_info_data_mut(&module_identifier)
        .set_used_in_unknown_way(Some(&runtime));
    }
  }
  Ok(())
}

impl Plugin for AssignLibraryPlugin {
  fn name(&self) -> &'static str {
    PLUGIN_NAME
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));
    ctx
      .compilation_hooks
      .finish_modules
      .tap(finish_modules::new(self));
    ctx
      .compilation_hooks
      .additional_chunk_runtime_requirements
      .tap(additional_chunk_runtime_requirements::new(self));
    Ok(())
  }
}

#[plugin_hook(CompilationAdditionalChunkRuntimeRequirements for AssignLibraryPlugin)]
async fn additional_chunk_runtime_requirements(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &mut RuntimeGlobals,
  _runtime_modules: &mut Vec<Box<dyn RuntimeModule>>,
) -> Result<()> {
  if self
    .get_options_for_chunk(compilation, chunk_ukey)?
    .is_none()
  {
    return Ok(());
  }
  runtime_requirements.insert(RuntimeGlobals::EXPORTS);
  Ok(())
}

fn access_with_init(accessor: &[String], existing_length: usize, init_last: bool) -> String {
  let base = accessor[0].clone();
  if accessor.len() == 1 && !init_last {
    return base;
  }

  let mut current = if existing_length > 0 {
    base.clone()
  } else {
    format!("({base} = typeof {base} === 'undefined' ? {{}} : {base})")
  };
  let mut i = 1;
  let mut props_so_far = vec![];
  if existing_length > i {
    props_so_far = accessor[1..existing_length].to_vec();
    i = existing_length;
    current.push_str(property_access(&props_so_far, 0).as_str());
  }

  let init_until = if init_last {
    accessor.len()
  } else {
    accessor.len() - 1
  };

  while i < init_until {
    props_so_far.push(accessor[i].clone());
    current = format!(
      "({current}{} = {base}{} || {{}})",
      property_access(vec![&accessor[i]], 0),
      property_access(&props_so_far, 0)
    );
    i += 1;
  }

  if i < accessor.len() {
    current = format!(
      "{current}{}",
      property_access([&accessor[accessor.len() - 1]], 0),
    );
  }

  current
}

static KEYWORD_REGEXP: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(r"^(await|break|case|catch|class|const|continue|debugger|default|delete|do|else|enum|export|extends|false|finally|for|function|if|implements|import|in|instanceof|interface|let|new|null|package|private|protected|public|return|super|switch|static|this|throw|try|true|typeof|var|void|while|with|yield)$").expect("should init regex")
});

static IDENTIFIER_REGEXP: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(r"^[\p{L}\p{Nl}$_][\p{L}\p{Nl}$\p{Mn}\p{Mc}\p{Nd}\p{Pc}]*$")
    .expect("should init regex")
});

#[inline]
fn is_name_valid(v: &str) -> bool {
  !KEYWORD_REGEXP.is_match(v) && IDENTIFIER_REGEXP.is_match(v)
}
