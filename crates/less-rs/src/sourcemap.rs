//! Source maps: ports of `source-map-output.js` and `source-map-builder.js`,
//! plus a minimal version of the `source-map` package's generator (mapping
//! list, VLQ encoding and the JSON layout).

use cow_utils::CowUtils;

use crate::{
  css::{self, CssCtx, Output},
  error::LResult,
  options::SourceMapOptions,
  session::{Session, line_col_in},
  tree::{Node, Pos, Ruleset},
};

#[derive(Clone, PartialEq, Eq, Debug)]
struct Mapping {
  gen_line: u32,
  gen_col: u32,
  source: u32,
  orig_line: u32,
  orig_col: u32,
}

struct Generator {
  file: Option<String>,
  sources: Vec<String>,
  sources_content: Vec<(String, String)>,
  mappings: Vec<Mapping>,
  sorted: bool,
}

impl Generator {
  fn source_index(&mut self, s: &str) -> u32 {
    if let Some(i) = self.sources.iter().position(|x| x == s) {
      return i as u32;
    }
    self.sources.push(s.to_string());
    (self.sources.len() - 1) as u32
  }

  fn add_mapping(
    &mut self,
    gen_line: u32,
    gen_col: u32,
    source: &str,
    orig_line: u32,
    orig_col: u32,
  ) {
    let source = self.source_index(source);
    let m = Mapping {
      gen_line,
      gen_col,
      source,
      orig_line,
      orig_col,
    };
    if let Some(last) = self.mappings.last() {
      let after = last.gen_line < m.gen_line
        || (last.gen_line == m.gen_line && last.gen_col <= m.gen_col)
        || self.compare(last, &m) != std::cmp::Ordering::Greater;
      if !after {
        self.sorted = false;
      }
    }
    self.mappings.push(m);
  }

  fn compare(&self, a: &Mapping, b: &Mapping) -> std::cmp::Ordering {
    a.gen_line
      .cmp(&b.gen_line)
      .then(a.gen_col.cmp(&b.gen_col))
      .then(self.sources[a.source as usize].cmp(&self.sources[b.source as usize]))
      .then(a.orig_line.cmp(&b.orig_line))
      .then(a.orig_col.cmp(&b.orig_col))
  }

  fn serialize_mappings(&mut self) -> String {
    if !self.sorted {
      let mut m = std::mem::take(&mut self.mappings);
      m.sort_by(|a, b| self.compare(a, b));
      self.mappings = m;
      self.sorted = true;
    }
    let mut out = String::new();
    let mut prev_gen_line = 1u32;
    let mut prev_gen_col = 0i64;
    let mut prev_orig_line = 0i64;
    let mut prev_orig_col = 0i64;
    let mut prev_source = 0i64;
    for (i, m) in self.mappings.iter().enumerate() {
      if m.gen_line != prev_gen_line {
        prev_gen_col = 0;
        while m.gen_line != prev_gen_line {
          out.push(';');
          prev_gen_line += 1;
        }
      } else if i > 0 {
        if self.compare(m, &self.mappings[i - 1]) == std::cmp::Ordering::Equal {
          continue;
        }
        out.push(',');
      }
      vlq(&mut out, m.gen_col as i64 - prev_gen_col);
      prev_gen_col = m.gen_col as i64;
      vlq(&mut out, m.source as i64 - prev_source);
      prev_source = m.source as i64;
      vlq(&mut out, m.orig_line as i64 - 1 - prev_orig_line);
      prev_orig_line = m.orig_line as i64 - 1;
      vlq(&mut out, m.orig_col as i64 - prev_orig_col);
      prev_orig_col = m.orig_col as i64;
    }
    out
  }

  fn to_json(&mut self) -> String {
    let mappings = self.serialize_mappings();
    let mut s = String::from("{\"version\":3,\"sources\":[");
    for (i, src) in self.sources.iter().enumerate() {
      if i > 0 {
        s.push(',');
      }
      json_str(&mut s, src);
    }
    s.push_str("],\"names\":[],\"mappings\":");
    json_str(&mut s, &mappings);
    if let Some(f) = &self.file {
      s.push_str(",\"file\":");
      json_str(&mut s, f);
    }
    if !self.sources_content.is_empty() {
      s.push_str(",\"sourcesContent\":[");
      for (i, src) in self.sources.iter().enumerate() {
        if i > 0 {
          s.push(',');
        }
        match self.sources_content.iter().find(|(k, _)| k == src) {
          Some((_, c)) => json_str(&mut s, c),
          None => s.push_str("null"),
        }
      }
      s.push(']');
    }
    s.push('}');
    s
  }
}

