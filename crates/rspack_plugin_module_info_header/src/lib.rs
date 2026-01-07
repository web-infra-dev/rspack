use std::{hash::Hash, rc::Rc, sync::LazyLock};

use regex::Regex;
use rspack_cacheable::with::AsVecConverter;
use rspack_core::{
  BuildMetaExportsType, ChunkGraph, ChunkInitFragments, ChunkUkey, Compilation, CompilationParams,
  CompilerCompilation, ExportInfo, ExportProvided, ExportsInfoGetter, GetTargetResult, Module,
  ModuleGraph, ModuleIdentifier, Plugin, PrefetchExportsInfoMode, PrefetchedExportsInfoWrapper,
  UsageState, get_target,
  rspack_sources::{ConcatSource, RawStringSource, SourceExt},
  to_comment_with_nl,
};
use rspack_error::Result;
use rspack_hash::RspackHash;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_css::{
  CssPlugin,
  plugin::{CssModulesRenderModulePackage, CssModulesRenderSource},
};
use rspack_plugin_javascript::{
  JavascriptModulesChunkHash, JavascriptModulesRenderModulePackage, JsPlugin, RenderSource,
};
use rustc_hash::FxHashSet;

static COMMENT_END_REGEX: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"\*/").expect("should init regex"));

#[plugin]
#[derive(Debug, Default)]
pub struct ModuleInfoHeaderPlugin {
  verbose: bool,
}

fn print_exports_info_to_source<F>(
  source: &mut ConcatSource,
  ident: &str,
  exports_info: &PrefetchedExportsInfoWrapper<'_>,
  request_shortener: &F,
  already_printed: &mut FxHashSet<ExportInfo>,
  module_graph: &ModuleGraph,
) where
  F: Fn(&ModuleIdentifier) -> String,
{
  let other_exports_info = exports_info.other_exports_info();

  let mut already_printed_exports = 0;

  let mut printed_exports = vec![];

  for (_, export_info) in exports_info.exports() {
    let export_info_id = export_info.id();
    if !already_printed.contains(&export_info_id) {
      already_printed.insert(export_info_id);
      printed_exports.push(export_info);
    } else {
      already_printed_exports += 1;
    }
  }
  let mut show_other_exports = false;
  let other_exports_info_id = other_exports_info.id();
  if !already_printed.contains(&other_exports_info_id) {
    already_printed.insert(other_exports_info_id);
    show_other_exports = true;
  } else {
    already_printed_exports += 1;
  }

  // print the exports
  for export_info in &printed_exports {
    let export_name: String = export_info
      .name()
      .map(|n| n.to_string())
      .unwrap_or("null".into());
    let provide_info = export_info.get_provided_info();
    let usage_info = export_info.get_used_info();
    let rename_info = export_info.get_rename_info();

    let target_desc = match get_target(
      export_info,
      module_graph,
      Rc::new(|_| true),
      &mut Default::default(),
    ) {
      Some(GetTargetResult::Target(resolve_target)) => {
        let target_module = request_shortener(&resolve_target.module);
        match resolve_target.export {
          None => format!("-> {target_module}"),
          Some(es) => {
            let exp = es.iter().map(|a| a.as_str()).collect::<Vec<_>>().join(".");
            format!(" -> {target_module} {exp}")
          }
        }
      }
      _ => "".into(),
    };

    let export_str = format!(
      r#"{ident}export {export_name} [{provide_info}] [{usage_info}] [{rename_info}]{target_desc}"#,
    );

    source.add(RawStringSource::from(to_comment_with_nl(&export_str)));

    if let Some(exports_info) = &export_info.exports_info() {
      let exports_info =
        ExportsInfoGetter::prefetch(exports_info, module_graph, PrefetchExportsInfoMode::Default);
      print_exports_info_to_source(
        source,
        &format!("{ident}  "),
        &exports_info,
        request_shortener,
        already_printed,
        module_graph,
      );
    }
  }

  if already_printed_exports > 0 {
    source.add(RawStringSource::from(to_comment_with_nl(&format!(
      "{ident}... {already_printed_exports} already listed exports",
    ))));
  }

  if show_other_exports {
    let target = get_target(
      other_exports_info,
      module_graph,
      Rc::new(|_| true),
      &mut Default::default(),
    );
    if matches!(target, Some(GetTargetResult::Target(_)))
      || !matches!(
        other_exports_info.provided(),
        Some(ExportProvided::NotProvided)
      )
      || other_exports_info.get_used(None) != UsageState::Unused
    {
      let title = if !printed_exports.is_empty() || already_printed_exports > 0 {
        "other exports"
      } else {
        "exports"
      };

      let provide_info = other_exports_info.get_provided_info();
      let used_info = other_exports_info.get_used_info();
      let target_desc = match target {
        Some(GetTargetResult::Target(resolve_target)) => {
          format!(" -> {}", request_shortener(&resolve_target.module))
        }
        _ => "".into(),
      };

      let other_export_str =
        format!(r#"{ident}{title} [{provide_info}] [{used_info}]{target_desc}"#,);

      source.add(RawStringSource::from(to_comment_with_nl(&other_export_str)))
    }
  }
}

impl ModuleInfoHeaderPlugin {
  pub fn new(verbose: bool) -> ModuleInfoHeaderPlugin {
    Self::new_inner(verbose)
  }

  pub fn generate_header(module: &dyn Module, compilation: &Compilation) -> String {
    let req = module.readable_identifier(&compilation.options.context);
    let req = COMMENT_END_REGEX.replace_all(&req, "*_/");

    let req_stars_str = "*".repeat(req.len());

    format!(
      r#"
/*!****{req_stars_str}****!*\
  !*** {req} ***!
  \****{req_stars_str}****/
"#
    )
  }
}

#[plugin_hook(CompilerCompilation for ModuleInfoHeaderPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  {
    let js_hooks = JsPlugin::get_compilation_hooks_mut(compilation.id());
    let mut js_hooks = js_hooks.write().await;
    js_hooks
      .render_module_package
      .tap(render_js_module_package::new(self));
    js_hooks.chunk_hash.tap(chunk_hash::new(self));
  }

  let css_hooks = CssPlugin::get_compilation_hooks_mut(compilation.id());
  css_hooks
    .borrow_mut()
    .render_module_package
    .tap(render_css_module_package::new(self));

  Ok(())
}

