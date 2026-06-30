use std::{borrow::Cow, sync::Arc};

use rspack_error::Result;
use rspack_sources::{
  MapOptions, Mapping, ObjectPool, OriginalLocation, Source, SourceMap, SourceMapSource,
  SourceMapSourceOptions, encode_mappings,
};
use rspack_util::source_map::SourceMapKind;
use rustc_hash::{FxHashMap, FxHashSet};
use swc_core::{
  common::{
    BytePos, FileName, LineCol, SourceFile, SourceMap as SwcSourceMap, comments::Comments,
    source_map::SmallPos, sync::Lrc,
  },
  ecma::{
    ast::{EsVersion, Ident, Program as SwcProgram},
    atoms::Atom,
    codegen::{
      self, Emitter, Node,
      text_writer::{self, WriteJs},
    },
    visit::{Visit, VisitWith, noop_visit_type},
  },
};

use super::{JavaScriptCompiler, TransformOutput};

#[derive(Default, Clone, Debug)]
pub struct SourceMapConfig {
  pub source_map_kind: SourceMapKind,
  pub names: FxHashMap<BytePos, Atom>,
}

impl SourceMapConfig {
  pub fn enabled(&self) -> bool {
    self.source_map_kind.source_map()
  }

  fn file_name_to_source(&self, f: &FileName) -> String {
    let f = f.to_string();
    if f.starts_with('<') && f.ends_with('>') {
      f[1..f.len() - 1].to_string()
    } else {
      f
    }
  }

  fn inline_sources_content(&self, _: &FileName) -> bool {
    // Ideally transform should keep the original source via `original_source`, but
    // NormalModule historically wraps loader output with `WithoutOriginalOptions`.
    // Keep the old behavior of carrying it through SWC's inline source content.
    self.source_map_kind.source_map()
  }

  fn emit_columns(&self, _f: &FileName) -> bool {
    self.source_map_kind.emit_columns()
  }

  fn name_for_bytepos(&self, pos: BytePos) -> Option<&str> {
    self.names.get(&pos).map(|v| &**v)
  }

  fn skip(&self, f: &FileName) -> bool {
    matches!(f, FileName::Internal(..))
  }

  fn ignore_list(&self, f: &FileName) -> bool {
    matches!(f, FileName::Anon | FileName::Internal(..))
  }
}

pub struct PrintOptions<'a> {
  pub source_len: u32,
  pub source_map: Arc<SwcSourceMap>,
  pub target: EsVersion,
  pub source_map_config: SourceMapConfig,
  pub input_source_map: Option<SourceMap<'static>>,
  pub minify: bool,
  pub comments: Option<&'a dyn Comments>,
  pub preamble: &'a str,
  pub ascii_only: bool,
  pub inline_script: bool,
}

impl JavaScriptCompiler {
  pub fn print(&self, node: &SwcProgram, options: PrintOptions<'_>) -> Result<TransformOutput> {
    let PrintOptions {
      source_len,
      source_map,
      target,
      mut source_map_config,
      input_source_map,
      minify,
      comments,
      preamble,
      ascii_only,
      inline_script,
    } = options;
    let mut src_map_buf = vec![];

    if source_map_config.enabled() {
      let mut v = IdentCollector {
        names: Default::default(),
      };

      node.visit_with(&mut v);

      source_map_config.names = v.names;
    }

    let src = {
      let mut buf = Vec::with_capacity(source_len as usize);
      {
        let mut w = text_writer::JsWriter::new(
          source_map.clone(),
          "\n",
          &mut buf,
          source_map_config.enabled().then_some(&mut src_map_buf),
        );

        w.preamble(preamble)?;
        let mut wr = Box::new(w) as Box<dyn WriteJs>;

        if minify {
          wr = Box::new(text_writer::omit_trailing_semi(wr));
        }

        let mut emitter = Emitter {
          cfg: codegen::Config::default()
            .with_minify(minify)
            .with_target(target)
            .with_ascii_only(ascii_only)
            .with_inline_script(inline_script),
          comments,
          cm: source_map.clone(),
          wr,
        };
        node.emit_with(&mut emitter)?;
      }
      // SAFETY: SWC will emit valid utf8 for sure
      unsafe { String::from_utf8_unchecked(buf) }
    };

    let map = if source_map_config.enabled() {
      build_rspack_source_map(
        &source_map,
        &src_map_buf,
        input_source_map,
        &source_map_config,
        &src,
      )
    } else {
      None
    };

    Ok(TransformOutput {
      code: src,
      map,
      diagnostics: Default::default(),
    })
  }
}