const B64: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn vlq(out: &mut String, value: i64) {
  let mut v: u64 = if value < 0 {
    (((-value) as u64) << 1) | 1
  } else {
    (value as u64) << 1
  };
  loop {
    let mut digit = (v & 31) as u8;
    v >>= 5;
    if v > 0 {
      digit |= 32;
    }
    out.push(B64[digit as usize] as char);
    if v == 0 {
      break;
    }
  }
}

/// `JSON.stringify` for a string.
pub(crate) fn json_str(out: &mut String, s: &str) {
  out.push('"');
  for ch in s.chars() {
    match ch {
      '"' => out.push_str("\\\""),
      '\\' => out.push_str("\\\\"),
      '\n' => out.push_str("\\n"),
      '\r' => out.push_str("\\r"),
      '\t' => out.push_str("\\t"),
      '\u{8}' => out.push_str("\\b"),
      '\u{c}' => out.push_str("\\f"),
      c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
      c => out.push(c),
    }
  }
  out.push('"');
}

struct SourceMapOutput<'s, 'a> {
  s: &'s Session<'a>,
  css: String,
  any: bool,
  line: u32,
  column: u32,
  generator: Generator,
  basepath: Option<String>,
  rootpath: String,
}

impl<'s, 'a> SourceMapOutput<'s, 'a> {
  fn remove_basepath(&self, path: &str) -> String {
    if let Some(b) = &self.basepath
      && !b.is_empty()
      && path.starts_with(b.as_str())
    {
      let mut p = &path[b.len()..];
      if p.starts_with('\\') || p.starts_with('/') {
        p = &p[1..];
      }
      return p.to_string();
    }
    path.to_string()
  }
  fn normalize_filename(&self, f: &str) -> String {
    let f = f.cow_replace('\\', "/").into_owned();
    let f = self.remove_basepath(&f);
    format!("{}{}", self.rootpath, f)
  }
}

impl<'s, 'a> Output for SourceMapOutput<'s, 'a> {
  fn add(&mut self, chunk: &str, pos: Option<Pos>, map_lines: bool) {
    if chunk.is_empty() {
      return;
    }
    let loc = pos.and_then(|p| self.s.locate(p));
    let mut source: Option<(String, u32, u32)> = None;
    if let Some((fid, idx)) = loc {
      let contents = self.s.file_contents(fid);
      let ignored = self.s.file_ignored_chars(fid) as usize;
      let mut idx = idx as usize;
      let input = if ignored > 0 {
        idx = idx.saturating_sub(ignored);
        &contents[ignored.min(contents.len())..]
      } else {
        contents
      };
      let (l, c) = line_col_in(input, idx.min(input.len()));
      let name = self.normalize_filename(self.s.file_name(fid));
      source = Some((name, l + 1, c));
    }
    let lines: Vec<&str> = chunk.split('\n').collect();
    let last_cols = lines
      .last()
      .map(|l| l.encode_utf16().count() as u32)
      .unwrap_or(0);
    if let Some((name, sl, sc)) = &source {
      if !map_lines {
        self
          .generator
          .add_mapping(self.line + 1, self.column, name, *sl, *sc);
      } else {
        for i in 0..lines.len() {
          let gc = if i == 0 { self.column } else { 0 };
          let oc = if i == 0 { *sc } else { 0 };
          self
            .generator
            .add_mapping(self.line + i as u32 + 1, gc, name, *sl + i as u32, oc);
        }
      }
    }
    if lines.len() == 1 {
      self.column += last_cols;
    } else {
      self.line += lines.len() as u32 - 1;
      self.column = last_cols;
    }
    self.any = true;
    self.css.push_str(chunk);
  }
  fn is_empty(&self) -> bool {
    !self.any
  }
}

