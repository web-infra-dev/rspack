use std::{borrow::Cow, sync::Arc};

use rspack_error::Result;
use rspack_sources::{
  MapOptions, Mapping, ObjectPool, OriginalLocation, Source, SourceMap, SourceMapSource,
  SourceMapSourceOptions, encode_mappings, utf8_column_to_utf16_column,
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
            .then(|| Cow::Owned(file.src.to_string())),
        );
        if input_source_map.is_none() && config.ignore_list(&file.name) {
          builder.add_to_ignore_list(cur_src_id);
        }

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

    let Some(original_column) = source_file_utf16_column(file, linebpos, pos) else {
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
        original_line: line + 1,
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
      .map(ToString::to_string)
      .unwrap_or_default();
    Arc::new(SourceMapSource::new(SourceMapSourceOptions {
      value: generated_code.to_string(),
      name,
      source_map: intermediate_map,
      original_source: None,
      inner_source_map: Some(input_source_map),
      remove_original_source: true,
    }))
    .map_static(
      &ObjectPool::default(),
      &MapOptions::new(config.source_map_kind.emit_columns()),
    )
  } else {
    Some(intermediate_map)
  }
}

#[derive(Default)]
struct RspackSourceMapBuilder<'a> {
  mappings: Vec<Mapping>,
  sources_content: Vec<Cow<'a, str>>,
  source_indices: FxHashMap<Cow<'a, str>, u32>,
  name_indices: FxHashMap<Cow<'a, str>, u32>,
  ignore_list: FxHashSet<u32>,
}

impl<'a> RspackSourceMapBuilder<'a> {
  fn add_source(&mut self, source: Cow<'a, str>, source_content: Option<Cow<'a, str>>) -> u32 {
    if let Some(index) = self.source_indices.get(source.as_ref()) {
      return *index;
    }

    let index = self.source_indices.len() as u32;
    self
      .sources_content
      .push(source_content.unwrap_or(Cow::Borrowed("")));
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

  fn into_source_map(self) -> SourceMap<'static> {
    let mut source_map = SourceMap::new(
      encode_mappings(self.mappings.into_iter()),
      ordered_cows(self.source_indices),
      self
        .sources_content
        .into_iter()
        .map(|source_content| Cow::Owned(source_content.into_owned()))
        .collect(),
      ordered_cows(self.name_indices),
    );
    if !self.ignore_list.is_empty() {
      let mut ignore_list = self.ignore_list.into_iter().collect::<Vec<_>>();
      ignore_list.sort_unstable();
      source_map.set_ignore_list(Some(Cow::Owned(ignore_list)));
    }
    source_map
  }
}

fn ordered_cows(entries: FxHashMap<Cow<'_, str>, u32>) -> Vec<Cow<'static, str>> {
  let mut ordered = vec![Cow::Borrowed(""); entries.len()];
  for (value, index) in entries {
    ordered[index as usize] = Cow::Owned(value.into_owned());
  }
  ordered
}

fn source_file_utf16_column(file: &SourceFile, linebpos: BytePos, pos: BytePos) -> Option<u32> {
  let line_start = linebpos.to_u32().checked_sub(file.start_pos.to_u32())? as usize;
  let utf8_column = pos.to_u32().checked_sub(linebpos.to_u32())? as usize;
  let line = file.src.get(line_start..)?;
  utf8_column_to_utf16_column(line, utf8_column)?
    .try_into()
    .ok()
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

  use rspack_sources::{Mapping, OriginalLocation, SourceMap, encode_mappings};
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
      ),
      vec!["original.js".into()],
      vec![source.into()],
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
      ),
      vec!["original.js".into()],
      vec![source.into()],
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
      ),
      vec!["/src/app.js".into(), "https://example.com/vendor.js".into()],
      vec![
        "const first = second;\n".into(),
        "export const second = 1;\n".into(),
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
