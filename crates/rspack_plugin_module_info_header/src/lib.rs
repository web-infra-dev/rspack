use std::hash::Hasher;

use once_cell::sync::Lazy;
use regex::Regex;
use rspack_core::{
  to_comment, ChunkHashArgs, ExportInfoId, ExportsInfo, Module, ModuleGraph, Plugin,
  PluginChunkHashHookOutput, PluginContext, RenderModulePackageContext, UsageState,
};
use rspack_error::Result;
use rspack_sources::{ConcatSource, RawSource};
use rustc_hash::FxHashSet as HashSet;

static COMMENT_END_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\*/").expect("Failed to initialize COMMENT_END_REGEX"));

pub fn print_exports_info_to_source(
  source: &mut ConcatSource,
  indent: &str,
  exports_info: &ExportsInfo,
  module_graph: &ModuleGraph,
  already_printed: &mut HashSet<ExportInfoId>,
) {
  let other_exports_info = &exports_info.other_exports_info;

  let mut already_printed_exports = 0;

  // determine exports to print
  let mut printed_exports = Vec::new();
  for export_info_id in exports_info.get_ordered_exports() {
    if !already_printed.contains(export_info_id) {
      already_printed.insert(*export_info_id);
      printed_exports.push(export_info_id);
    } else {
      already_printed_exports += 1;
    }
  }

  let mut show_other_exports = false;
  if !already_printed.contains(other_exports_info) {
    already_printed.insert(other_exports_info.clone());
    show_other_exports = true;
  } else {
    already_printed_exports += 1;
  }

  // print the exports
  for export_info_id in &printed_exports {
    let mut export_details = String::new();

    let export_info = module_graph.get_export_info_by_id(export_info_id);
    export_details.push_str(indent);
    export_details.push_str("export ");
    if let Some(name) = &export_info.name {
      export_details.push_str(name.as_str());
    }
    export_details.push_str(" [");
    export_details.push_str(&export_info.get_provided_info());
    export_details.push(']');

    export_details.push_str(" [");
    export_details.push_str(&export_info.get_used_info());
    export_details.push(']');

    export_details.push_str(" [");
    export_details.push_str(&export_info.get_rename_info());
    export_details.push(']');

    // TODO: print target

    export_details.push('\n');
    source.add(RawSource::from(to_comment(&export_details)));

    if let Some(exports_info_id) = &export_info.exports_info {
      let exports_info = module_graph.get_exports_info_by_id(exports_info_id);
      print_exports_info_to_source(
        source,
        &format!("{}  ", indent),
        exports_info,
        module_graph,
        already_printed,
      )
    }
  }

  if already_printed_exports != 0 {
    source.add(RawSource::from(to_comment(&format!(
      "{}... ({} already listed exports)\n",
      indent, already_printed_exports
    ))));
  }

  if show_other_exports {
    if !matches!(
      other_exports_info.get_used(module_graph, None),
      UsageState::Unused
    ) {
      let mut other_exports_details = String::new();
      other_exports_details.push_str(indent);
      let title = if printed_exports.len() > 0 || already_printed_exports > 0 {
        "other exports".to_string()
      } else {
        "exports".to_string()
      };
      other_exports_details.push_str(&title);

      other_exports_details.push_str(" [");
      other_exports_details.push_str(&other_exports_info.get_provided_info(module_graph));
      other_exports_details.push(']');

      other_exports_details.push_str(" [");
      other_exports_details.push_str(&other_exports_info.get_used_info(module_graph));
      other_exports_details.push(']');

      other_exports_details.push('\n');
      source.add(RawSource::from(to_comment(&other_exports_details)));
    }
  }
}

#[derive(Debug)]
pub struct ModuleInfoHeaderPlugin {
  verbose: bool,
}

impl ModuleInfoHeaderPlugin {
  pub fn new(verbose: bool) -> Self {
    ModuleInfoHeaderPlugin { verbose }
  }
}

#[async_trait::async_trait]
impl Plugin for ModuleInfoHeaderPlugin {
  fn name(&self) -> &'static str {
    "ModuleInfoHeaderPlugin"
  }

  fn render_module_package(
    &self,
    module_source: ConcatSource,
    module: &dyn Module,
    args: &RenderModulePackageContext,
  ) -> Result<ConcatSource> {
    let RenderModulePackageContext {
      chunk,
      context,
      module_graph,
      chunk_graph,
    } = args;

    let mut source = ConcatSource::default();
    let req = module.readable_identifier(context);
    let req_str = COMMENT_END_REGEX.replace_all(&req, "*_/");
    let req_str_star = "*".repeat(req_str.len());
    let header_str = format!(
      "/*!****{}****!*\\\n  !*** {} ***!\n  \\****{}****/\n",
      req_str_star, req_str, req_str_star
    );
    let header = RawSource::from(header_str);
    source.add(header);

    if self.verbose {
      let exports_type = match module.build_meta() {
        Some(build_meta) => Some(build_meta.exports_type),
        None => None,
      };
      let exports_type_string = if let Some(exports_type) = exports_type {
        format!("{} exports\n", exports_type)
      } else {
        "unknown exports (runtime-defined)\n".to_string()
      };
      let export_types_comment = RawSource::from(to_comment(&exports_type_string));
      source.add(export_types_comment);
      if exports_type.is_some() {
        let exports_info = module_graph.get_exports_info(&module.identifier());
        print_exports_info_to_source(
          &mut source,
          "",
          exports_info,
          module_graph,
          &mut HashSet::default(),
        )
      }
      let runtime_requirements_details = chunk_graph
        .get_module_runtime_requirements(module.identifier(), &chunk.runtime)
        .map(|runtime_requirements| {
          runtime_requirements
            .iter()
            .map(|runtime_requirement| runtime_requirement.name())
            .collect::<Vec<_>>()
            .join(", ")
        })
        .unwrap_or_default();
      source.add(RawSource::from(to_comment(&format!(
        "runtime requirements: {}\n",
        runtime_requirements_details
      ))));
      let optimization_bailout = module_graph.get_optimization_bailout(&module.identifier());
      for text in optimization_bailout {
        source.add(RawSource::from(to_comment(&format!("{}\n", text))));
      }
      source.add(module_source);
      return Ok(source);
    }

    source.add(module_source);
    // TODO: wrap with CachedSource
    // let cached_source = CachedSource::new(source);
    // Ok(cached_source)
    Ok(source)
  }

  async fn chunk_hash(
    &self,
    _ctx: PluginContext,
    args: &mut ChunkHashArgs<'_>,
  ) -> PluginChunkHashHookOutput {
    let hasher = &mut args.hasher;
    hasher.write(&"ModuleInfoHeaderPlugin".as_bytes());
    hasher.write("1".as_bytes());
    Ok(())
  }
}