fn build_rspack_source_map(
  swc_cm: &SwcSourceMap,
  mappings: &[(BytePos, LineCol)],
  input_source_map: Option<SourceMap<'static>>,
  config: &SourceMapConfig,
  generated_code: &str,
) -> Option<SourceMap<'static>> {
  let mut builder = RspackSourceMapBuilder::default();

  let mut cur_file: Option<Lrc<SourceFile>> = None;
  let mut cur_src_id = 0u32;
  let mut prev_dst_line = u32::MAX;
  let mut ch_state = ByteToCharPosState::default();
  let mut line_state = ByteToCharPosState::default();

  for (raw_pos, lc) in mappings {
    let pos = *raw_pos;

    if pos.is_reserved_for_comments() {
      continue;
    }

    if lc.line == 0 && lc.col == 0 && pos.is_dummy() {
      continue;
    }

    if pos == BytePos(u32::MAX) {
      builder.add_mapping(Mapping {
        generated_line: lc.line + 1,
        generated_column: lc.col,
        original: Some(OriginalLocation {
          source_index: cur_src_id,
          original_line: 1,
          original_column: 0,
          name_index: None,
        }),
      });
      continue;
    }

    let file = match cur_file {
      Some(ref file) if file.start_pos <= *raw_pos && *raw_pos < file.end_pos => file,
      _ => {
        let Some(file) = swc_cm.try_lookup_source_file(*raw_pos).ok().flatten() else {
          continue;
        };
        if config.skip(&file.name) {
          continue;
        }

        cur_src_id = builder.add_source(
          config.file_name_to_source(&file.name),
          config
            .inline_sources_content(&file.name)
            .then(|| file.src.to_string()),
        );
        if input_source_map.is_none() && config.ignore_list(&file.name) {
          builder.add_to_ignore_list(cur_src_id);
        }

        ch_state = ByteToCharPosState::default();
        line_state = ByteToCharPosState::default();
        cur_file = Some(file.clone());
        cur_file.as_ref().expect("source file was just set")
      }
    };

    if config.skip(&file.name) {
      continue;
    }

    if !config.emit_columns(&file.name) && lc.line == prev_dst_line {
      continue;
    }

    let Some(line) = file.lookup_line(pos) else {
      continue;
    };
    let line = line as u32;
    let linebpos = file.analyze().lines[line as usize];
    debug_assert!(
      pos >= linebpos,
      "{}: bpos = {:?}; linebpos = {:?};",
      file.name,
      pos,
      linebpos,
    );

    let linechpos = linebpos.to_u32() - calc_utf16_offset(file, linebpos, &mut line_state);
    let chpos = pos.to_u32() - calc_utf16_offset(file, pos, &mut ch_state);
    debug_assert!(
      chpos >= linechpos,
      "{}: chpos = {:?}; linechpos = {:?};",
      file.name,
      chpos,
      linechpos,
    );

    let name_index = if input_source_map.is_none() {
      config
        .name_for_bytepos(pos)
        .map(|name| builder.add_name(name))
    } else {
      None
    };

    builder.add_mapping(Mapping {
      generated_line: lc.line + 1,
      generated_column: lc.col,
      original: Some(OriginalLocation {
        source_index: cur_src_id,
        original_line: line + 1,
        original_column: chpos - linechpos,
        name_index,
      }),
    });
    prev_dst_line = lc.line;
  }

  let intermediate_map = builder.into_source_map();
  if let Some(input_source_map) = input_source_map {
    let ignored_sources = ignored_sources(&input_source_map);
    let name = intermediate_map
      .get_source(0)
      .map(ToString::to_string)
      .unwrap_or_default();
    Arc::new(SourceMapSource::new(SourceMapSourceOptions {
      value: generated_code.to_string(),
      name,
      source_map: intermediate_map,
      original_source: None,
      inner_source_map: Some(input_source_map),
      remove_original_source: false,
    }))
    .map_static(
      &ObjectPool::default(),
      &MapOptions::new(config.source_map_kind.emit_columns()),
    )
    .map(|mut source_map| {
      restore_ignore_list(&mut source_map, &ignored_sources);
      source_map
    })
  } else {
    Some(intermediate_map)
  }
}

fn ignored_sources(source_map: &SourceMap<'_>) -> FxHashSet<String> {
  source_map
    .ignore_list()
    .into_iter()
    .flatten()
    .filter_map(|source_index| {
      source_map
        .get_source(*source_index as usize)
        .map(|source| source_with_root(source_map, source))
    })
    .collect()
}

fn restore_ignore_list(source_map: &mut SourceMap<'static>, ignored_sources: &FxHashSet<String>) {
  if ignored_sources.is_empty() {
    return;
  }

  let ignore_list = source_map
    .sources()
    .iter()
    .enumerate()
    .filter_map(|(source_index, source)| {
      ignored_sources
        .contains(&source_with_root(source_map, source))
        .then_some(source_index as u32)
    })
    .collect::<Vec<_>>();

  if !ignore_list.is_empty() {
    source_map.set_ignore_list(Some(Cow::Owned(ignore_list)));
  }
}

fn source_with_root(source_map: &SourceMap<'_>, source: &str) -> String {
  match source_map.source_root() {
    Some("") | None => source.to_string(),
    Some(source_root) if source_root.ends_with('/') => format!("{source_root}{source}"),
    Some(source_root) => format!("{source_root}/{source}"),
  }
}

#[derive(Default)]
struct RspackSourceMapBuilder {
  mappings: Vec<Mapping>,
  sources: Vec<Cow<'static, str>>,
  sources_content: Vec<Cow<'static, str>>,
  names: Vec<Cow<'static, str>>,
  source_indices: FxHashMap<String, u32>,
  name_indices: FxHashMap<String, u32>,
  ignore_list: FxHashSet<u32>,
}