/// Generates CSS and the source map (`SourceMapBuilder.toCSS`).
pub(crate) fn generate<'a>(
  s: &Session<'a>,
  root: &'a Ruleset<'a>,
  ctx: &mut CssCtx,
  sm: &SourceMapOptions,
  filename: &str,
) -> LResult<(String, Option<String>)> {
  let mut sm = sm.clone();
  // parse-tree.js option defaults
  if sm.source_map_input_filename.is_none() {
    sm.source_map_input_filename = Some(filename.to_string());
  }
  if sm.source_map_basepath.is_none() {
    let last = filename.rfind(['/', '\\']);
    sm.source_map_basepath = Some(match last {
      Some(p) => filename[..p].to_string(),
      None => ".".to_string(),
    });
  }
  let basename_no_ext = || {
    let base = filename.rsplit(['/', '\\']).next().unwrap_or(filename);
    match base.rfind('.') {
      Some(p) if p > 0 && !base[p + 1..].contains('/') => base[..p].to_string(),
      _ => base.to_string(),
    }
  };
  if sm.source_map_full_filename.is_some() && !sm.source_map_file_inline {
    if sm.source_map_filename.is_none() && sm.source_map_url.is_none() {
      let full = sm
        .source_map_full_filename
        .clone()
        .expect("full source map filename is present");
      sm.source_map_filename = Some(full.rsplit(['/', '\\']).next().unwrap_or("").to_string());
    }
  } else if sm.source_map_filename.is_none() && sm.source_map_url.is_none() {
    if let Some(out) = &sm.source_map_output_filename {
      sm.source_map_filename = Some(format!("{}.map", out));
    } else {
      sm.source_map_filename = Some(format!("{}.css.map", basename_no_ext()));
    }
  }
  if sm.source_map_output_filename.is_none() {
    sm.source_map_output_filename = Some(format!("{}.css", basename_no_ext()));
  }

  let mut rootpath = sm
    .source_map_rootpath
    .clone()
    .map(|r| r.cow_replace('\\', "/").into_owned())
    .unwrap_or_default();
  if !rootpath.is_empty() && !rootpath.ends_with('/') {
    rootpath.push('/');
  }
  let mut out = SourceMapOutput {
    s,
    css: String::new(),
    any: false,
    line: 0,
    column: 0,
    generator: Generator {
      file: sm
        .source_map_output_filename
        .as_ref()
        .map(|f| f.cow_replace('\\', "/").into_owned()),
      sources: Vec::new(),
      sources_content: Vec::new(),
      mappings: Vec::new(),
      sorted: true,
    },
    basepath: sm
      .source_map_basepath
      .as_ref()
      .map(|b| b.cow_replace('\\', "/").into_owned()),
    rootpath,
  };
  if sm.output_source_files {
    let mut contents = Vec::new();
    let mut id = 0u32;
    while let Some(name) = file_name_checked(s, id) {
      let mut src = s.file_contents(id).to_string();
      let ignored = s.file_ignored_chars(id) as usize;
      if ignored > 0 {
        src = src[ignored.min(src.len())..].to_string();
      }
      contents.push((out.normalize_filename(name), src));
      id += 1;
    }
    out.generator.sources_content = contents;
  }
  css::gen_css(Node::Ruleset(root), ctx, &mut out)?;
  let mut map = None;
  let mut url: Option<String> = None;
  if out.any {
    map = Some(out.generator.to_json());
    url = sm.source_map_url.clone().or_else(|| {
      sm.source_map_filename
        .as_ref()
        .map(|f| f.cow_replace('\\', "/").into_owned())
    });
  }
  if sm.source_map_basepath.is_some()
    && let Some(u) = &url
  {
    url = Some(out.remove_basepath(u));
  }
  let mut css = out.css;
  // getCSSAppendage
  let mut appendage = String::new();
  let mut url_final = url;
  if sm.source_map_file_inline {
    match &map {
      None => {
        url_final = None;
      }
      Some(m) => {
        use base64::Engine;
        url_final = Some(format!(
          "data:application/json;base64,{}",
          base64::engine::general_purpose::STANDARD.encode(m)
        ));
      }
    }
  }
  if !sm.disable_sourcemap_annotation
    && let Some(u) = url_final
    && !(sm.source_map_file_inline && map.is_none())
  {
    appendage = format!("/*# sourceMappingURL={} */", u);
  }
  css.push_str(&appendage);
  Ok((css, map))
}

fn file_name_checked<'a>(s: &Session<'a>, id: u32) -> Option<&'a str> {
  if (id as usize) < s.file_count() {
    Some(s.file_name(id))
  } else {
    None
  }
}
