mod source_map_writer;

use std::{borrow::Cow, sync::Arc};

use bytes_str::BytesStr;
use rspack_error::Result;
use rspack_sources::{
  FullMappingsEncoder, MapOptions, Mapping, OriginalLocation, Source, SourceMap, SourceMapSource,
  SourceMapSourceOptions,
};
use rspack_util::source_map::SourceMapKind;
use rustc_hash::FxHashMap;
use source_map_writer::SourceMapWriter;
use swc_core::{
  common::{
    BytePos, FileName, LineCol, SourceFile, SourceMap as SwcSourceMap,
    comments::Comments,
    source_map::{MultiByteChar, SmallPos},
    sync::Lrc,
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

  fn name_for_bytepos(&self, pos: BytePos) -> Option<&Atom> {
    self.names.get(&pos)
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
  pub input_source_map: Option<SourceMap>,
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
    if source_map_config.enabled() {
      let mut v = IdentCollector {
        names: Default::default(),
      };

      node.visit_with(&mut v);

      source_map_config.names = v.names;
    }

    let mut source_map_generator = source_map_config
      .enabled()
      .then(|| SourceMapGenerator::new(&source_map, source_map_config, input_source_map));
    let src = {
      let mut buf = Vec::with_capacity(source_len as usize);
      {
        let wr: Box<dyn WriteJs> = if let Some(generator) = &mut source_map_generator {
          let mut writer = SourceMapWriter::new(&mut buf, |position, location| {
            generator.add_mapping(position, location);
          });
          writer.preamble(preamble)?;
          into_box_writer(writer, minify)
        } else {
          let mut writer = text_writer::JsWriter::new(source_map.clone(), "\n", &mut buf, None);
          writer.preamble(preamble)?;
          into_box_writer(writer, minify)
        };

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

    let map = source_map_generator.and_then(|generator| generator.finish(&src));

    Ok(TransformOutput {
      code: src,
      map,
      diagnostics: Default::default(),
    })
  }
}

fn into_box_writer<'a>(writer: impl WriteJs + 'a, minify: bool) -> Box<dyn WriteJs + 'a> {
  if minify {
    Box::new(text_writer::omit_trailing_semi(writer))
  } else {
    Box::new(writer)
  }
}

struct SourceMapGenerator<'a> {
  swc_cm: &'a SwcSourceMap,
  config: SourceMapConfig,
  input_source_map: Option<SourceMap>,
  builder: RspackSourceMapBuilder,
  encoder: FullMappingsEncoder,
  cur_file: Option<Lrc<SourceFile>>,
  cur_src_id: u32,
  prev_dst_line: u32,
  position_converter: SourceMapPositionConverter,
}

impl<'a> SourceMapGenerator<'a> {
  fn new(
    swc_cm: &'a SwcSourceMap,
    config: SourceMapConfig,
    input_source_map: Option<SourceMap>,
  ) -> Self {
    Self {
      swc_cm,
      config,
      input_source_map,
      builder: Default::default(),
      encoder: FullMappingsEncoder::new(),
      cur_file: None,
      cur_src_id: 0,
      prev_dst_line: u32::MAX,
      position_converter: Default::default(),
    }
  }

  fn add_mapping(&mut self, pos: BytePos, location: LineCol) {
    if let Some(mapping) = self.convert_mapping(pos, location) {
      self.encoder.encode(&mapping);
    }
  }

  fn convert_mapping(&mut self, pos: BytePos, lc: LineCol) -> Option<Mapping> {
    if pos.is_reserved_for_comments() {
      return None;
    }

    if lc.line == 0 && lc.col == 0 && pos.is_dummy() {
      return None;
    }

    if pos == BytePos(u32::MAX) {
      return Some(Mapping {
        generated_line: lc.line + 1,
        generated_column: lc.col,
        original: Some(OriginalLocation {
          source_index: self.cur_src_id,
          original_line: 1,
          original_column: 0,
          name_index: None,
        }),
      });
    }

    let file = match self.cur_file {
      Some(ref file) if file.start_pos <= pos && pos < file.end_pos => file,
      _ => {
        let file = self.swc_cm.try_lookup_source_file(pos).ok().flatten()?;
        if self.config.skip(&file.name) {
          return None;
        }

        self.cur_src_id = self.builder.add_source(
          self.config.file_name_to_source(&file.name),
          self
            .config
            .inline_sources_content(&file.name)
            .then(|| file.src.clone()),
        );
        if self.input_source_map.is_none() && self.config.ignore_list(&file.name) {
          self.builder.add_to_ignore_list(self.cur_src_id);
        }

        self.cur_file = Some(file);
        self.cur_file.as_ref().expect("source file was just set")
      }
    };

    if self.config.skip(&file.name) {
      return None;
    }

    if !self.config.emit_columns(&file.name) && lc.line == self.prev_dst_line {
      return None;
    }

    let (original_line, original_column) = self.position_converter.convert(file, pos)?;

    let name_index = self
      .config
      .name_for_bytepos(pos)
      .map(|name| self.builder.add_name(name));

    self.prev_dst_line = lc.line;
    Some(Mapping {
      generated_line: lc.line + 1,
      generated_column: lc.col,
      original: Some(OriginalLocation {
        source_index: self.cur_src_id,
        original_line,
        original_column,
        name_index,
      }),
    })
  }