#[plugin_hook(JavascriptModulesChunkHash for ModuleInfoHeaderPlugin)]
async fn chunk_hash(
  &self,
  _compilation: &Compilation,
  _chunk_ukey: &ChunkUkey,
  hasher: &mut RspackHash,
) -> Result<()> {
  "ModuleInfoHeaderPlugin".hash(hasher);
  "1".hash(hasher);

  Ok(())
}

#[plugin_hook(CssModulesRenderModulePackage for ModuleInfoHeaderPlugin,tracing=false)]
async fn render_css_module_package(
  &self,
  compilation: &Compilation,
  _chunk_key: &ChunkUkey,
  module: &dyn Module,
  render_source: &mut CssModulesRenderSource,
) -> Result<()> {
  let mut new_source: ConcatSource = Default::default();

  new_source.add(RawStringSource::from(
    ModuleInfoHeaderPlugin::generate_header(module, compilation),
  ));

  new_source.add(render_source.source.clone());
  render_source.source = new_source.boxed();

  Ok(())
}

#[plugin_hook(JavascriptModulesRenderModulePackage for ModuleInfoHeaderPlugin,tracing=false)]
async fn render_js_module_package(
  &self,
  compilation: &Compilation,
  chunk_key: &ChunkUkey,
  module: &dyn Module,
  render_source: &mut RenderSource,
  _init_fragments: &mut ChunkInitFragments,
) -> Result<()> {
  let mut new_source: ConcatSource = Default::default();

  new_source.add(RawStringSource::from(
    ModuleInfoHeaderPlugin::generate_header(module, compilation),
  ));

  if self.verbose {
    let export_type = module.build_meta().exports_type;

    new_source.add(RawStringSource::from(to_comment_with_nl(
      &module.build_meta().exports_type.to_string(),
    )));

    let module_graph = compilation.get_module_graph();

    let exports_info = module_graph
      .get_prefetched_exports_info(&module.identifier(), PrefetchExportsInfoMode::Default);

    if !matches!(export_type, BuildMetaExportsType::Unset) {
      let request_shortener = |id: &ModuleIdentifier| {
        module_graph
          .module_by_identifier(id)
          .expect("target module should exists")
          .readable_identifier(&compilation.options.context)
          .to_string()
      };

      print_exports_info_to_source(
        &mut new_source,
        "",
        &exports_info,
        &request_shortener,
        &mut FxHashSet::default(),
        module_graph,
      );
    }

    let chunk = compilation
      .chunk_by_ukey
      .get(chunk_key)
      .expect("Chunk must exists");

    if let Some(runtime_requirements) =
      ChunkGraph::get_module_runtime_requirements(compilation, module.identifier(), chunk.runtime())
    {
      let reqs = {
        let mut rr = runtime_requirements
          .iter()
          .map(|v| compilation.runtime_template.render_runtime_globals(&v))
          .collect::<Vec<_>>();
        rr.sort_by(|a, b| b.cmp(a));
        rr.join(", ")
      };

      new_source.add(RawStringSource::from(to_comment_with_nl(&format!(
        "runtime requirements: {reqs}"
      ))));
    }

    for b in module_graph.get_optimization_bailout(&module.identifier()) {
      new_source.add(RawStringSource::from(to_comment_with_nl(b)))
    }
  }

  new_source.add(render_source.source.clone());

  render_source.source = new_source.boxed();

  Ok(())
}

impl Plugin for ModuleInfoHeaderPlugin {
  fn name(&self) -> &'static str {
    "rspack.ModuleInfoHeaderPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));
    Ok(())
  }
}