impl RspackSourceMapBuilder {
  fn add_source(&mut self, source: String, source_content: Option<String>) -> u32 {
    if let Some(index) = self.source_indices.get(&source) {
      return *index;
    }

    let index = self.sources.len() as u32;
    self.sources.push(Cow::Owned(source.clone()));
    self
      .sources_content
      .push(Cow::Owned(source_content.unwrap_or_default()));
    self.source_indices.insert(source, index);
    index
  }

  fn add_to_ignore_list(&mut self, source_index: u32) {
    self.ignore_list.insert(source_index);
  }

  fn add_name(&mut self, name: &str) -> u32 {
    if let Some(index) = self.name_indices.get(name) {
      return *index;
    }

    let index = self.names.len() as u32;
    self.names.push(Cow::Owned(name.to_string()));
    self.name_indices.insert(name.to_string(), index);
    index
  }

  fn add_mapping(&mut self, mapping: Mapping) {
    self.mappings.push(mapping);
  }

  fn into_source_map(self) -> SourceMap<'static> {
    let mut source_map = SourceMap::new(
      encode_mappings(self.mappings.into_iter()),
      self.sources,
      self.sources_content,
      self.names,
    );
    if !self.ignore_list.is_empty() {
      let mut ignore_list = self.ignore_list.into_iter().collect::<Vec<_>>();
      ignore_list.sort_unstable();
      source_map.set_ignore_list(Some(Cow::Owned(ignore_list)));
    }
    source_map
  }
}

#[derive(Default)]
struct ByteToCharPosState {
  pos: BytePos,
  total_extra_bytes: u32,
  mbc_index: usize,
}

fn calc_utf16_offset(file: &SourceFile, bpos: BytePos, state: &mut ByteToCharPosState) -> u32 {
  let mut total_extra_bytes = state.total_extra_bytes;
  let mut index = state.mbc_index;
  let analysis = file.analyze();
  if bpos >= state.pos {
    for mbc in analysis.multibyte_chars[index..].iter() {
      if mbc.pos >= bpos {
        break;
      }
      total_extra_bytes += mbc.byte_to_char_diff() as u32;
      debug_assert!(
        bpos.to_u32() >= mbc.pos.to_u32() + mbc.bytes as u32,
        "bpos = {:?}, mbc.pos = {:?}, mbc.bytes = {:?}",
        bpos,
        mbc.pos,
        mbc.bytes,
      );
      index += 1;
    }
  } else {
    for mbc in analysis.multibyte_chars[..index].iter().rev() {
      if mbc.pos < bpos {
        break;
      }
      total_extra_bytes -= mbc.byte_to_char_diff() as u32;
      debug_assert!(
        bpos.to_u32() <= mbc.pos.to_u32(),
        "bpos = {:?}, mbc.pos = {:?}",
        bpos,
        mbc.pos,
      );
      index -= 1;
    }
  }

  state.pos = bpos;
  state.total_extra_bytes = total_extra_bytes;
  state.mbc_index = index;

  total_extra_bytes
}

struct IdentCollector {
  pub names: FxHashMap<BytePos, Atom>,
}

impl Visit for IdentCollector {
  noop_visit_type!();

  fn visit_ident(&mut self, ident: &Ident) {
    self.names.insert(ident.span.lo, ident.sym.clone());
  }
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use rspack_sources::SourceMap;
  use rspack_util::source_map::SourceMapKind;
  use swc_core::common::{BytePos, FileName, LineCol, SourceMap as SwcSourceMap};

  use super::{SourceMapConfig, build_rspack_source_map};

  #[test]
  fn composes_input_source_map_in_cheap_mode() {
    let swc_cm = Arc::new(SwcSourceMap::default());
    let source = "\n\n\nconsole.log(user.name);\n";
    let file = swc_cm.new_source_file(Arc::new(FileName::Custom("input.js".into())), source);
    let line_four = BytePos(file.start_pos.0 + source.find("console").expect("source line") as u32);

    let input_source_map = SourceMap::from_json(
      r#"{
        "version": 3,
        "file": "input.js",
        "sources": ["original.vue"],
        "sourcesContent": ["<template>\n  <p>{{ user.name }}</p>\n</template>\n"],
        "names": [],
        "mappings": ";;;UACA"
      }"#
        .to_string(),
    )
    .expect("valid input source map");

    let source_map = build_rspack_source_map(
      &swc_cm,
      &[(line_four, LineCol { line: 0, col: 0 })],
      Some(input_source_map),
      &SourceMapConfig {
        source_map_kind: SourceMapKind::SourceMap.with_cheap(true),
        names: Default::default(),
      },
      "console.log(user.name);\n",
    )
    .expect("composed source map");

    let mappings = source_map.decoded_mappings().collect::<Vec<_>>();
    assert_eq!(source_map.get_source(0), Some("original.vue"));
    assert_eq!(
      mappings[0]
        .original
        .as_ref()
        .map(|original| original.original_line),
      Some(2)
    );
  }
}