  fn finish(self, generated_code: &str) -> Option<SourceMap> {
    let Self {
      config,
      input_source_map,
      builder,
      encoder,
      ..
    } = self;
    let emit_columns = config.source_map_kind.emit_columns();
    drop(config);

    let intermediate_map = builder.into_source_map(encoder.finish());
    if let Some(input_source_map) = input_source_map {
      let name = intermediate_map
        .get_source(0)
        .map(ToString::to_string)
        .unwrap_or_default();
      SourceMapSource::new(SourceMapSourceOptions {
        value: generated_code.to_string(),
        name,
        source_map: intermediate_map,
        original_source: None,
        inner_source_map: Some(input_source_map),
        remove_original_source: true,
      })
      .map(&MapOptions::new(emit_columns))
    } else {
      Some(intermediate_map)
    }
  }
}

#[derive(Default)]
struct RspackSourceMapBuilder {
  sources: Vec<BytesStr>,
  sources_content: Vec<Option<BytesStr>>,
  source_indices: FxHashMap<BytesStr, u32>,
  names: Vec<BytesStr>,
  name_indices: FxHashMap<Atom, u32>,
  ignore_list: Vec<u32>,
}

impl RspackSourceMapBuilder {
  fn add_source(&mut self, source: String, source_content: Option<BytesStr>) -> u32 {
    if let Some(&index) = self.source_indices.get(source.as_str()) {
      return index;
    }

    let source = BytesStr::from_string(source);
    let index = self.sources.len() as u32;
    self.sources.push(source.clone());
    self.sources_content.push(source_content);
    self.source_indices.insert(source, index);
    index
  }

  fn add_to_ignore_list(&mut self, source_index: u32) {
    self.ignore_list.push(source_index);
  }

  fn add_name(&mut self, name: &Atom) -> u32 {
    if let Some(&index) = self.name_indices.get(name) {
      return index;
    }

    let index = self.names.len() as u32;
    self.names.push(BytesStr::from_str_slice(name));
    self.name_indices.insert(name.clone(), index);
    index
  }

  fn into_source_map(mut self, mappings: String) -> SourceMap {
    drop(self.source_indices);
    drop(self.name_indices);
    let mut source_map = SourceMap::new(
      mappings.into(),
      self.sources,
      self.sources_content,
      self.names,
    );
    if !self.ignore_list.is_empty() {
      self.ignore_list.sort_unstable();
      self.ignore_list.dedup();
      source_map.set_ignore_list(Some(Cow::Owned(self.ignore_list)));
    }
    source_map
  }
}

#[derive(Default)]
struct SourceMapPositionConverter {
  state: Option<SourceMapPositionState>,
}

struct SourceMapPositionState {
  file_start: BytePos,
  line: u32,
  line_start: BytePos,
  line_end: BytePos,
  pos: BytePos,
  utf8_to_utf16_diff: u32,
  line_utf8_to_utf16_diff: u32,
  multibyte_index: usize,
}

