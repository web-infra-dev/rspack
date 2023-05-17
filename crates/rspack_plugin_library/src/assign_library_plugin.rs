use std::hash::Hash;

use rspack_core::{
  rspack_sources::{ConcatSource, RawSource, SourceExt},
  Chunk, Compilation, Filename, JsChunkHashArgs, LibraryOptions, PathData, Plugin, PluginContext,
  PluginJsChunkHashHookOutput, PluginRenderHookOutput, PluginRenderStartupHookOutput, RenderArgs,
  RenderStartupArgs, SourceType,
};

use crate::utils::property_access;

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
pub struct AssignLibraryPluginOptions {
  pub library_type: String,
  pub prefix: Vec<String>,
  pub declare: bool,
  pub unnamed: Unnamed,
  pub named: Option<Named>,
}

#[derive(Debug)]
pub struct AssignLibraryPlugin {
  options: AssignLibraryPluginOptions,
}

impl AssignLibraryPlugin {
  pub fn new(options: AssignLibraryPluginOptions) -> Self {
    Self { options }
  }

  pub fn get_resolved_full_name(&self, compilation: &Compilation, chunk: &Chunk) -> Vec<String> {
    if let Some(library) = &compilation.options.output.library {
      if let Some(name) = &library.name {
        if let Some(root) = &name.root {
          let mut prefix = self.options.prefix.clone();
          prefix.extend(
            root
              .iter()
              .map(|v| {
                compilation.get_path(
                  &Filename::from(v.to_owned()),
                  PathData::default().chunk(chunk).content_hash_optional(
                    chunk
                      .content_hash
                      .get(&SourceType::JavaScript)
                      .map(|i| i.as_str()),
                  ),
                )
              })
              .collect::<Vec<_>>(),
          );
          return prefix;
        }
      }
    }
    self.options.prefix.clone()
  }
}

impl Plugin for AssignLibraryPlugin {
  fn name(&self) -> &'static str {
    "AssignLibraryPlugin"
  }

  fn render(&self, _ctx: PluginContext, args: &RenderArgs) -> PluginRenderHookOutput {
    if self.options.declare {
      let base = &self.get_resolved_full_name(args.compilation, args.chunk())[0];
      let mut source = ConcatSource::default();
      source.add(RawSource::from(format!("var {base};\n")));
      source.add(args.source.clone());
      return Ok(Some(source.boxed()));
    }
    Ok(Some(args.source.clone()))
  }

  fn render_startup(
    &self,
    _ctx: PluginContext,
    args: &RenderStartupArgs,
  ) -> PluginRenderStartupHookOutput {
    let mut source = ConcatSource::default();
    source.add(args.source.clone());
    // TODO: respect entryOptions.library
    let library = &args.compilation.options.output.library;
    let is_copy = if let Some(library) = library {
      if library.name.is_some() {
        matches!(self.options.named, Some(Named::Copy))
      } else {
        matches!(self.options.unnamed, Unnamed::Copy)
      }
    } else {
      false
    };
    let full_name_resolved = self.get_resolved_full_name(args.compilation, args.chunk());
    let export_access = property_library(library);
    if matches!(self.options.unnamed, Unnamed::Static) {
      let export_target = access_with_init(&full_name_resolved, self.options.prefix.len(), true);
      source.add(RawSource::from(format!(
        "Object.defineProperty({export_target}, '__esModule', {{ value: true }});\n",
      )));
    } else if is_copy {
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

    Ok(Some(source.boxed()))
  }

  fn js_chunk_hash(
    &self,
    _ctx: PluginContext,
    args: &mut JsChunkHashArgs,
  ) -> PluginJsChunkHashHookOutput {
    self.name().hash(&mut args.hasher);
    args
      .compilation
      .options
      .output
      .library
      .hash(&mut args.hasher);
    Ok(())
  }
}

#[inline]
fn property_library(library: &Option<LibraryOptions>) -> String {
  if let Some(library) = library {
    if let Some(export) = &library.export {
      return property_access(export);
    }
  }
  String::default()
}

fn access_with_init(accessor: &Vec<String>, existing_length: usize, init_last: bool) -> String {
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
    current.push_str(property_access(&props_so_far).as_str());
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
      property_access(&vec![accessor[i].clone()]),
      property_access(&props_so_far)
    );
    i += 1;
  }

  if i < accessor.len() {
    current = format!(
      "{current}{}",
      property_access(&vec![accessor[accessor.len() - 1].clone()]),
    );
  }

  current
}
