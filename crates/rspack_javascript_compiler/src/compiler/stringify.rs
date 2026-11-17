use std::{borrow::Cow, sync::Arc};

use bytes_str::BytesStr;
use rspack_error::Result;
use rspack_sources::{
  MapOptions, Mapping, OriginalLocation, Source, SourceMap, SourceMapSource,
  SourceMapSourceOptions, encode_mappings,
};
use rspack_util::source_map::SourceMapKind;
use rustc_hash::{FxHashMap, FxHashSet};
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
  input_source_map: Option<SourceMap>,
  config: &SourceMapConfig,
  generated_code: &str,
) -> Option<SourceMap> {
  let mut builder = RspackSourceMapBuilder::default();

  let mut cur_file: Option<Lrc<SourceFile>> = None;
  let mut cur_src_id = 0u32;
  let mut prev_dst_line = u32::MAX;
  let mut position_converter = SourceMapPositionConverter::default();

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
          Cow::Owned(config.file_name_to_source(&file.name)),
          config
            .inline_sources_content(&file.name)
            .then(|| file.src.clone()),
        );
        if input_source_map.is_none() && config.ignore_list(&file.name) {
          builder.add_to_ignore_list(cur_src_id);
        }

        cur_file = Some(file);
        cur_file.as_ref().expect("source file was just set")
      }
    };

    if config.skip(&file.name) {
      continue;
    }

    if !config.emit_columns(&file.name) && lc.line == prev_dst_line {
      continue;
    }

    let Some((original_line, original_column)) = position_converter.convert(file, pos) else {
      continue;
    };

    let name_index = config
      .name_for_bytepos(pos)
      .map(|name| builder.add_name(name));

    builder.add_mapping(Mapping {
      generated_line: lc.line + 1,
      generated_column: lc.col,
      original: Some(OriginalLocation {
        source_index: cur_src_id,
        original_line,
        original_column,
        name_index,
      }),
    });
    prev_dst_line = lc.line;
  }

  let intermediate_map = builder.into_source_map();
  if let Some(input_source_map) = input_source_map {
    let name = intermediate_map
      .get_source(0)
      .map_or_default(ToString::to_string);
    SourceMapSource::new(SourceMapSourceOptions {
      value: generated_code.to_string(),
      name,
      source_map: intermediate_map,
      original_source: None,
      inner_source_map: Some(input_source_map),
      remove_original_source: true,
    })
    .map(&MapOptions::new(config.source_map_kind.emit_columns()))
  } else {
    Some(intermediate_map)
  }
}

#[derive(Default)]
struct RspackSourceMapBuilder<'a> {
  mappings: Vec<Mapping>,
  sources_content: Vec<Option<BytesStr>>,
  source_indices: FxHashMap<Cow<'a, str>, u32>,
  name_indices: FxHashMap<Cow<'a, str>, u32>,
  ignore_list: FxHashSet<u32>,
}

impl<'a> RspackSourceMapBuilder<'a> {
  fn add_source(&mut self, source: Cow<'a, str>, source_content: Option<BytesStr>) -> u32 {
    if let Some(index) = self.source_indices.get(source.as_ref()) {
      return *index;
    }

    let index = self.source_indices.len() as u32;
    self.sources_content.push(source_content);
    self.source_indices.insert(source, index);
    index
  }

  fn add_to_ignore_list(&mut self, source_index: u32) {
    self.ignore_list.insert(source_index);
  }

  fn add_name(&mut self, name: &'a str) -> u32 {
    if let Some(index) = self.name_indices.get(name) {
      return *index;
    }

    let index = self.name_indices.len() as u32;
    self.name_indices.insert(Cow::Borrowed(name), index);
    index
  }

  fn add_mapping(&mut self, mapping: Mapping) {
    self.mappings.push(mapping);
  }

  fn into_source_map(self) -> SourceMap {
    let mut source_map = SourceMap::new(
      encode_mappings(self.mappings.into_iter()).into(),
      ordered_texts(self.source_indices),
      self.sources_content,
      ordered_texts(self.name_indices),
    );
    if !self.ignore_list.is_empty() {
      let mut ignore_list = self.ignore_list.into_iter().collect::<Vec<_>>();
      ignore_list.sort_unstable();
      source_map.set_ignore_list(Some(Cow::Owned(ignore_list)));
    }
    source_map
  }
}

fn ordered_texts(entries: FxHashMap<Cow<'_, str>, u32>) -> Vec<BytesStr> {
  let mut ordered = vec![BytesStr::new(); entries.len()];
  for (value, index) in entries {
    ordered[index as usize] = value.into_owned().into();
  }
  ordered
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
    self.names.insert(ident.span.lo, ident.sym.clone());
  }
}
