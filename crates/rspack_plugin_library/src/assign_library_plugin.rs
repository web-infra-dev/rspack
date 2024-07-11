use std::hash::Hash;

use once_cell::sync::Lazy;
use regex::Regex;
use rspack_core::rspack_sources::SourceExt;
use rspack_core::{
  get_entry_runtime, property_access, ApplyContext, BoxModule, ChunkUkey,
  CodeGenerationDataTopLevelDeclarations, CompilationFinishModules, CompilationParams,
  CompilerCompilation, CompilerOptions, EntryData, FilenameTemplate, LibraryExport, LibraryName,
  LibraryNonUmdObject, ModuleIdentifier, UsageState,
};
use rspack_core::{
  rspack_sources::{ConcatSource, RawSource},
  to_identifier, Chunk, Compilation, LibraryOptions, PathData, Plugin, PluginContext, SourceType,
};
use rspack_error::{error, error_bail, Result};
use rspack_hash::RspackHash;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::{
  JavascriptModulesChunkHash, JavascriptModulesEmbedInRuntimeBailout, JavascriptModulesRender,
  JavascriptModulesRenderStartup, JavascriptModulesStrictRuntimeBailout, JsPlugin, RenderSource,
};
use rspack_util::infallible::ResultInfallibleExt as _;

use crate::utils::{get_options_for_chunk, COMMON_LIBRARY_NAME_MESSAGE};

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
        Some(LibraryName::NonUmdObject(LibraryNonUmdObject::Array(_)))
          | Some(LibraryName::NonUmdObject(LibraryNonUmdObject::String(_)))
      ) {
        error_bail!("Library name must be a string or string array. {COMMON_LIBRARY_NAME_MESSAGE}")
      }
    } else if let Some(name) = &library.name
      && !matches!(
        name,
        LibraryName::NonUmdObject(LibraryNonUmdObject::Array(_))
          | LibraryName::NonUmdObject(LibraryNonUmdObject::String(_))
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
    chunk_ukey: &'a ChunkUkey,
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

  fn get_resolved_full_name(
    &self,
    options: &AssignLibraryPluginParsed,
    compilation: &Compilation,
    chunk: &Chunk,
  ) -> Vec<String> {
    if let Some(name) = options.name {
      let mut prefix = self.options.prefix.value(compilation);
      let get_path = |v: &str| {
        compilation
          .get_path(
            &FilenameTemplate::from(v.to_owned()),
            PathData::default().chunk(chunk).content_hash_optional(
              chunk
                .content_hash
                .get(&SourceType::JavaScript)
                .map(|i| i.rendered(compilation.options.output.hash_digest_length)),
            ),
          )
          .always_ok()
      };
      match name {
        LibraryNonUmdObject::Array(arr) => {
          prefix.extend(arr.iter().map(|s| get_path(s)).collect::<Vec<_>>());
        }
        LibraryNonUmdObject::String(s) => prefix.push(get_path(s)),
      };
      return prefix;
    }
    self.options.prefix.value(compilation)
  }
}

#[plugin_hook(CompilerCompilation for AssignLibraryPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  let mut hooks = JsPlugin::get_compilation_hooks_mut(compilation);
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
fn render(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  render_source: &mut RenderSource,
) -> Result<()> {
  let Some(options) = self.get_options_for_chunk(compilation, chunk_ukey)? else {
    return Ok(());
  };
  if self.options.declare {
    let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
    let base = &self.get_resolved_full_name(&options, compilation, chunk)[0];
    if !is_name_valid(base) {
      let base_identifier = to_identifier(base);
      return Err(
        error!("Library name base ({base}) must be a valid identifier when using a var declaring library type. Either use a valid identifier (e. g. {base_identifier}) or use a different library type (e. g. `type: 'global'`, which assign a property on the global scope instead of declaring a variable). {COMMON_LIBRARY_NAME_MESSAGE}"),
      );
    }
    let mut source = ConcatSource::default();
    source.add(RawSource::from(format!("var {base};\n")));
    source.add(render_source.source.clone());
    render_source.source = source.boxed();
    return Ok(());
  }
  Ok(())
}