impl SourceMapPositionConverter {
  /// Converts SWC UTF-8 byte positions to source-map positions. The source map
  /// specification requires columns to use zero-based UTF-16 code units.
  fn convert(&mut self, file: &SourceFile, pos: BytePos) -> Option<(u32, u32)> {
    let (line, line_start, line_end) = match &self.state {
      Some(state)
        if state.file_start == file.start_pos
          && state.line_start <= pos
          && pos < state.line_end =>
      {
        (state.line, state.line_start, state.line_end)
      }
      _ => {
        let line = file.lookup_line(pos)?;
        let (line_start, line_end) = file.line_bounds(line);
        debug_assert!(
          u32::try_from(line).is_ok(),
          "{}: line index {line} exceeds u32::MAX",
          file.name,
        );
        let line = line as u32;
        (line, line_start, line_end)
      }
    };
    debug_assert!(
      line_start <= pos && pos <= line_end,
      "{}: bpos = {:?}; line bounds = {:?}..{:?};",
      file.name,
      pos,
      line_start,
      line_end,
    );

    let source_pos = pos.to_u32().checked_sub(file.start_pos.to_u32())? as usize;
    if !file.src.is_char_boundary(source_pos) {
      return None;
    }

    let multibyte_chars = &file.analyze().multibyte_chars;
    let (mut state_pos, mut utf8_to_utf16_diff, mut multibyte_index) = match &self.state {
      Some(state) if state.file_start == file.start_pos => {
        (state.pos, state.utf8_to_utf16_diff, state.multibyte_index)
      }
      _ => (file.start_pos, 0, 0),
    };

    let line_utf8_to_utf16_diff = match &self.state {
      Some(state) if state.file_start == file.start_pos && state.line_start == line_start => {
        state.line_utf8_to_utf16_diff
      }
      _ => {
        Self::move_to(
          multibyte_chars,
          line_start,
          &mut state_pos,
          &mut utf8_to_utf16_diff,
          &mut multibyte_index,
        )?;
        utf8_to_utf16_diff
      }
    };
    Self::move_to(
      multibyte_chars,
      pos,
      &mut state_pos,
      &mut utf8_to_utf16_diff,
      &mut multibyte_index,
    )?;

    let byte_column = pos.to_u32().checked_sub(line_start.to_u32())?;
    let column =
      byte_column.checked_sub(utf8_to_utf16_diff.checked_sub(line_utf8_to_utf16_diff)?)?;

    self.state = Some(SourceMapPositionState {
      file_start: file.start_pos,
      line,
      line_start,
      line_end,
      pos,
      utf8_to_utf16_diff,
      line_utf8_to_utf16_diff,
      multibyte_index,
    });
    Some((line + 1, column))
  }

  fn move_to(
    multibyte_chars: &[MultiByteChar],
    target: BytePos,
    pos: &mut BytePos,
    utf8_to_utf16_diff: &mut u32,
    multibyte_index: &mut usize,
  ) -> Option<()> {
    if *pos <= target {
      while let Some(multibyte_char) = multibyte_chars.get(*multibyte_index) {
        if multibyte_char.pos >= target {
          break;
        }
        *utf8_to_utf16_diff =
          utf8_to_utf16_diff.checked_add(multibyte_char.byte_to_char_diff() as u32)?;
        *multibyte_index += 1;
      }
    } else {
      while let Some(index) = multibyte_index.checked_sub(1) {
        let multibyte_char = &multibyte_chars[index];
        if multibyte_char.pos < target {
          break;
        }
        *utf8_to_utf16_diff =
          utf8_to_utf16_diff.checked_sub(multibyte_char.byte_to_char_diff() as u32)?;
        *multibyte_index = index;
      }
    }
    *pos = target;
    Some(())
  }
}

#[cfg(feature = "codspeed")]
#[doc(hidden)]
pub fn benchmark_source_map_position_conversion(file: &SourceFile, positions: &[BytePos]) -> u64 {
  let mut converter = SourceMapPositionConverter::default();
  positions.iter().fold(0, |checksum, &pos| {
    let (line, column) = converter
      .convert(file, pos)
      .expect("benchmark positions should be valid");
    checksum.wrapping_add((u64::from(line) << 32) | u64::from(column))
  })
}

struct IdentCollector {
  pub names: FxHashMap<BytePos, Atom>,
}

impl Visit for IdentCollector {
  noop_visit_type!();

  fn visit_ident(&mut self, ident: &Ident) {
    if !ident.span.lo.is_dummy() && !ident.span.lo.is_reserved_for_comments() {
      self.names.insert(ident.span.lo, ident.sym.clone());
    }
  }
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use rspack_sources::{Mapping, OriginalLocation, SourceMap, encode_mappings};
  use rspack_util::source_map::SourceMapKind;
  use swc_core::common::{BytePos, FileName, LineCol, SourceMap as SwcSourceMap};

  use super::{SourceMapConfig, SourceMapGenerator};

  fn build_rspack_source_map(
    swc_cm: &SwcSourceMap,
    mappings: &[(BytePos, LineCol)],
    input_source_map: Option<SourceMap>,
    config: &SourceMapConfig,
    generated_code: &str,
  ) -> Option<SourceMap> {
    let mut generator = SourceMapGenerator::new(swc_cm, config.clone(), input_source_map);
    for &(position, location) in mappings {
      generator.add_mapping(position, location);
    }
    generator.finish(generated_code)
  }

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