#[plugin_hook(JavascriptModulesRenderStartup for AssignLibraryPlugin)]
fn render_startup(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  _module: &ModuleIdentifier,
  render_source: &mut RenderSource,
) -> Result<()> {
  let Some(options) = self.get_options_for_chunk(compilation, chunk_ukey)? else {
    return Ok(());
  };
  let mut source = ConcatSource::default();
  source.add(render_source.source.clone());
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
  let full_name_resolved = self.get_resolved_full_name(&options, compilation, chunk);
  let export_access = options
    .export
    .map(|e| property_access(e, 0))
    .unwrap_or_default();
  if matches!(self.options.unnamed, Unnamed::Static) {
    let export_target = access_with_init(&full_name_resolved, self.options.prefix.len(), true);
    source.add(RawSource::from(format!(
      "Object.defineProperty({export_target}, '__esModule', {{ value: true }});\n",
    )));
  } else if self.is_copy(&options) {
    source.add(RawSource::from(format!(
      "var __webpack_export_target__ = {};\n",
      access_with_init(&full_name_resolved, self.options.prefix.len(), true)
    )));
    let mut exports = "__webpack_exports__";
    if !export_access.is_empty() {
      source.add(RawSource::from(format!(
        "var __webpack_exports_export__ = __webpack_exports__{export_access};\n"
      )));
      exports = "__webpack_exports_export__";
    }
    source.add(RawSource::from(format!(
      "for(var i in {exports}) __webpack_export_target__[i] = {exports}[i];\n"
    )));
    source.add(RawSource::from(format!(
      "if({exports}.__esModule) Object.defineProperty(__webpack_export_target__, '__esModule', {{ value: true }});\n"
    )));
  } else {
    source.add(RawSource::from(format!(
      "{} = __webpack_exports__{export_access};\n",
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
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
  let full_resolved_name = self.get_resolved_full_name(&options, compilation, chunk);
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
fn embed_in_runtime_bailout(
  &self,
  compilation: &Compilation,
  module: &BoxModule,
  chunk: &Chunk,
) -> Result<Option<String>> {
  let Some(options) = self.get_options_for_chunk(compilation, &chunk.ukey)? else {
    return Ok(None);
  };
  let codegen = compilation
    .code_generation_results
    .get(&module.identifier(), Some(&chunk.runtime));
  let top_level_decls = codegen
    .data
    .get::<CodeGenerationDataTopLevelDeclarations>()
    .map(|d| d.inner())
    .or_else(|| {
      module
        .build_info()
        .and_then(|build_info| build_info.top_level_declarations.as_ref())
    });
  if let Some(top_level_decls) = top_level_decls {
    let full_name = self.get_resolved_full_name(&options, compilation, chunk);
    if let Some(base) = full_name.first()
      && top_level_decls.contains(&base.as_str().into())
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
fn strict_runtime_bailout(
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
async fn finish_modules(&self, compilation: &mut Compilation) -> Result<()> {
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
    if let Some(export) = export {
      let exports_info = compilation
        .get_module_graph_mut()
        .get_export_info(module_identifier, &(export.as_str()).into());
      exports_info.set_used(
        &mut compilation.get_module_graph_mut(),
        UsageState::Used,
        Some(&runtime),
      );
    } else {
      let exports_info_id = compilation
        .get_module_graph()
        .get_exports_info(&module_identifier)
        .id;
      exports_info_id
        .set_used_in_unknown_way(&mut compilation.get_module_graph_mut(), Some(&runtime));
    }
  }
  Ok(())
}

#[async_trait::async_trait]
impl Plugin for AssignLibraryPlugin {
  fn name(&self) -> &'static str {
    PLUGIN_NAME
  }

  fn apply(
    &self,
    ctx: PluginContext<&mut ApplyContext>,
    _options: &mut CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compiler_hooks
      .compilation
      .tap(compilation::new(self));
    ctx
      .context
      .compilation_hooks
      .finish_modules
      .tap(finish_modules::new(self));
    Ok(())
  }
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
      property_access(&vec![&accessor[i]], 0),
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

static KEYWORD_REGEXP: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"^(await|break|case|catch|class|const|continue|debugger|default|delete|do|else|enum|export|extends|false|finally|for|function|if|implements|import|in|instanceof|interface|let|new|null|package|private|protected|public|return|super|switch|static|this|throw|try|true|typeof|var|void|while|with|yield)$").expect("should init regex")
});

static IDENTIFIER_REGEXP: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"^[\p{L}\p{Nl}$_][\p{L}\p{Nl}$\p{Mn}\p{Mc}\p{Nd}\p{Pc}]*$")
    .expect("should init regex")
});

#[inline]
fn is_name_valid(v: &str) -> bool {
  !KEYWORD_REGEXP.is_match(v) && IDENTIFIER_REGEXP.is_match(v)
}