  #[test]
  fn preserves_outer_names_when_input_source_map_has_no_names() {
    let swc_cm = Arc::new(SwcSourceMap::default());
    let source = "const user = value;\n";
    let file = swc_cm.new_source_file(Arc::new(FileName::Custom("input.js".into())), source);
    let user_column = source.find("user").expect("identifier column") as u32;
    let user_pos = BytePos(file.start_pos.0 + user_column);

    let input_source_map = SourceMap::new(
      encode_mappings(
        [Mapping {
          generated_line: 1,
          generated_column: user_column,
          original: Some(OriginalLocation {
            source_index: 0,
            original_line: 1,
            original_column: user_column,
            name_index: None,
          }),
        }]
        .into_iter(),
      )
      .into(),
      vec!["original.js".into()],
      vec![Some(source.into())],
      vec![],
    );

    let source_map = build_rspack_source_map(
      &swc_cm,
      &[(
        user_pos,
        LineCol {
          line: 0,
          col: user_column,
        },
      )],
      Some(input_source_map),
      &SourceMapConfig {
        source_map_kind: SourceMapKind::SourceMap,
        names: [(user_pos, "user".into())].into_iter().collect(),
      },
      source,
    )
    .expect("composed source map");

    let mappings = source_map.decoded_mappings().collect::<Vec<_>>();
    let name_index = mappings[0]
      .original
      .as_ref()
      .and_then(|original| original.name_index)
      .expect("outer name should be preserved");
    assert_eq!(source_map.get_name(name_index as usize), Some("user"));
  }

  #[test]
  fn keeps_input_source_map_gaps_unmapped() {
    let swc_cm = Arc::new(SwcSourceMap::default());
    let source = "const a = 1; const b = 2;\n";
    let file = swc_cm.new_source_file(Arc::new(FileName::Custom("input.js".into())), source);
    let a_pos = BytePos(file.start_pos.0 + source.find("a").expect("a column") as u32);
    let b_column = source.find("b").expect("b column") as u32;
    let b_pos = BytePos(file.start_pos.0 + b_column);

    let input_source_map = SourceMap::new(
      encode_mappings(
        [
          Mapping {
            generated_line: 1,
            generated_column: 0,
            original: Some(OriginalLocation {
              source_index: 0,
              original_line: 1,
              original_column: 0,
              name_index: None,
            }),
          },
          Mapping {
            generated_line: 1,
            generated_column: b_column,
            original: None,
          },
        ]
        .into_iter(),
      )
      .into(),
      vec!["original.js".into()],
      vec![Some(source.into())],
      vec![],
    );

    let source_map = build_rspack_source_map(
      &swc_cm,
      &[
        (a_pos, LineCol { line: 0, col: 0 }),
        (
          b_pos,
          LineCol {
            line: 0,
            col: b_column,
          },
        ),
      ],
      Some(input_source_map),
      &SourceMapConfig {
        source_map_kind: SourceMapKind::SourceMap,
        names: Default::default(),
      },
      source,
    )
    .expect("composed source map");

    assert_eq!(source_map.sources(), &["original.js"]);
    let mappings = source_map.decoded_mappings().collect::<Vec<_>>();
    assert!(mappings[0].original.is_some());
    assert!(mappings[1].original.is_none());
  }

  #[test]
  fn preserves_absolute_input_sources_under_source_root() {
    let swc_cm = Arc::new(SwcSourceMap::default());
    let source = "const first = second;\n";
    let file = swc_cm.new_source_file(Arc::new(FileName::Custom("input.js".into())), source);
    let first_column = source.find("first").expect("first column") as u32;
    let second_column = source.find("second").expect("second column") as u32;
    let first_pos = BytePos(file.start_pos.0 + first_column);
    let second_pos = BytePos(file.start_pos.0 + second_column);

    let mut input_source_map = SourceMap::new(
      encode_mappings(
        [
          Mapping {
            generated_line: 1,
            generated_column: first_column,
            original: Some(OriginalLocation {
              source_index: 0,
              original_line: 1,
              original_column: 0,
              name_index: None,
            }),
          },
          Mapping {
            generated_line: 1,
            generated_column: second_column,
            original: Some(OriginalLocation {
              source_index: 1,
              original_line: 1,
              original_column: 0,
              name_index: None,
            }),
          },
        ]
        .into_iter(),
      )
      .into(),
      vec!["/src/app.js".into(), "https://example.com/vendor.js".into()],
      vec![
        Some("const first = second;\n".into()),
        Some("export const second = 1;\n".into()),
      ],
      vec![],
    );
    input_source_map.set_source_root(Some("/root".into()));

    let source_map = build_rspack_source_map(
      &swc_cm,
      &[
        (
          first_pos,
          LineCol {
            line: 0,
            col: first_column,
          },
        ),
        (
          second_pos,
          LineCol {
            line: 0,
            col: second_column,
          },
        ),
      ],
      Some(input_source_map),
      &SourceMapConfig {
        source_map_kind: SourceMapKind::SourceMap,
        names: Default::default(),
      },
      source,
    )
    .expect("composed source map");

    assert_eq!(source_map.get_source(0), Some("/src/app.js"));
    assert_eq!(
      source_map.get_source(1),
      Some("https://example.com/vendor.js")
    );
  }
}
