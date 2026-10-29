//! Port of `parser/parser.js`.
//!
//! The grammar functions keep the names, alternative order, backtracking
//! structure and error messages of less.js, because error fixtures compare the
//! resulting messages and positions verbatim.

use std::cell::Cell;

use cow_utils::CowUtils;
use rustc_hash::{FxHashMap, FxHashSet};

use super::{
  matchers as m,
  scanner::{ParseUntil, PendingComment, Scanner, UntilGroup},
};
use crate::{
  error::{LResult, LessError},
  session::Session,
  tree::*,
};

/// A declaration's property name: a plain name, or interpolated parts.
pub(crate) enum PropName<'a> {
  Simple(&'a str),
  Parts(smallvec::SmallVec<[Node<'a>; 4]>),
}

pub(crate) struct Parser<'a, 's> {
  /// `None` for a detached parse on a worker thread (see [`Parser::detached`]).
  s: Option<&'s Session<'a>>,
  opts: &'s crate::options::Options,
  file_name: &'a str,
  /// Warnings of a detached parse, emitted when the file is adopted.
  pub warnings: Vec<(String, Option<&'static str>)>,
  /// Line starts of the input, for positioning buffered warnings.
  line_starts: Option<Vec<u32>>,
  /// Interned declaration names of a detached parse.
  decl_names: FxHashMap<&'a str, &'a DeclName<'a>>,
  pub a: Alloc<'a>,
  pub inp: Scanner<'a>,
  pub file: u32,
  base: u32,
  current_index: u32,
  warned: FxHashSet<(&'static str, usize)>,
  /// Input positions where a call failed to parse: a call reached again
  /// there by another alternative (an operand, then an entity) fails again,
  /// and retrying every nested level makes unclosed calls exponential.
  failed_calls: FxHashMap<usize, FailedCall<'a>>,
  /// Calls of `enter` (see there).
  steps: u64,
  deprecation_counts: FxHashMap<&'static str, u32>,
  dump_line_numbers: bool,
  strict_imports: bool,
  disable_plugin_rule: bool,
  /// Nesting depth of recursive grammar rules (stack overflow guard).
  depth: u32,
}

/// The file id of errors raised by a detached parse (patched on adoption).
pub(crate) const DETACHED_FILE: u32 = u32::MAX;

type Comments<'a> = smallvec::SmallVec<[PendingComment<'a>; 2]>;

/// A failed call attempt: a retry from the same comment store and at most
/// the same nesting depth ends the same way (restored position, the same
/// furthest-error bookkeeping) and leaves the store as `after` (comments
/// are not restored on backtracking).
struct FailedCall<'a> {
  depth: u32,
  before: Comments<'a>,
  after: Comments<'a>,
}

/// Maximum nesting of blocks/parentheses/calls before reporting
/// "Maximum call stack size exceeded" (less.js hits a RangeError).
const MAX_NESTING: u32 = 1000;

const MAX_REPETITIONS: u32 = 5;

/// Parser entry points usable with [`Parser::parse_node`].
#[derive(Copy, Clone, Debug)]
pub(crate) enum ParseFn {
  Value,
  Important,
  Selectors,
  Selector,
}

pub(crate) enum Parsed<'a> {
  Node(Option<Node<'a>>),
  Important(Option<&'a str>),
  Selectors(Option<smallvec::SmallVec<[&'a Selector<'a>; 4]>>),
}

type PR<'a> = LResult<Option<Node<'a>>>;

/// `tree.Color.fromKeyword`.
pub(crate) fn color_from_keyword<'a>(a: Alloc<'a>, keyword: &'a str) -> Option<Node<'a>> {
  if let Some(hex) = crate::data::color_by_name_ci(keyword) {
    let rgb = [
      ((hex >> 16) & 0xff) as f64,
      ((hex >> 8) & 0xff) as f64,
      (hex & 0xff) as f64,
    ];
    return Some(Node::Color(a.color(rgb, 1.0, Some(keyword))));
  }
  if keyword.eq_ignore_ascii_case("transparent") {
    return Some(Node::Color(a.color([0.0, 0.0, 0.0], 0.0, Some(keyword))));
  }
  None
}

/// `new Color(hexString, undefined, originalForm)`.
pub(crate) fn color_from_hex<'a>(
  a: Alloc<'a>,
  hex: &str,
  original: Option<&'a str>,
) -> &'a Color<'a> {
  let b = hex.as_bytes();
  let mut rgb = [0.0; 3];
  let mut alpha = 1.0;
  let val =
    |c: &[u8]| u32::from_str_radix(std::str::from_utf8(c).unwrap_or("0"), 16).unwrap_or(0) as f64;
  if b.len() >= 6 {
    for (i, chunk) in b.chunks(2).enumerate() {
      if chunk.len() < 2 {
        break;
      }
      if i < 3 {
        rgb[i] = val(chunk);
      } else {
        alpha = val(chunk) / 255.0;
      }
    }
  } else {
    for (i, &c) in b.iter().enumerate() {
      let pair = [c, c];
      if i < 3 {
        rgb[i] = val(&pair);
      } else {
        alpha = val(&pair) / 255.0;
      }
    }
  }
  a.color(rgb, alpha, original)
}

fn trim_js(s: &str) -> &str {
  s.trim_matches(|c: char| c.is_whitespace() || c == '\u{feff}')
}

impl<'a, 's> Parser<'a, 's> {
  pub fn new(s: &'s Session<'a>, file: u32, input: &'a str, current_index: u32) -> Parser<'a, 's> {
    Parser {
      s: Some(s),
      opts: s.opts,
      file_name: s.file_name(file),
      warnings: Vec::new(),
      line_starts: None,
      decl_names: FxHashMap::default(),
      a: s.a,
      inp: Scanner::new(input),
      file,
      base: s.file_base(file),
      current_index,
      warned: FxHashSet::default(),
      failed_calls: FxHashMap::default(),
      steps: 0,
      deprecation_counts: FxHashMap::default(),
      dump_line_numbers: s.opts.dump_line_numbers.is_some(),
      strict_imports: s.opts.strict_imports,
      disable_plugin_rule: s.opts.disable_plugin_rule,
      depth: 0,
    }
  }

  /// A parser for a whole file that does not touch the session (it may run
  /// on another thread): `base` was reserved with
  /// [`Session::reserve_base`] and warnings are buffered.
  pub fn detached(
    a: Alloc<'a>,
    opts: &'s crate::options::Options,
    file_name: &'a str,
    base: u32,
    input: &'a str,
  ) -> Parser<'a, 's> {
    Parser {
      s: None,
      opts,
      file_name,
      warnings: Vec::new(),
      line_starts: None,
      decl_names: FxHashMap::default(),
      a,
      inp: Scanner::new(input),
      file: DETACHED_FILE,
      base,
      current_index: 0,
      warned: FxHashSet::default(),
      failed_calls: FxHashMap::default(),
      steps: 0,
      deprecation_counts: FxHashMap::default(),
      dump_line_numbers: opts.dump_line_numbers.is_some(),
      strict_imports: opts.strict_imports,
      disable_plugin_rule: opts.disable_plugin_rule,
      depth: 0,
    }
  }

  fn enter(&mut self) -> LResult<()> {
    self.depth += 1;
    self.tick()?;
    if self.depth > MAX_NESTING {
      let mut e = LessError::native("Maximum call stack size exceeded");
      e.set_pos(self.pos(self.inp.i));
      return Err(e);
    }
    Ok(())
  }

  /// One step of work. Backtracking that never ends in less.js
  /// (exponential retries, growing comment stores) meets a step budget and
  /// the arena budget; loops that do not recurse call this directly.
  fn tick(&mut self) -> LResult<()> {
    self.steps += 1;
    if self.steps & 4095 == 0 {
      let work = self.steps + self.inp.comments_stored;
      if work
        > (self.inp.input.len() as u64)
          .saturating_mul(100)
          .saturating_add(1_000_000)
      {
        let mut e = LessError::new(
          "Parse",
          "Too complex: the input needs more backtracking than the parser allows",
        );
        e.set_pos(self.pos(self.inp.i));
        return Err(e);
      }
      // the compilation's sources so far (a parser thread: this file's)
      let source = self.s.map_or(self.inp.input.len(), |s| {
        s.source_len().max(self.inp.input.len())
      });
      let budget = crate::error::arena_budget(source);
      if self.a.bump.allocated_bytes() > budget {
        return Err(crate::error::out_of_memory(budget));
      }
    }
    Ok(())
  }
  fn leave(&mut self) {
    self.depth -= 1;
  }

  /// Position of a scanner index (`index + currentIndex`).
  #[inline]
  fn pos(&self, i: usize) -> Pos {
    Pos(self.base + self.current_index + i as u32)
  }
  /// Position without `currentIndex` (a few less.js nodes omit it).
  #[inline]
  fn pos_raw(&self, i: usize) -> Pos {
    Pos(self.base + i as u32)
  }

  fn error<T>(&self, msg: impl Into<String>, typ: Option<&str>) -> LResult<T> {
    Err(LessError::at(
      typ.unwrap_or("Syntax"),
      msg,
      self.pos(self.inp.i),
    ))
  }

  fn warn(&mut self, msg: &str, index: Option<usize>, typ: Option<&str>, id: Option<&'static str>) {
    if self.opts.quiet {
      return;
    }
    if id.is_some() && self.opts.quiet_deprecations {
      return;
    }
    let index = index.unwrap_or(self.inp.i);
    if let Some(id) = id {
      if !self.warned.insert((id, index)) {
        return;
      }
      let c = self.deprecation_counts.entry(id).or_insert(0);
      *c += 1;
      if *c > MAX_REPETITIONS {
        return;
      }
    }
    let typ = match typ {
      Some(t) => format!("{} WARNING", t.cow_to_uppercase()),
      None => "WARNING".to_string(),
    };
    match self.s {
      Some(s) => {
        let text = s.format_positioned(&typ, msg, self.pos(index));
        s.warn(text, id);
      }
      None => {
        let input = self.inp.input;
        let starts = self
          .line_starts
          .get_or_insert_with(|| crate::session::line_starts(input));
        let text =
          crate::session::format_positioned_with(&typ, msg, self.file_name, input, starts, index);
        self.warnings.push((text, id));
      }
    }
  }

  /// An interned name (a detached parse keeps its own table).
  fn decl_name(&mut self, name: &'a str) -> &'a DeclName<'a> {
    match self.s {
      Some(s) if std::ptr::eq(s.a.bump, self.a.bump) => s.decl_name(name),
      _ => self
        .decl_names
        .entry(name)
        .or_insert_with(|| self.a.alloc(DeclName::Str(name))),
    }
  }

  fn warn_bare_at_rule_variable(&mut self, index: usize) {
    self.warn(
            "A bare @variable in an at-rule prelude is deprecated. Use @{variable} interpolation instead.",
            Some(index),
            Some("DEPRECATED"),
            Some("variable-in-at-rule-prelude"),
        );
  }

  fn warn_numeric_variable_name(&mut self, name: &str, index: usize) {
    let b = name.trim_start_matches('@').as_bytes();
    let stripped = if name.starts_with("@@") {
      &name.as_bytes()[2..]
    } else if name.starts_with('@') {
      &name.as_bytes()[1..]
    } else {
      name.as_bytes()
    };
    let _ = b;
    if stripped
      .first()
      .map(|c| c.is_ascii_digit())
      .unwrap_or(false)
    {
      self.warn(
                "Variable names beginning with a number are deprecated and will be removed in Less 5.x. Rename the variable to start with a valid identifier character.",
                Some(index),
                Some("DEPRECATED"),
                Some("numeric-variable-name"),
            );
    }
  }

  fn warn_dash_only_variable_name(&mut self, name: &str, index: usize) {
    let bare = if let Some(s) = name.strip_prefix("@@") {
      s
    } else if let Some(s) = name.strip_prefix('@') {
      s
    } else {
      name
    };
    if bare == "-" {
      self.warn(
                "The dash-only variable names @- and @{-} are deprecated and will be removed in Less 5.x. Rename the variable to use a valid identifier.",
                Some(index),
                Some("DEPRECATED"),
                Some("dash-only-variable-name"),
            );
    }
  }

  fn warn_dash_only_mixin_name(&mut self, name: &str, index: usize) {
    if name == ".-" || name == "#-" {
      self.warn(
                "The dash-only mixin names .-() and #-() are deprecated and will be removed in Less 5.x. Rename the mixin to use a valid CSS identifier.",
                Some(index),
                Some("DEPRECATED"),
                Some("dash-only-mixin-name"),
            );
    }
  }

  fn is_bare_variable_reference(e: &Option<Node<'a>>) -> bool {
    matches!(
      e,
      Some(Node::Variable(_) | Node::VariableCall(_) | Node::NamespaceValue(_))
    )
  }

  fn expect_char(&mut self, c: u8, msg: Option<&str>) -> LResult<()> {
    if self.inp.char(c) {
      return Ok(());
    }
    let m = match msg {
      Some(m) => m.to_string(),
      None => format!("expected '{}' got '{}'", c as char, self.inp.cur_str()),
    };
    self.error(m, None)
  }

  fn debug_info(&self, index: usize) -> DebugInfo<'a> {
    let (line, _) = crate::session::line_col_in(self.inp.input, index.min(self.inp.input.len()));
    DebugInfo {
      line_number: line + 1,
      file_name: self.file_name,
    }
  }

  // ------------------------------------------------------------------
  // Entry points
  // ------------------------------------------------------------------

  /// Parses a whole stylesheet into the root ruleset's rules.
  pub fn parse_root(&mut self) -> LResult<&'a Ruleset<'a>> {
    let rules = self.primary()?;
    let root = self.a.ruleset(None, Some(self.a.vec_from(&rules)));
    root.root.set(true);
    root.first_root.set(true);
    let end = self.inp.end();
    if !end.is_finished {
      let message = match end.furthest_possible_error_message {
        Some(m) => m,
        None => {
          let mut m = "Unrecognised input".to_string();
          if end.furthest_char == Some(b'}') {
            m.push_str(". Possibly missing opening '{'");
          } else if end.furthest_char == Some(b')') {
            m.push_str(". Possibly missing opening '('");
          } else if end.furthest_reached_end {
            m.push_str(". Possibly missing something");
          }
          m
        }
      };
      return Err(LessError::at("Parse", message, self.pos(end.furthest)));
    }
    Ok(root)
  }

  /// `parseNode`: runs the given parsers over the whole input. Returns
  /// `None` when the input was not fully consumed.
  pub fn parse_node(
    &mut self,
    list: &[ParseFn],
  ) -> LResult<Option<smallvec::SmallVec<[Parsed<'a>; 2]>>> {
    let mut out = smallvec::SmallVec::new();
    for p in list {
      let r = match p {
        ParseFn::Value => Parsed::Node(self.value()?),
        ParseFn::Important => Parsed::Important(self.important()),
        ParseFn::Selectors => Parsed::Selectors(self.selectors()?),
        ParseFn::Selector => Parsed::Node(self.selector(true)?.map(Node::Selector)),
      };
      out.push(r);
    }
    let end = self.inp.end();
    if end.is_finished {
      Ok(Some(out))
    } else {
      Ok(None)
    }
  }

  // ------------------------------------------------------------------
  // primary / comment
  // ------------------------------------------------------------------

  fn primary(&mut self) -> LResult<smallvec::SmallVec<[Node<'a>; 16]>> {
    let mut root = smallvec::SmallVec::new();
    loop {
      while let Some(c) = self.comment() {
        root.push(c);
      }
      if self.inp.finished {
        break;
      }
      if self.inp.peek_char(b'}') {
        break;
      }
      if let Some(ext) = self.extend(true)? {
        for e in ext {
          root.push(Node::Extend(e));
        }
        continue;
      }
      let mut node = self.mixin_definition()?;
      if node.is_none() {
        node = self.declaration()?;
      }
      if node.is_none() {
        node = self.mixin_call(false, Some(false))?;
      }
      if node.is_none() {
        node = self.ruleset()?;
      }
      if node.is_none() {
        node = self.variable_call(None)?;
      }
      if node.is_none() {
        node = self.entities_call()?;
      }
      if node.is_none() {
        node = self.atrule()?;
      }
      if self.inp.fault {
        let mut e = LessError::native("Cannot read properties of undefined (reading 'current')");
        e.file = Some(self.file);
        return Err(e);
      }
      match node {
        Some(n) => root.push(n),
        None => {
          let mut found = false;
          while self.inp.char(b';') {
            found = true;
          }
          if !found {
            break;
          }
        }
      }
    }
    Ok(root)
  }

  fn comment(&mut self) -> Option<Node<'a>> {
    let c = self.inp.comment_store.pop_front()?;
    Some(Node::Comment(self.a.alloc(Comment {
      vis: Vis::default(),
      value: c.text,
      is_line_comment: c.is_line_comment,
      pos: self.pos(c.index),
      debug_info: Cell::new(None),
    })))
  }

  // ------------------------------------------------------------------
  // entities
  // ------------------------------------------------------------------

  fn mixin_lookup(&mut self) -> PR<'a> {
    self.mixin_call(true, Some(true))
  }

  fn quoted(&mut self, force_escaped: bool) -> PR<'a> {
    let index = self.inp.i;
    let mut escaped = false;
    self.inp.save();
    if self.inp.char(b'~') {
      escaped = true;
    } else if force_escaped {
      self.inp.restore();
      return Ok(None);
    }
    let s = match self.inp.quoted() {
      Some(s) => s,
      None => {
        self.inp.restore();
        return Ok(None);
      }
    };
    self.inp.forget();
    let q = self
      .a
      .quoted(&s[..1], &s[1..s.len() - 1], escaped, self.pos(index));
    Ok(Some(Node::Quoted(q)))
  }

  fn keyword(&mut self) -> Option<Node<'a>> {
    let k = if self.inp.char(b'%') {
      Some("%")
    } else {
      self.inp.re(m::keyword)
    };
    let k = k?;
    Some(color_from_keyword(self.a, k).unwrap_or_else(|| self.a.keyword(k)))
  }

  fn entities_call(&mut self) -> PR<'a> {
    self.enter()?;
    let r = if self.depth.is_multiple_of(8) {
      crate::eval::grow(|| self.entities_call_inner())
    } else {
      self.entities_call_inner()
    };
    self.leave();
    r
  }

  fn entities_call_inner(&mut self) -> PR<'a> {
    let index = self.inp.i;
    if m::url_start_ci(self.inp.rest_bytes()) {
      return Ok(None);
    }
    if let Some(f) = (!self.failed_calls.is_empty())
      .then(|| self.failed_calls.get(&index))
      .flatten()
      && self.depth <= f.depth
      && self.inp.comment_store.iter().eq(f.before.iter())
    {
      let after = f.after.clone();
      self.inp.comment_store.clear();
      self.inp.comment_store.extend(after);
      return Ok(None);
    }
    // (a few pending comments at most: copying larger stores per attempt
    // would cost more than the retries it saves)
    let memo = self.inp.comment_store.len() <= 4;
    let before: Comments<'a> = if memo {
      self.inp.comment_store.iter().copied().collect()
    } else {
      Comments::new()
    };
    self.inp.save();
    let (total, name_len) = match m::call_name(self.inp.rest_bytes()) {
      Some(x) => x,
      None => {
        self.inp.forget();
        return Ok(None);
      }
    };
    let name = &self.inp.rest()[..name_len];
    self.inp.consume(total);
    let mut prev_args: Option<Node<'a>> = None;
    if name.eq_ignore_ascii_case("alpha") {
      // ieAlpha with stop = true
      if let Some(q) = self.ie_alpha()? {
        self.inp.forget();
        return Ok(Some(q));
      }
    } else if name.eq_ignore_ascii_case("boolean") || name.eq_ignore_ascii_case("if") {
      let c = self.condition(false, true)?;
      match c {
        Some(c) => prev_args = Some(c),
        None => return self.error("expected condition", None),
      }
    }
    let args = self.arguments(prev_args)?;
    if !self.inp.char(b')') {
      self
        .inp
        .restore_msg(Some("Could not parse call arguments or missing ')'"));
      if memo && self.inp.comment_store.len() <= 4 {
        let after = self.inp.comment_store.iter().copied().collect();
        let depth = self
          .failed_calls
          .get(&index)
          .filter(|f| f.before == before)
          .map_or(self.depth, |f| f.depth.max(self.depth));
        self.failed_calls.insert(
          index,
          FailedCall {
            depth,
            before,
            after,
          },
        );
      }
      return Ok(None);
    }
    self.inp.forget();
    Ok(Some(self.a.call(name, &args, self.pos(index))))
  }

  fn declaration_call(&mut self) -> PR<'a> {
    let index = self.inp.i;
    self.inp.save();
    let valid = match self.inp.re(m::word_paren) {
      Some(v) => v,
      None => {
        self.inp.forget();
        return Ok(None);
      }
    };
    let valid = &valid[..valid.find('(').unwrap_or(valid.len())];
    if matches!(
      valid.cow_to_ascii_lowercase().as_ref(),
      "and" | "or" | "not" | "only" | "layer"
    ) {
      self.inp.restore();
      return Ok(None);
    }
    let rule = self.rule_property()?;
    let mut value = None;
    if rule.is_some() {
      value = self.value()?;
    }
    let mut args: Vec<Node<'a>> = Vec::new();
    if let (Some(rule), Some(value)) = (rule, value) {
      let d = self.a.declaration(
        DeclName::Parts(self.a.slice(&rule)),
        value,
        "",
        Merge::None,
        self.pos(self.inp.i),
        true,
        None,
      );
      args.push(Node::Declaration(d));
    }
    if !self.inp.char(b')') {
      self
        .inp
        .restore_msg(Some("Could not parse call arguments or missing ')'"));
      return Ok(None);
    }
    self.inp.forget();
    Ok(Some(self.a.call(self.a.str(valid), &args, self.pos(index))))
  }

  fn arguments(
    &mut self,
    prev_args: Option<Node<'a>>,
  ) -> LResult<smallvec::SmallVec<[Node<'a>; 4]>> {
    let mut has_prev = prev_args.is_some();
    let mut args_comma: smallvec::SmallVec<[Node<'a>; 4]> = prev_args.into_iter().collect();
    let mut args_semi: smallvec::SmallVec<[Node<'a>; 4]> = smallvec::SmallVec::new();
    let mut is_semi = false;
    let mut stalled_at: Option<usize> = None;
    self.inp.save();
    loop {
      if has_prev {
        has_prev = false;
      } else {
        // Divergence: an argument made only of comments re-absorbed by
        // a failed attempt does not advance the input; less.js loops
        // until it runs out of memory, stop instead.
        if stalled_at == Some(self.inp.i) {
          break;
        }
        stalled_at = Some(self.inp.i);
        let mut value = self.detached_ruleset()?;
        if value.is_none() {
          value = self.assignment()?;
        }
        if value.is_none() {
          value = self.expression()?;
        }
        let mut value = match value {
          Some(v) => v,
          None => break,
        };
        if let Node::Expression(e) = value
          && e.value.len() == 1
        {
          value = e.value[0];
        }
        args_comma.push(value);
      }
      if self.inp.char(b',') {
        continue;
      }
      if self.inp.char(b';') || is_semi {
        is_semi = true;
        let value = if args_comma.is_empty() {
          self.a.null_anonymous()
        } else {
          self.a.value(&args_comma)
        };
        args_semi.push(value);
        args_comma.clear();
      }
    }
    self.inp.forget();
    Ok(if is_semi { args_semi } else { args_comma })
  }

  fn literal(&mut self) -> PR<'a> {
    if let Some(d) = self.dimension() {
      return Ok(Some(d));
    }
    if let Some(c) = self.color() {
      return Ok(Some(c));
    }
    if let Some(q) = self.quoted(false)? {
      return Ok(Some(q));
    }
    Ok(self.unicode_descriptor())
  }

  fn assignment(&mut self) -> PR<'a> {
    self.inp.save();
    let key = match self.inp.re(m::assignment_key) {
      Some(k) => k,
      None => {
        self.inp.restore();
        return Ok(None);
      }
    };
    if !self.inp.char(b'=') {
      self.inp.restore();
      return Ok(None);
    }
    match self.entity()? {
      Some(value) => {
        self.inp.forget();
        Ok(Some(Node::Assignment(
          self.a.alloc(Assignment { key, value }),
        )))
      }
      None => {
        self.inp.restore();
        Ok(None)
      }
    }
  }

  fn url(&mut self) -> PR<'a> {
    let index = self.inp.i;
    self.inp.auto_comment_absorb = false;
    if !self.inp.str("url(") {
      self.inp.auto_comment_absorb = true;
      return Ok(None);
    }
    let mut value: Option<Node<'a>> = self.quoted(false)?;
    if value.is_none() {
      value = self.entities_variable()?;
    }
    if value.is_none() {
      value = self.entities_property();
    }
    let node = match value {
      Some(v) => v,
      None => {
        let text = self.inp.re(m::url_body).unwrap_or("");
        if m::has_interpolation(text) {
          Node::Quoted(self.a.quoted("", text, true, self.pos(index)))
        } else {
          self.a.anonymous(text, self.pos(index))
        }
      }
    };
    self.inp.auto_comment_absorb = true;
    self.expect_char(b')', None)?;
    Ok(Some(Node::Url(self.a.alloc(Url {
      value: node,
      pos: self.pos(index),
      is_evald: false,
    }))))
  }

  fn entities_variable(&mut self) -> PR<'a> {
    let index = self.inp.i;
    self.inp.save();
    if self.inp.cur() == b'@'
      && let Some(name) = self.inp.re(m::variable_name)
    {
      self.warn_numeric_variable_name(name, index);
      self.warn_dash_only_variable_name(name, index);
      let ch = self.inp.cur();
      if (ch == b'(' || ch == b'[')
        && !is_js_space(self.inp.prev())
        && let Some(r) = self.variable_call(Some(name))?
      {
        self.inp.forget();
        return Ok(Some(r));
      }
      self.inp.forget();
      return Ok(Some(self.a.variable(name, self.pos(index))));
    }
    self.inp.restore();
    Ok(None)
  }

  fn variable_curly(&mut self) -> PR<'a> {
    let index = self.inp.i;
    if self.inp.cur() != b'@' {
      return Ok(None);
    }
    self.inp.save();
    let curly = match self.inp.re(m::variable_curly_start) {
      Some(c) => c,
      None => {
        self.inp.restore();
        return Ok(None);
      }
    };
    let name = &curly[2..];
    let lookups = self.rule_lookups();
    if !self.inp.char(b'}') {
      self.inp.restore();
      return Ok(None);
    }
    self.inp.forget();
    self.warn_numeric_variable_name(name, index);
    self.warn_dash_only_variable_name(name, index);
    let var_name = self.a.str(&format!("@{}", name));
    match lookups {
      None => Ok(Some(self.a.variable(var_name, self.pos(index)))),
      Some(l) => {
        let call = Node::VariableCall(self.a.alloc(VariableCall {
          vis: Vis::default(),
          variable: var_name,
          pos: self.pos(index),
        }));
        Ok(Some(Node::NamespaceValue(self.a.alloc(NamespaceValue {
          value: call,
          lookups: self.a.slice(&l),
          pos: self.pos(index),
        }))))
      }
    }
  }

  fn entities_property(&mut self) -> Option<Node<'a>> {
    let index = self.inp.i;
    if self.inp.cur() == b'$'
      && let Some(name) = self.inp.re(m::property_name)
    {
      return Some(Node::Property(self.a.alloc(Property {
        name,
        pos: self.pos(index),
        evaluating: Cell::new(false),
      })));
    }
    None
  }

  fn color(&mut self) -> Option<Node<'a>> {
    self.inp.save();
    if self.inp.cur() == b'#'
      && let Some((len, digits, trailing)) = m::hex_color(self.inp.rest_bytes())
    {
      let text = self.inp.consume(len);
      if !trailing {
        self.inp.forget();
        let c = color_from_hex(self.a, &text[1..1 + digits], Some(text));
        return Some(Node::Color(c));
      }
    }
    self.inp.restore();
    None
  }

  fn color_keyword(&mut self) -> Option<Node<'a>> {
    self.inp.save();
    let aca = self.inp.auto_comment_absorb;
    self.inp.auto_comment_absorb = false;
    let k = self.inp.re(m::color_keyword);
    self.inp.auto_comment_absorb = aca;
    let k = match k {
      Some(k) => k,
      None => {
        self.inp.forget();
        return None;
      }
    };
    self.inp.restore();
    let c = color_from_keyword(self.a, k)?;
    self.inp.str(k);
    Some(c)
  }

  fn dimension(&mut self) -> Option<Node<'a>> {
    if self.inp.peek_not_numeric() {
      return None;
    }
    let (len, num_len) = m::dimension(self.inp.rest_bytes())?;
    let text = self.inp.consume(len);
    let value = crate::num::parse_float(&text[..num_len]);
    let unit = &text[num_len..];
    let u = if unit.is_empty() {
      &EMPTY_UNIT as &Unit
    } else {
      self.a.unit(&[unit], &[], None)
    };
    Some(self.a.dimension(value, u))
  }

  fn unicode_descriptor(&mut self) -> Option<Node<'a>> {
    let ud = self.inp.re(m::unicode_descriptor)?;
    Some(Node::UnicodeDescriptor(
      self.a.alloc(UnicodeDescriptor { value: ud }),
    ))
  }

  fn javascript(&mut self) -> PR<'a> {
    let index = self.inp.i;
    self.inp.save();
    let escape = self.inp.char(b'~');
    let js_quote = self.inp.char(b'`');
    if !js_quote {
      self.inp.restore();
      return Ok(None);
    }
    if let Some(js) = self.inp.re(m::until_backtick) {
      self.warn(
                "Inline JavaScript evaluation (backtick expressions) is deprecated and will be removed in Less 5.x. Use Less functions or custom plugins instead.",
                Some(index),
                Some("DEPRECATED"),
                Some("js-eval"),
            );
      self.inp.forget();
      return Ok(Some(Node::JavaScript(self.a.alloc(JavaScript {
        expression: &js[..js.len() - 1],
        escaped: escape,
        pos: self.pos(index),
      }))));
    }
    self.inp.restore_msg(Some("invalid javascript definition"));
    Ok(None)
  }

  // ------------------------------------------------------------------
  // variables
  // ------------------------------------------------------------------

  fn variable(&mut self) -> Option<&'a str> {
    if self.inp.cur() != b'@' {
      return None;
    }
    let (len, name_len) = m::variable_def(self.inp.rest_bytes())?;
    let text = self.inp.consume(len);
    let name = &text[..name_len];
    let idx = self.inp.i.saturating_sub(len);
    self.warn_numeric_variable_name(name, idx);
    self.warn_dash_only_variable_name(name, idx);
    Some(name)
  }

  fn variable_call(&mut self, parsed_name: Option<&'a str>) -> PR<'a> {
    let i = self.inp.i;
    let in_value = parsed_name.is_some();
    self.inp.save();
    let (name, has_parens) = match parsed_name {
      Some(n) => (n, false),
      None => {
        if self.inp.cur() != b'@' {
          self.inp.restore();
          return Ok(None);
        }
        match m::variable_call(self.inp.rest_bytes()) {
          Some((len, name_len, parens)) => {
            let t = self.inp.consume(len);
            (&t[..name_len], parens)
          }
          None => {
            self.inp.restore();
            return Ok(None);
          }
        }
      }
    };
    let lookups = self.rule_lookups();
    if lookups.is_none() {
      let cond = if in_value {
        // `$str('()') !== '()'` is evaluated (and may consume) but the
        // `name[2] !== '()'` alternative is always true for strings.
        self.inp.str("()");
        true
      } else {
        !has_parens
      };
      if cond {
        self
          .inp
          .restore_msg(Some("Missing '[...]' lookup in variable call"));
        return Ok(None);
      }
    }
    if !in_value {
      self.warn_numeric_variable_name(name, i);
      self.warn_dash_only_variable_name(name, i);
    }
    let call = self.a.alloc(VariableCall {
      vis: Vis::default(),
      variable: name,
      pos: self.pos_raw(i),
    });
    if !in_value && self.end() {
      self.inp.forget();
      return Ok(Some(Node::VariableCall(call)));
    }
    self.inp.forget();
    Ok(Some(Node::NamespaceValue(self.a.alloc(NamespaceValue {
      value: Node::VariableCall(call),
      lookups: self.a.slice(&lookups.unwrap_or_default()),
      pos: self.pos_raw(i),
    }))))
  }

  // ------------------------------------------------------------------
  // extend
  // ------------------------------------------------------------------

  fn extend(&mut self, is_rule: bool) -> LResult<Option<Vec<&'a Extend<'a>>>> {
    let index = self.inp.i;
    if !self.inp.str(if is_rule { "&:extend(" } else { ":extend(" }) {
      return Ok(None);
    }
    let mut extend_list: Vec<&'a Extend<'a>> = Vec::new();
    loop {
      let mut option: Option<&'a str> = None;
      let mut elements: Vec<&'a Element<'a>> = Vec::new();
      let mut first = true;
      loop {
        if let Some(o) = self.inp.re(m::extend_all) {
          option = Some(o);
          break;
        }
        let e = match self.element()? {
          Some(e) => e,
          None => break,
        };
        if !first && !e.combinator.get().value().is_empty() {
          self.warn(
                        "Targeting complex selectors can have unexpected behavior, and this behavior may change in the future.",
                        Some(index),
                        None,
                        None,
                    );
        }
        first = false;
        elements.push(e);
      }
      if elements.is_empty() {
        return self.error("Missing target selector for :extend().", None);
      }
      let sel = self
        .a
        .selector(&elements, None, None, Pos::NONE, VisInfo::default());
      let ext = self
        .a
        .extend(sel, option, self.pos(index), VisInfo::default());
      extend_list.push(ext);
      if !self.inp.char(b',') {
        break;
      }
    }
    if !self.inp.char(b')') {
      return self.error("unexpected token", None);
    }
    if is_rule && !self.inp.char(b';') {
      return self.error("unexpected token", None);
    }
    Ok(Some(extend_list))
  }

  // ------------------------------------------------------------------
  // mixins
  // ------------------------------------------------------------------

  fn mixin_call(&mut self, in_value: bool, get_lookup: Option<bool>) -> PR<'a> {
    let s = self.inp.cur();
    let mut important = false;
    let index = self.inp.i;
    let mut args: &'a [MixinArg<'a>] = &[];
    let mut has_parens = false;
    if s != b'.' && s != b'#' {
      return Ok(None);
    }
    self.inp.save();
    let elements = self.mixin_elements();
    if let Some(elements) = elements {
      let parens_index = self.inp.i;
      let parens_ws = self.inp.is_whitespace(-1);
      if self.inp.char(b'(') {
        let r = self.mixin_args(true)?;
        args = self.a.slice(&r.0);
        self.expect_char(b')', None)?;
        has_parens = true;
        if parens_ws {
          self.warn(
            "Whitespace between a mixin name and parentheses for a mixin call is deprecated",
            Some(parens_index),
            Some("DEPRECATED"),
            Some("mixin-call-whitespace"),
          );
        }
      }
      let mut lookups = None;
      if get_lookup != Some(false) {
        lookups = self.rule_lookups();
      }
      if get_lookup == Some(true) && lookups.is_none() {
        self.inp.restore();
        return Ok(None);
      }
      if in_value && lookups.is_none() && !has_parens {
        self.inp.restore();
        return Ok(None);
      }
      if !in_value && self.important().is_some() {
        important = true;
      }
      if in_value || self.end() {
        self.inp.forget();
        let sel = self
          .a
          .selector(&elements, None, None, Pos::NONE, VisInfo::default());
        let mixin = self.a.alloc(MixinCall {
          vis: Vis::default(),
          selector: sel,
          arguments: args,
          pos: self.pos(index),
          important: lookups.is_none() && important,
        });
        for el in &elements {
          if let ElValue::Str(v) = el.value {
            let idx = (el.pos.0 - self.base - self.current_index) as usize;
            self.warn_dash_only_mixin_name(v, idx);
          }
        }
        if let Some(l) = lookups {
          return Ok(Some(Node::NamespaceValue(self.a.alloc(NamespaceValue {
            value: Node::MixinCall(mixin),
            lookups: self.a.slice(&l),
            pos: Pos::NONE,
          }))));
        }
        if !has_parens {
          self.warn(
            "Calling a mixin without parentheses is deprecated",
            Some(parens_index),
            Some("DEPRECATED"),
            Some("mixin-call-no-parens"),
          );
        }
        return Ok(Some(Node::MixinCall(mixin)));
      }
    }
    self.inp.restore();
    Ok(None)
  }

  fn mixin_elements(&mut self) -> Option<smallvec::SmallVec<[&'a Element<'a>; 4]>> {
    let mut elements: smallvec::SmallVec<[&'a Element<'a>; 4]> = smallvec::SmallVec::new();
    let mut c = Combinator::EMPTY;
    loop {
      let elem_index = self.inp.i;
      let e = match self.inp.re(m::mixin_element) {
        Some(e) => e,
        None => break,
      };
      elements.push(self.a.element(
        c,
        ElValue::Str(e),
        false,
        self.pos(elem_index),
        VisInfo::default(),
      ));
      c = if self.inp.char(b'>') {
        Combinator::new(">")
      } else {
        Combinator::EMPTY
      };
    }
    if elements.is_empty() {
      None
    } else {
      Some(elements)
    }
  }

  /// `mixin.args(isCall)` → (args, variadic)
  fn mixin_args(&mut self, is_call: bool) -> LResult<(Vec<MixinArg<'a>>, bool)> {
    let mut variadic = false;
    let mut expressions: Vec<Node<'a>> = Vec::new();
    let mut args_semi: Vec<MixinArg<'a>> = Vec::new();
    let mut args_comma: Vec<MixinArg<'a>> = Vec::new();
    let mut is_semi = false;
    let mut expression_contains_named = false;
    let mut name: Option<&'a str> = None;
    let mut expand = false;
    let mut has_sep = true;
    self.inp.save();
    loop {
      let arg: Option<Node<'a>>;
      if is_call {
        let mut a = self.detached_ruleset()?;
        if a.is_none() {
          a = self.expression()?;
        }
        arg = a;
      } else {
        self.inp.comment_store.clear();
        if self.inp.str("...") {
          variadic = true;
          if self.inp.char(b';') && !is_semi {
            is_semi = true;
          }
          let v = MixinArg {
            name: None,
            value: None,
            expand: false,
            variadic: true,
          };
          if is_semi {
            args_semi.push(v);
          } else {
            args_comma.push(v);
          }
          break;
        }
        let mut a = self.entities_variable()?;
        if a.is_none() {
          a = self.entities_property();
        }
        if a.is_none() {
          a = self.literal()?;
        }
        if a.is_none() {
          a = self.keyword();
        }
        if a.is_none() {
          a = self.mixin_call(true, None)?;
        }
        arg = a;
      }
      let mut arg = match arg {
        Some(a) if has_sep => a,
        _ => break,
      };
      let mut name_loop: Option<&'a str> = None;
      if let Node::Expression(e) = arg
        && e.value.iter().any(|v| v.is_comment())
      {
        let filtered: Vec<Node<'a>> = e
          .value
          .iter()
          .copied()
          .filter(|v| !v.is_comment())
          .collect();
        let ne = self.a.alloc(Expression {
          vis: Vis::default(),
          value: self.a.slice(&filtered),
          no_spacing: Cell::new(e.no_spacing.get()),
          parens: e.parens,
          parens_in_op: Cell::new(e.parens_in_op.get()),
          pos: e.pos,
        });
        arg = Node::Expression(ne);
      }
      let mut value: Option<Node<'a>> = Some(arg);
      let val: Option<Node<'a>> = if is_call {
        match arg {
          Node::Expression(e) if e.value.len() == 1 => Some(e.value[0]),
          _ => None,
        }
      } else {
        Some(arg)
      };
      let val_name = match val {
        Some(Node::Variable(v)) => Some(v.name),
        Some(Node::Property(p)) => Some(p.name),
        _ => None,
      };
      if let Some(vname) = val_name {
        if self.inp.char(b':') {
          if !expressions.is_empty() {
            if is_semi {
              return self.error("Cannot mix ; and , as delimiter types", None);
            }
            expression_contains_named = true;
          }
          let mut v = self.detached_ruleset()?;
          if v.is_none() {
            v = self.expression()?;
          }
          match v {
            Some(v) => value = Some(v),
            None => {
              if is_call {
                return self.error("could not understand value for named argument", None);
              } else {
                self.inp.restore();
                return Ok((Vec::new(), variadic));
              }
            }
          }
          name = Some(vname);
          name_loop = Some(vname);
        } else if self.inp.str("...") {
          if !is_call {
            variadic = true;
            if self.inp.char(b';') && !is_semi {
              is_semi = true;
            }
            let v = MixinArg {
              name: Some(vname),
              value: None,
              expand: false,
              variadic: true,
            };
            if is_semi {
              args_semi.push(v);
            } else {
              args_comma.push(v);
            }
            break;
          } else {
            expand = true;
          }
        } else if !is_call {
          name = Some(vname);
          name_loop = Some(vname);
          value = None;
        }
      }
      if let Some(v) = value {
        expressions.push(v);
      }
      args_comma.push(MixinArg {
        name: name_loop,
        value,
        expand,
        variadic: false,
      });
      if self.inp.char(b',') {
        has_sep = true;
        continue;
      }
      has_sep = self.inp.char(b';');
      if has_sep || is_semi {
        if expression_contains_named {
          return self.error("Cannot mix ; and , as delimiter types", None);
        }
        is_semi = true;
        let mut v = value;
        if expressions.len() > 1 {
          v = Some(self.a.value(&expressions));
        }
        args_semi.push(MixinArg {
          name,
          value: v,
          expand,
          variadic: false,
        });
        name = None;
        expressions.clear();
        expression_contains_named = false;
      }
    }
    self.inp.forget();
    Ok((if is_semi { args_semi } else { args_comma }, variadic))
  }

  fn mixin_definition(&mut self) -> PR<'a> {
    let index = self.inp.i;
    let c = self.inp.cur();
    if (c != b'.' && c != b'#') || m::no_brace_before_close(self.inp.rest_bytes()) {
      return Ok(None);
    }
    self.inp.save();
    if let Some((len, name_len)) = m::mixin_def_start(self.inp.rest_bytes()) {
      let t = self.inp.consume(len);
      let name = &t[..name_len];
      let (params, variadic) = self.mixin_args(false)?;
      if !self.inp.char(b')') {
        self.inp.restore_msg(Some("Missing closing ')'"));
        return Ok(None);
      }
      self.inp.comment_store.clear();
      let mut cond = None;
      if self.inp.str("when") {
        cond = self.conditions()?;
        if cond.is_none() {
          return self.error("expected condition", None);
        }
      }
      let ruleset = self.block()?;
      if let Some(rules) = ruleset {
        self.inp.forget();
        self.warn_dash_only_mixin_name(name, index);
        return Ok(Some(self.make_mixin_definition(
          Some(name),
          &params,
          rules,
          cond,
          variadic,
          false,
        )));
      }
      self.inp.restore();
    } else {
      self.inp.restore();
    }
    Ok(None)
  }

  pub(crate) fn make_mixin_definition(
    &self,
    name: Option<&'a str>,
    params: &[MixinArg<'a>],
    rules: smallvec::SmallVec<[Node<'a>; 16]>,
    condition: Option<Node<'a>>,
    variadic: bool,
    _rules_is_ruleset: bool,
  ) -> Node<'a> {
    make_mixin_definition(
      self.a,
      name,
      params,
      self.a.vec_from(&rules),
      condition,
      variadic,
      None,
      VisInfo::default(),
    )
  }

  fn rule_lookups(&mut self) -> Option<Vec<&'a str>> {
    if self.inp.cur() != b'[' {
      return None;
    }
    let mut lookups = Vec::new();
    loop {
      self.inp.save();
      match self.lookup_value() {
        Some(r) => {
          lookups.push(r);
          self.inp.forget();
        }
        None => {
          self.inp.restore();
          break;
        }
      }
    }
    if lookups.is_empty() {
      None
    } else {
      Some(lookups)
    }
  }

  fn lookup_value(&mut self) -> Option<&'a str> {
    self.inp.save();
    if !self.inp.char(b'[') {
      self.inp.restore();
      return None;
    }
    let n = m::lookup_value(self.inp.rest_bytes());
    let name = self.inp.consume(n);
    if !self.inp.char(b']') {
      self.inp.restore();
      return None;
    }
    self.inp.forget();
    Some(name)
  }

  // ------------------------------------------------------------------
  // entity / end / ieAlpha
  // ------------------------------------------------------------------

  fn entity(&mut self) -> PR<'a> {
    if let Some(c) = self.comment() {
      return Ok(Some(c));
    }
    if let Some(n) = self.literal()? {
      return Ok(Some(n));
    }
    if let Some(n) = self.entities_variable()? {
      return Ok(Some(n));
    }
    if let Some(n) = self.url()? {
      return Ok(Some(n));
    }
    if let Some(n) = self.entities_property() {
      return Ok(Some(n));
    }
    if let Some(n) = self.entities_call()? {
      return Ok(Some(n));
    }
    if let Some(n) = self.keyword() {
      return Ok(Some(n));
    }
    if let Some(n) = self.mixin_call(true, None)? {
      return Ok(Some(n));
    }
    self.javascript()
  }

  fn end(&mut self) -> bool {
    self.inp.char(b';') || self.inp.peek_char(b'}')
  }

  fn ie_alpha(&mut self) -> PR<'a> {
    let rest = self.inp.rest_bytes();
    if !(rest.len() >= 8 && rest[..8].eq_ignore_ascii_case(b"opacity=")) {
      return Ok(None);
    }
    self.inp.consume(8);
    let value: String = match self.inp.re(m::digits) {
      Some(d) => d.to_string(),
      None => match self.entities_variable()? {
        Some(Node::Variable(v)) => format!("@{{{}}}", &v.name[1..]),
        Some(_) => String::new(),
        None => return self.error("Could not parse alpha", None),
      },
    };
    self.expect_char(b')', None)?;
    let text = self.a.str(&format!("alpha(opacity={})", value));
    Ok(Some(Node::Quoted(self.a.quoted("", text, true, Pos::NONE))))
  }

  // ------------------------------------------------------------------
  // selectors
  // ------------------------------------------------------------------

  fn element(&mut self) -> LResult<Option<&'a Element<'a>>> {
    self.enter()?;
    let r = if self.depth.is_multiple_of(8) {
      crate::eval::grow(|| self.element_inner())
    } else {
      self.element_inner()
    };
    self.leave();
    r
  }

  fn element_inner(&mut self) -> LResult<Option<&'a Element<'a>>> {
    let index = self.inp.i;
    let c = self.combinator();
    let mut e: Option<ElValue<'a>> = None;
    if let Some(t) = self.inp.re(m::percentage_element) {
      e = Some(ElValue::Str(t));
    } else if let Some(t) = self.inp.re(m::element_name) {
      e = Some(ElValue::Str(t));
    } else if self.inp.char(b'*') {
      e = Some(ElValue::Str("*"));
    } else if self.inp.char(b'&') {
      e = Some(ElValue::Str("&"));
    } else if let Some(attr) = self.attribute()? {
      e = Some(ElValue::Node(attr));
    } else if let Some(t) = self.inp.re(m::simple_paren) {
      e = Some(ElValue::Str(t));
    } else if let Some(t) = self.inp.re(m::sigil_before_var) {
      e = Some(ElValue::Str(t));
    } else if let Some(v) = self.variable_curly()? {
      e = Some(ElValue::Node(v));
    }
    if e.is_none() {
      self.inp.save();
      if self.inp.char(b'(') {
        if let Some(mut v) = self.selector(false)? {
          let mut selectors: Vec<Node<'a>> = Vec::new();
          let mut last: Option<&'a Selector<'a>> = Some(v);
          while self.inp.char(b',') {
            if let Some(l) = last {
              selectors.push(Node::Selector(l));
            }
            selectors.push(self.a.anonymous(",", Pos::NONE));
            last = self.selector(false)?;
            if let Some(l) = last {
              v = l;
            }
          }
          if let Some(l) = last {
            selectors.push(Node::Selector(l));
          }
          if self.inp.char(b')') {
            if selectors.len() > 1 {
              let list = self.a.alloc(Selector::new(
                self.a,
                &[],
                Some(crate::tree::SelAux::List(self.a.slice(&selectors))),
                None,
                Pos::NONE,
                VisInfo::default(),
              ));
              e = Some(ElValue::Node(self.a.paren(Node::Selector(list))));
            } else {
              e = Some(ElValue::Node(self.a.paren(Node::Selector(v))));
            }
            self.inp.forget();
          } else {
            self.inp.restore_msg(Some("Missing closing ')'"));
          }
        } else {
          self.inp.restore_msg(Some("Missing closing ')'"));
        }
      } else {
        self.inp.forget();
      }
    }
    match e {
      Some(e) => {
        let is_var = matches!(e, ElValue::Node(Node::Variable(_)));
        Ok(Some(self.a.element(
          c,
          e,
          is_var,
          self.pos(index),
          VisInfo::default(),
        )))
      }
      None => Ok(None),
    }
  }

  fn combinator(&mut self) -> Combinator<'a> {
    let c = self.inp.cur();
    if c == b'/' {
      self.inp.save();
      if let Some(s) = self.inp.re(m::slashed_combinator) {
        self.inp.forget();
        return Combinator::new_in(self.a, s);
      }
      self.inp.restore();
    }
    if matches!(c, b'>' | b'+' | b'~' | b'|' | b'^') {
      let start = self.inp.i;
      self.inp.i += 1;
      if c == b'^' && self.inp.cur() == b'^' {
        self.inp.i += 1;
      }
      let v = &self.inp.input[start..self.inp.i];
      while self.inp.is_whitespace(0) {
        self.inp.i += 1;
      }
      Combinator::new(v)
    } else if self.inp.is_whitespace(-1) {
      Combinator::SPACE
    } else {
      Combinator::EMPTY
    }
  }

  pub(crate) fn selector(&mut self, is_less: bool) -> LResult<Option<&'a Selector<'a>>> {
    let index = self.inp.i;
    let mut elements: smallvec::SmallVec<[&'a Element<'a>; 8]> = smallvec::SmallVec::new();
    let mut all_extends: Option<Vec<&'a Extend<'a>>> = None;
    let mut condition: Option<Node<'a>> = None;
    let mut c: u8 = 0;
    let mut when = false;
    loop {
      let mut extend_list = None;
      let mut e = None;
      let mut matched = false;
      if is_less {
        extend_list = self.extend(false)?;
        if extend_list.is_some() {
          matched = true;
        }
      }
      if !matched && is_less {
        when = self.inp.str("when");
        if when {
          matched = true;
        }
      }
      if !matched {
        e = self.element()?;
        if e.is_some() {
          matched = true;
        }
      }
      if !matched {
        break;
      }
      if when {
        condition = self.conditions()?;
        if condition.is_none() {
          return self.error("expected condition", None);
        }
      } else if condition.is_some() {
        return self.error("CSS guard can only be used at the end of selector", None);
      } else if let Some(el) = extend_list {
        all_extends.get_or_insert_with(Vec::new).extend(el);
      } else {
        if all_extends.is_some() {
          return self.error("Extend can only be used at the end of selector", None);
        }
        c = self.inp.cur();
        if let Some(e) = e {
          elements.push(e);
        }
      }
      if matches!(c, b'{' | b'}' | b';' | b',' | b')') {
        break;
      }
    }
    if !elements.is_empty() {
      let ext = all_extends.map(|v| self.a.slice(&v));
      return Ok(Some(self.a.selector(
        &elements,
        ext,
        condition,
        self.pos(index),
        VisInfo::default(),
      )));
    }
    if all_extends.is_some() {
      return self.error(
        "Extend must be used to extend a selector, it cannot be used on its own",
        None,
      );
    }
    Ok(None)
  }

  fn selectors(&mut self) -> LResult<Option<smallvec::SmallVec<[&'a Selector<'a>; 4]>>> {
    let mut selectors: smallvec::SmallVec<[&'a Selector<'a>; 4]> = smallvec::SmallVec::new();
    loop {
      let s = match self.selector(true)? {
        Some(s) => s,
        None => break,
      };
      selectors.push(s);
      self.inp.comment_store.clear();
      if s.condition().is_some() && selectors.len() > 1 {
        return self.error(
          "Guards are only currently allowed on a single selector.",
          None,
        );
      }
      if !self.inp.char(b',') {
        break;
      }
      if s.condition().is_some() {
        return self.error(
          "Guards are only currently allowed on a single selector.",
          None,
        );
      }
      self.inp.comment_store.clear();
    }
    Ok(if selectors.is_empty() {
      None
    } else {
      Some(selectors)
    })
  }

  fn attribute(&mut self) -> PR<'a> {
    if !self.inp.char(b'[') {
      return Ok(None);
    }
    let key = match self.variable_curly()? {
      Some(k) => StrOrNode::Node(k),
      None => match self.inp.re(m::attribute_key) {
        Some(k) => StrOrNode::Str(k),
        None => return self.error("unexpected token", None),
      },
    };
    let op = self.inp.re(m::attribute_op);
    let mut val = None;
    let mut cif = None;
    if op.is_some() {
      if let Some(q) = self.quoted(false)? {
        val = Some(StrOrNode::Node(q));
      } else if let Some(t) = self.inp.re(m::int_percent) {
        val = Some(StrOrNode::Str(t));
      } else if let Some(t) = self.inp.re(m::word_dash) {
        val = Some(StrOrNode::Str(t));
      } else if let Some(v) = self.variable_curly()? {
        val = Some(StrOrNode::Node(v));
      }
      if val.is_some() {
        cif = self.inp.re(|s| {
          if matches!(s.first(), Some(b'i' | b'I' | b's' | b'S')) {
            Some(1)
          } else {
            None
          }
        });
      }
    }
    self.expect_char(b']', None)?;
    Ok(Some(Node::Attribute(self.a.alloc(Attribute {
      key,
      op,
      value: val,
      cif,
    }))))
  }

  fn block(&mut self) -> LResult<Option<smallvec::SmallVec<[Node<'a>; 16]>>> {
    self.enter()?;
    let r = if self.depth.is_multiple_of(8) {
      crate::eval::grow(|| self.block_inner())
    } else {
      self.block_inner()
    };
    self.leave();
    r
  }

  fn block_inner(&mut self) -> LResult<Option<smallvec::SmallVec<[Node<'a>; 16]>>> {
    if self.inp.char(b'{') {
      let content = self.primary()?;
      if self.inp.char(b'}') {
        return Ok(Some(content));
      }
    }
    Ok(None)
  }

  fn block_ruleset(&mut self) -> LResult<Option<&'a Ruleset<'a>>> {
    Ok(
      self
        .block()?
        .map(|b| self.a.ruleset(None, Some(self.a.vec_from(&b)))),
    )
  }

  fn detached_ruleset(&mut self) -> PR<'a> {
    self.inp.save();
    let mut params = None;
    if matches!(self.inp.rest_bytes(), [b'.' | b'#', b'(', ..]) {
      self.inp.consume(2);
      let (p, variadic) = self.mixin_args(false)?;
      if !self.inp.char(b')') {
        self.inp.restore();
        return Ok(None);
      }
      params = Some((p, variadic));
    }
    if let Some(rs) = self.block_ruleset()? {
      self.inp.forget();
      if let Some((p, variadic)) = params {
        let mut v = self.a.vec();
        v.push(Node::Ruleset(rs));
        return Ok(Some(make_mixin_definition(
          self.a,
          None,
          &p,
          v,
          None,
          variadic,
          None,
          VisInfo::default(),
        )));
      }
      return Ok(Some(self.a.detached_ruleset(Node::Ruleset(rs), None)));
    }
    self.inp.restore();
    Ok(None)
  }

  fn ruleset(&mut self) -> PR<'a> {
    self.inp.save();
    let debug = if self.dump_line_numbers {
      Some(self.debug_info(self.inp.i))
    } else {
      None
    };
    let selectors = self.selectors()?;
    if let Some(selectors) = selectors
      && let Some(rules) = self.block()?
    {
      self.inp.forget();
      let rs = self.a.ruleset_full(
        Some(self.a.slice(&selectors)),
        Some(self.a.vec_from(&rules)),
        self.strict_imports,
        VisInfo::default(),
        None,
        Pos::NONE,
      );
      if let Some(d) = debug {
        rs.set_debug_info(self.a, Some(self.a.alloc(d)));
      }
      return Ok(Some(Node::Ruleset(rs)));
    }
    self.inp.restore();
    Ok(None)
  }

  fn declaration(&mut self) -> PR<'a> {
    let index = self.inp.i;
    let c = self.inp.cur();
    if matches!(c, b'.' | b'#' | b'&' | b':') {
      return Ok(None);
    }
    self.inp.save();
    let var_name = self.variable();
    let mut prop_name: Option<PropName<'a>> = None;
    if var_name.is_none() {
      prop_name = self.rule_property_name()?;
    }
    if var_name.is_none() && prop_name.is_none() {
      self.inp.restore();
      return Ok(None);
    }
    let is_variable = var_name.is_some();
    let mut value: Option<Node<'a>> = None;
    let mut has_dr = false;
    let mut important: Option<&'a str> = None;
    let mut merge = Merge::None;
    if is_variable {
      value = self.detached_ruleset()?;
      if value.is_some() {
        has_dr = true;
      }
    }
    self.inp.comment_store.clear();
    if value.is_none() {
      if let Some(PropName::Parts(name)) = prop_name.as_mut()
        && name.len() > 1
        && let Some(Node::Keyword(k)) = name.pop()
      {
        merge = match k.value {
          "+" => Merge::Comma,
          "+_" => Merge::Space,
          _ => Merge::None,
        };
      }
      let is_custom = match prop_name.as_ref() {
        Some(PropName::Simple(s)) => s.starts_with("--"),
        Some(PropName::Parts(n)) => {
          matches!(n.first(), Some(Node::Keyword(k)) if k.value.starts_with("--"))
        }
        None => false,
      };
      if is_custom {
        if self.inp.char(b';') {
          value = Some(self.a.anonymous("", Pos::NONE));
        } else {
          value = self.permissive_value(UntilTok::SemiOrClose, false)?;
        }
      } else {
        value = self.anonymous_value();
      }
      if let Some(v) = value {
        self.inp.forget();
        return Ok(Some(
          self.make_declaration(var_name, prop_name, v, None, merge, index),
        ));
      }
      value = self.value()?;
      if value.is_some() {
        important = self.important();
      } else if is_variable {
        value = self.permissive_value(UntilTok::Semi, false)?;
      }
    }
    if let Some(v) = value
      && (self.end() || has_dr)
    {
      self.inp.forget();
      return Ok(Some(
        self.make_declaration(var_name, prop_name, v, important, merge, index),
      ));
    }
    self.inp.restore();
    Ok(None)
  }

  fn make_declaration(
    &mut self,
    var_name: Option<&'a str>,
    prop_name: Option<PropName<'a>>,
    value: Node<'a>,
    important: Option<&'a str>,
    merge: Merge,
    index: usize,
  ) -> Node<'a> {
    // a plain property name is stored as the string (less.js keeps a
    // one-keyword array, which every consumer reads the same way)
    let name: &'a DeclName<'a> = match (var_name, prop_name) {
      (Some(n), _) => self.decl_name(n),
      (None, Some(PropName::Simple(s))) => self.decl_name(s),
      (None, Some(PropName::Parts(p))) => self.a.alloc(DeclName::Parts(self.a.slice(&p))),
      (None, None) => &DeclName::Parts(&[]),
    };
    let imp = important_str(self.a, important.unwrap_or(""));
    Node::Declaration(self.a.declaration_ref(
      name,
      value,
      imp,
      merge,
      self.pos(index),
      false,
      Some(var_name.is_some()),
    ))
  }

  fn anonymous_value(&mut self) -> Option<Node<'a>> {
    let index = self.inp.i;
    let len = m::anonymous_value(self.inp.rest_bytes())?;
    let text = self.inp.consume(len + 1);
    Some(self.a.anonymous(&text[..len], self.pos(index)))
  }

  fn permissive_value(&mut self, until: UntilTok, deprecate_variables: bool) -> PR<'a> {
    let index = self.inp.i;
    let test = |c: u8, at_end: bool| -> bool {
      if at_end {
        return false;
      }
      match until {
        UntilTok::Semi => c == b';',
        UntilTok::SemiOrClose => c == b';' || c == b'}',
        UntilTok::OpenOrSemi => c == b'{' || c == b';',
      }
    };
    let at_end = |s: &Self| s.inp.i >= s.inp.bytes.len();
    if test(self.inp.cur(), at_end(self)) {
      return Ok(None);
    }
    let mut value: Vec<Node<'a>> = Vec::new();
    let mut result: Vec<Node<'a>> = Vec::new();
    let mut last_comment: Option<(Pos, usize)> = None;
    // Divergence: less.js re-absorbs the same comments forever when a
    // failed entity attempt keeps backtracking over them (and runs out of
    // memory); stop when a comment comes back without progress.
    let again = |last: Option<(Pos, usize)>, c: Node<'a>, i: usize| {
      last.is_some_and(|(p, li)| c.pos().0 <= p.0 && i <= li)
    };
    loop {
      self.tick()?;
      if let Some(c) = self.comment() {
        if again(last_comment, c, self.inp.i) {
          break;
        }
        last_comment = Some((c.pos(), self.inp.i));
        value.push(c);
        continue;
      }
      let e;
      if deprecate_variables {
        let mut x = self.variable_curly()?;
        if x.is_none() {
          let var_index = self.inp.i;
          x = self.entity()?;
          if Self::is_bare_variable_reference(&x) {
            self.warn_bare_at_rule_variable(var_index);
          }
        }
        e = x;
      } else {
        e = self.entity()?;
      }
      if let Some(n @ Node::Comment(_)) = e {
        if again(last_comment, n, self.inp.i) {
          break;
        }
        last_comment = Some((n.pos(), self.inp.i));
      }
      if let Some(n) = e {
        value.push(n);
      }
      if self.inp.peek_char(b',') {
        value.push(self.a.anonymous(",", self.pos(self.inp.i)));
        self.inp.char(b',');
      }
      if e.is_none() {
        break;
      }
    }
    let done = test(self.inp.cur(), at_end(self));
    if !value.is_empty() {
      let expr = Node::Expression(self.a.expression(&value, None));
      if done {
        return Ok(Some(expr));
      }
      result.push(expr);
      if self.inp.prev() == b' ' {
        result.push(self.a.anonymous(" ", self.pos(index)));
      }
    }
    self.inp.save();
    let parsed = self
      .inp
      .parse_until(|c| test(c, false), deprecate_variables);
    match parsed {
      Some(ParseUntil::Unexpected(c)) => {
        return self.error(format!("Expected '{}'", c), Some("Parse"));
      }
      Some(ParseUntil::Groups(groups, bare_var_index)) => {
        if groups.len() == 1
          && let UntilGroup::Text(" ") = groups[0]
        {
          self.inp.forget();
          return Ok(Some(self.a.anonymous("", self.pos(index))));
        }
        if deprecate_variables && let Some(b) = bare_var_index {
          self.warn_bare_at_rule_variable(b);
        }
        let n = groups.len();
        for (i, g) in groups.into_iter().enumerate() {
          match g {
            UntilGroup::Quoted(q, s) => {
              let qs = self.a.str(&q.to_string());
              result.push(Node::Quoted(self.a.quoted(qs, s, true, self.pos(index))));
            }
            UntilGroup::Text(mut item) => {
              if i == n - 1 {
                item = trim_js(item);
              }
              let bare = strip_interpolations(item);
              if !deprecate_variables && has_bare_at_word(&bare, b'@') {
                self.warn(
                                    "@variable in unknown values will not be evaluated as variables in the future. Use @{variable}",
                                    Some(index),
                                    Some("DEPRECATED"),
                                    Some("variable-in-unknown-value"),
                                );
              }
              if has_bare_at_word(&bare, b'$') {
                self.warn(
                                    "$property in unknown values will not be evaluated as property references in the future. Use ${property}",
                                    Some(index),
                                    Some("DEPRECATED"),
                                    Some("property-in-unknown-value"),
                                );
              }
              let q = self.a.alloc(Quoted {
                vis: Vis::default(),
                quote: "'",
                value: item,
                escaped: true,
                pos: self.pos(index),
                permissive: true,
              });
              result.push(Node::Quoted(q));
            }
          }
        }
        self.inp.forget();
        return Ok(Some(Node::Expression(
          self.a.expression(&result, Some(true)),
        )));
      }
      None => {}
    }
    self.inp.restore();
    Ok(None)
  }

  // ------------------------------------------------------------------
  // @import, media features, at-rules
  // ------------------------------------------------------------------

  fn import(&mut self) -> PR<'a> {
    let index = self.inp.i;
    if self.inp.re(m::at_import).is_none() {
      return Ok(None);
    }
    let options = self.import_options()?.unwrap_or_default();
    let mut path = self.quoted(false)?;
    if path.is_none() {
      path = self.url()?;
    }
    if let Some(path) = path {
      let features = self.media_features(false)?;
      if !self.inp.char(b';') {
        self.inp.i = index;
        return self.error(
          "missing semi-colon or unrecognised media features on import",
          None,
        );
      }
      let features = features.map(|f| self.a.value(&f));
      return Ok(Some(Node::Import(make_import(
        self.a,
        path,
        features,
        options,
        self.pos(index),
        VisInfo::default(),
      ))));
    }
    self.inp.i = index;
    self.error("malformed import statement", None)
  }

  fn import_options(&mut self) -> LResult<Option<ImportOptions>> {
    if !self.inp.char(b'(') {
      return Ok(None);
    }
    let mut options = ImportOptions::default();
    loop {
      let o = self.inp.re(m::import_option);
      match o {
        Some(o) => {
          match o {
            "css" => options.less = Some(false),
            "once" => options.multiple = Some(false),
            "less" => options.less = Some(true),
            "multiple" => options.multiple = Some(true),
            "inline" => options.inline = true,
            "reference" => options.reference = true,
            "optional" => options.optional = true,
            _ => {}
          }
          if !self.inp.char(b',') {
            break;
          }
        }
        None => break,
      }
    }
    self.expect_char(b')', None)?;
    Ok(Some(options))
  }

  fn css_keyword_node(&mut self) -> Option<Node<'a>> {
    let k = self.inp.re(m::css_keyword)?;
    Some(color_from_keyword(self.a, k).unwrap_or_else(|| self.a.keyword(k)))
  }

  fn media_feature(&mut self, query_in_parens: bool) -> PR<'a> {
    self.enter()?;
    let r = if self.depth.is_multiple_of(8) {
      crate::eval::grow(|| self.media_feature_inner(query_in_parens))
    } else {
      self.media_feature_inner(query_in_parens)
    };
    self.leave();
    r
  }

  fn media_feature_inner(&mut self, query_in_parens: bool) -> PR<'a> {
    let mut nodes: Vec<Node<'a>> = Vec::new();
    let mut spacing = false;
    self.inp.save();
    loop {
      self.inp.save();
      if m::css_keyword_space_paren(self.inp.rest_bytes()) {
        spacing = true;
      }
      self.inp.restore();
      let mut e = self.declaration_call()?;
      if e.is_none() {
        e = self.css_keyword_node();
      }
      if e.is_none() {
        e = self.keyword();
      }
      if e.is_none() {
        e = self.variable_curly()?;
      }
      if e.is_none() {
        let var_index = self.inp.i;
        let bare = self.entities_variable()?;
        if bare.is_some() {
          self.warn_bare_at_rule_variable(var_index);
          e = bare;
        } else {
          e = self.mixin_lookup()?;
        }
      }
      if let Some(n) = e {
        nodes.push(n);
        let sp = match n {
          Node::Variable(_) => true,
          Node::Keyword(k) => matches!(
            k.value.cow_to_ascii_lowercase().as_ref(),
            "and" | "or" | "not" | "only"
          ),
          _ => false,
        };
        if sp {
          spacing = true;
        }
      } else if self.inp.char(b'(') {
        let mut closed = false;
        let mut p: Option<PropOrCond<'a>> = self.property().map(PropOrCond::Prop);
        let mut range_p: Option<&'a Condition<'a>> = None;
        self.inp.save();
        if p.is_none() && query_in_parens && m::range_query_ahead(self.inp.rest_bytes()) {
          self.inp.restore();
          let c = self.condition(false, false)?;
          let c = match c {
            Some(Node::Condition(c)) => c,
            _ => return self.error("expected condition", None),
          };
          p = Some(PropOrCond::Cond(c));
          self.inp.save();
          match self.atomic_condition(false, Some(c.rvalue), false)? {
            Some(Node::Condition(r)) => range_p = Some(r),
            _ => self.inp.restore(),
          }
        } else {
          self.inp.restore();
          self.inp.save();
          e = self.value()?;
          if e.is_some() && self.inp.char(b')') {
            closed = true;
            self.inp.forget();
          } else {
            self.inp.restore();
            e = self.media_feature(query_in_parens)?;
          }
        }
        if !closed && self.inp.char(b')') {
          closed = true;
        }
        if closed {
          match (p, e) {
            (Some(PropOrCond::Cond(c)), None) => {
              let q = self.a.alloc(QueryInParens {
                op: c.op,
                lvalue: c.lvalue,
                mvalue: c.rvalue,
                op2: range_p.map(|r| r.op),
                rvalue: range_p.map(|r| r.rvalue),
                pos: c.pos,
              });
              let paren = self.a.paren(Node::QueryInParens(q));
              if !spacing {
                paren.set_no_spacing(Some(true));
              }
              nodes.push(paren);
              e = Some(Node::Condition(c));
            }
            (Some(PropOrCond::Prop(pname)), Some(val)) => {
              let d = self.a.declaration(
                DeclName::Str(pname),
                val,
                "",
                Merge::None,
                self.pos(self.inp.i),
                true,
                None,
              );
              let paren = self.a.paren(Node::Declaration(d));
              if !spacing {
                paren.set_no_spacing(Some(true));
              }
              nodes.push(paren);
              spacing = false;
            }
            (Some(PropOrCond::Cond(c)), Some(val)) => {
              // `p && e` with a condition: less.js builds a
              // Declaration named by the condition object.
              let _ = c;
              let paren = self.a.paren(val);
              nodes.push(paren);
              spacing = false;
            }
            (None, Some(val)) => {
              let paren = self.a.paren(val);
              if !spacing {
                paren.set_no_spacing(Some(true));
              }
              nodes.push(paren);
              spacing = false;
            }
            (Some(PropOrCond::Prop(_)), None) | (None, None) => {
              return self.error("badly formed media feature definition", None);
            }
          }
        } else {
          return self.error("Missing closing ')'", Some("Parse"));
        }
      }
      if e.is_none() {
        break;
      }
    }
    self.inp.forget();
    if !nodes.is_empty() {
      let expr = self.a.expression(&nodes, None);
      if nodes.len() == 2
        && matches!(nodes[0], Node::Keyword(_) | Node::Variable(_))
        && matches!(nodes[1], Node::Paren(p) if p.no_spacing.get() == Some(true))
      {
        expr.no_spacing.set(Some(true));
      }
      return Ok(Some(Node::Expression(expr)));
    }
    Ok(None)
  }

  fn media_features(&mut self, query_in_parens: bool) -> LResult<Option<Vec<Node<'a>>>> {
    let mut features: Vec<Node<'a>> = Vec::new();
    loop {
      let mut e = self.media_feature(query_in_parens)?;
      if let Some(n) = e {
        features.push(n);
        if !self.inp.char(b',') {
          break;
        } else if n.no_spacing() != Some(true) {
          n.set_no_spacing(Some(false));
        }
      } else {
        e = self.variable_curly()?;
        if e.is_none() {
          let var_index = self.inp.i;
          let bare = self.entities_variable()?;
          if bare.is_some() {
            self.warn_bare_at_rule_variable(var_index);
            e = bare;
          } else {
            e = self.mixin_lookup()?;
          }
        }
        if let Some(n) = e {
          features.push(n);
          if !self.inp.char(b',') {
            break;
          } else if n.no_spacing() != Some(true) {
            n.set_no_spacing(Some(false));
          }
        }
      }
      if e.is_none() {
        break;
      }
    }
    Ok(if features.is_empty() {
      None
    } else {
      Some(features)
    })
  }

  fn prepare_nestable_at_rule(
    &mut self,
    kind: AtRuleKind,
    index: usize,
    debug: Option<DebugInfo<'a>>,
  ) -> PR<'a> {
    let features = self.media_features(true)?;
    let rules = match self.block()? {
      Some(r) => r,
      None => {
        return self.error(
          "media definitions require block statements after any features",
          None,
        );
      }
    };
    self.inp.forget();
    let n = make_media(
      self.a,
      kind,
      rules.to_vec(),
      features.unwrap_or_default(),
      self.pos(index),
      VisInfo::default(),
      debug,
    );
    Ok(Some(n))
  }

  fn nestable_at_rule(&mut self) -> PR<'a> {
    let index = self.inp.i;
    let debug = if self.dump_line_numbers {
      Some(self.debug_info(index))
    } else {
      None
    };
    self.inp.save();
    if self.inp.peek_char(b'@') {
      if self.inp.str("@media") {
        return self.prepare_nestable_at_rule(AtRuleKind::Media, index, debug);
      }
      if self.inp.str("@container") {
        return self.prepare_nestable_at_rule(AtRuleKind::Container, index, debug);
      }
    }
    self.inp.restore();
    Ok(None)
  }

  fn plugin(&mut self) -> PR<'a> {
    let index = self.inp.i;
    if self.disable_plugin_rule {
      // `/^@plugin?\s+/`
      let r = self.inp.rest_bytes();
      let n = if r.starts_with(b"@plugin") {
        7
      } else if r.starts_with(b"@plugi") {
        6
      } else {
        0
      };
      if n > 0 && r.get(n).map(|&b| m::is_space(b)).unwrap_or(false) {
        return self.error(
          "@plugin statements are not allowed when disablePluginRule is set to true",
          None,
        );
      }
      return Ok(None);
    }
    if self.inp.re(m::at_plugin).is_none() {
      return Ok(None);
    }
    self.warn(
            "The @plugin directive is deprecated and will be replaced in Less 5.x. Use --plugin CLI option or the programmatic plugin API instead.",
            Some(index),
            Some("DEPRECATED"),
            Some("at-plugin"),
        );
    // pluginArgs
    self.inp.save();
    if self.inp.char(b'(') {
      if let Some((len, _, _)) = m::plugin_args(self.inp.rest_bytes()) {
        self.inp.consume(len);
        self.inp.forget();
      } else {
        self.inp.restore();
      }
    } else {
      self.inp.restore();
    }
    let mut path = self.quoted(false)?;
    if path.is_none() {
      path = self.url()?;
    }
    if let Some(path) = path {
      if !self.inp.char(b';') {
        self.inp.i = index;
        return self.error("missing semi-colon on @plugin", None);
      }
      let options = ImportOptions {
        is_plugin: true,
        ..Default::default()
      };
      return Ok(Some(Node::Import(make_import(
        self.a,
        path,
        None,
        options,
        self.pos(index),
        VisInfo::default(),
      ))));
    }
    self.inp.i = index;
    self.error("malformed @plugin statement", None)
  }

  fn at_rule_entity(&mut self) -> PR<'a> {
    if let Some(c) = self.variable_curly()? {
      return Ok(Some(c));
    }
    let index = self.inp.i;
    let e = self.entity()?;
    if Self::is_bare_variable_reference(&e) {
      self.warn_bare_at_rule_variable(index);
    }
    Ok(e)
  }

  fn atrule_unknown(&mut self, name: &str) -> LResult<(Option<Node<'a>>, bool)> {
    let mut value = self.permissive_value(UntilTok::OpenOrSemi, true)?;
    let has_block = self.inp.cur() == b'{';
    match value {
      None => {
        if !has_block && self.inp.cur() != b';' {
          return self.error(
            format!("{} rule is missing block or ending semi-colon", name),
            None,
          );
        }
      }
      Some(v) => {
        // `!value.value`: an Anonymous('') has a falsy value.
        if let Node::Anonymous(a) = v
          && a.value.is_empty()
        {
          value = None;
        }
      }
    }
    Ok((value, has_block))
  }

  fn atrule_block(
    &mut self,
    mut value: AtValue<'a>,
    is_rooted: bool,
  ) -> LResult<(Option<&'a Ruleset<'a>>, AtValue<'a>, bool)> {
    let mut is_keyword_list = false;
    let mut rules = self.block_ruleset()?;
    self.inp.save();
    if rules.is_none() && !is_rooted {
      value = AtValue::One(self.at_rule_entity()?);
      rules = self.block_ruleset()?;
    }
    if rules.is_none() && !is_rooted {
      self.inp.restore();
      let mut e: Vec<Node<'a>> = Vec::new();
      let mut v = self.at_rule_entity()?;
      while self.inp.char(b',') {
        if let Some(x) = v {
          e.push(x);
        }
        v = self.at_rule_entity()?;
      }
      if v.is_some() && !e.is_empty() {
        e.push(v.expect("at-rule value is present"));
        value = AtValue::List(e);
        is_keyword_list = true;
      } else {
        value = AtValue::One(v);
        rules = self.block_ruleset()?;
      }
    } else {
      self.inp.forget();
    }
    Ok((rules, value, is_keyword_list))
  }

  fn atrule(&mut self) -> PR<'a> {
    let index = self.inp.i;
    if self.inp.cur() != b'@' {
      return Ok(None);
    }
    if let Some(v) = self.import()? {
      return Ok(Some(v));
    }
    if let Some(v) = self.plugin()? {
      return Ok(Some(v));
    }
    if let Some(v) = self.nestable_at_rule()? {
      return Ok(Some(v));
    }
    self.inp.save();
    let mut name = match self.inp.re(m::at_rule_name) {
      Some(n) => n,
      None => return Ok(None),
    };
    let mut non_vendor = name.to_string();
    if name.as_bytes().get(1) == Some(&b'-')
      && let Some(p) = name[2..].find('-')
    {
      non_vendor = format!("@{}", &name[2 + p + 1..]);
    }
    let mut has_identifier = false;
    let mut has_expression = false;
    let mut has_unknown = false;
    let mut has_block = true;
    let mut is_rooted = true;
    match non_vendor.as_str() {
      "@charset" => {
        has_identifier = true;
        has_block = false;
      }
      "@namespace" => {
        has_expression = true;
        has_block = false;
      }
      "@keyframes" | "@counter-style" => has_identifier = true,
      "@document" | "@supports" => {
        has_unknown = true;
        is_rooted = false;
      }
      "@starting-style" | "@layer" => is_rooted = false,
      _ => has_unknown = true,
    }
    self.inp.comment_store.clear();
    let mut value = AtValue::One(None);
    if has_identifier {
      let v = self.at_rule_entity()?;
      if v.is_none() {
        return self.error(format!("expected {} identifier", name), None);
      }
      if non_vendor == "@charset" {
        self.warn_dynamic_charset(v.expect("charset value was parsed"), index);
      }
      value = AtValue::One(v);
    } else if has_expression {
      let mut prefix = self.variable_curly()?;
      if prefix.is_none() && m::variable_name(self.inp.rest_bytes()).is_some() {
        let prefix_index = self.inp.i;
        prefix = self.entities_variable()?;
        if prefix.is_some() {
          self.warn_bare_at_rule_variable(prefix_index);
        }
      }
      let mut v = self.expression()?;
      if let Some(p) = prefix {
        v = Some(match v {
          Some(Node::Expression(e)) => {
            let mut items = vec![p];
            items.extend_from_slice(e.value);
            Node::Expression(self.a.expression(&items, None))
          }
          Some(other) => other,
          None => p,
        });
      }
      if v.is_none() {
        return self.error(format!("expected {} expression", name), None);
      }
      value = AtValue::One(v);
    } else if has_unknown {
      let (v, hb) = self.atrule_unknown(name)?;
      value = AtValue::One(v);
      has_block = hb;
    }
    let mut rules = None;
    let mut is_keyword_list = false;
    if has_block {
      let (r, v, k) = self.atrule_block(value, is_rooted)?;
      rules = r;
      value = v;
      is_keyword_list = k;
      if rules.is_none() && !has_unknown {
        self.inp.restore();
        name = self.inp.re(m::at_rule_name).unwrap_or(name);
        let (v, hb) = self.atrule_unknown(name)?;
        value = AtValue::One(v);
        has_block = hb;
        if has_block {
          let (r, v, k) = self.atrule_block(value, is_rooted)?;
          rules = r;
          value = v;
          is_keyword_list = k;
        }
      }
    }
    let value_present = match &value {
      AtValue::One(v) => v.is_some(),
      AtValue::List(l) => !l.is_empty(),
    };
    if rules.is_some() || is_keyword_list || (!has_block && value_present && self.inp.char(b';')) {
      self.inp.forget();
      let debug = if self.dump_line_numbers {
        Some(self.debug_info(index))
      } else {
        None
      };
      let value_node = match value {
        AtValue::One(v) => v,
        AtValue::List(l) => Some(Node::Rules(self.a.rules(&l))),
      };
      return Ok(Some(make_at_rule_ext(
        self.a,
        name,
        value_node,
        rules.map(AtRules::Single),
        self.pos(index),
        debug,
        is_rooted,
        VisInfo::default(),
      )));
    }
    self.inp.restore_msg(Some("at-rule options not recognised"));
    Ok(None)
  }

  fn warn_dynamic_charset(&mut self, value: Node<'a>, index: usize) {
    let dynamic = match value {
      Node::Variable(_) => true,
      Node::Quoted(q) => {
        let b = q.value.as_bytes();
        (0..b.len()).any(|i| b[i] == b'@' && m::interpolation_at(b, i).is_some())
          || (0..b.len()).any(|i| b[i] == b'$' && m::prop_interpolation_at(b, i).is_some())
      }
      _ => false,
    };
    if dynamic {
      self.warn(
                "Dynamic @charset interpolation is deprecated and will be removed in Less 5.x. Use a static quoted encoding declaration instead.",
                Some(index),
                Some("DEPRECATED"),
                Some("dynamic-charset"),
            );
    }
  }

  // ------------------------------------------------------------------
  // values, operations, conditions
  // ------------------------------------------------------------------

  pub(crate) fn value(&mut self) -> PR<'a> {
    let index = self.inp.i;
    let mut expressions: smallvec::SmallVec<[Node<'a>; 4]> = smallvec::SmallVec::new();
    loop {
      let e = self.expression()?;
      match e {
        Some(e) => {
          expressions.push(e);
          if !self.inp.char(b',') {
            break;
          }
        }
        None => break,
      }
    }
    if !expressions.is_empty() {
      return Ok(Some(Node::Value(self.a.alloc(Value {
        vis: Vis::default(),
        value: self.a.slice(&expressions),
        pos: self.pos(index),
      }))));
    }
    Ok(None)
  }

  pub(crate) fn important(&mut self) -> Option<&'a str> {
    if self.inp.cur() == b'!' {
      return self.inp.re(m::important);
    }
    None
  }

  fn sub(&mut self) -> PR<'a> {
    self.enter()?;
    let r = if self.depth.is_multiple_of(8) {
      crate::eval::grow(|| self.sub_inner())
    } else {
      self.sub_inner()
    };
    self.leave();
    r
  }

  fn sub_inner(&mut self) -> PR<'a> {
    self.inp.save();
    if self.inp.char(b'(') {
      let a = self.addition()?;
      if let Some(a) = a
        && self.inp.char(b')')
      {
        self.inp.forget();
        let e = self.a.alloc(Expression {
          vis: Vis::default(),
          value: self.a.slice(&[a]),
          no_spacing: Cell::new(None),
          parens: true,
          parens_in_op: Cell::new(false),
          pos: Pos::NONE,
        });
        return Ok(Some(Node::Expression(e)));
      }
      self.inp.restore_msg(Some("Expected ')'"));
      return Ok(None);
    }
    self.inp.restore();
    Ok(None)
  }

  fn color_operand(&mut self) -> Option<Node<'a>> {
    self.inp.save();
    if let Some(mt) = self.inp.re(m::color_operand) {
      self.inp.forget();
      return Some(self.a.keyword(&mt[..1]));
    }
    self.inp.restore();
    None
  }

  fn set_parens_in_op(n: Node<'a>) {
    if let Node::Expression(e) = n {
      e.parens_in_op.set(true);
    }
  }

  fn multiplication(&mut self) -> PR<'a> {
    let m0 = match self.operand()? {
      Some(m) => m,
      None => return Ok(None),
    };
    let mut is_spaced = self.inp.is_whitespace(-1);
    let mut operation: Option<Node<'a>> = None;
    loop {
      if matches!(self.inp.rest_bytes(), [b'/', b'*' | b'/', ..]) {
        break;
      }
      self.inp.save();
      let mut op: Option<&'a str> = None;
      if self.inp.char(b'/') {
        op = Some("/");
      } else if self.inp.char(b'*') {
        op = Some("*");
      } else {
        let index = self.inp.i;
        if self.inp.str("./") {
          op = Some("./");
          self.warn(
            "./ operator is deprecated",
            Some(index),
            Some("DEPRECATED"),
            Some("dot-slash-operator"),
          );
        }
      }
      let op = match op {
        Some(o) => o,
        None => {
          self.inp.forget();
          break;
        }
      };
      let a = match self.operand()? {
        Some(a) => a,
        None => {
          self.inp.restore();
          break;
        }
      };
      self.inp.forget();
      Self::set_parens_in_op(m0);
      Self::set_parens_in_op(a);
      operation = Some(Node::Operation(self.a.alloc(Operation {
        op,
        operands: [operation.unwrap_or(m0), a],
        is_spaced,
      })));
      is_spaced = self.inp.is_whitespace(-1);
    }
    Ok(Some(operation.unwrap_or(m0)))
  }

  fn addition(&mut self) -> PR<'a> {
    let m0 = match self.multiplication()? {
      Some(m) => m,
      None => return Ok(None),
    };
    let mut is_spaced = self.inp.is_whitespace(-1);
    let mut operation: Option<Node<'a>> = None;
    loop {
      let mut op: Option<&'a str> = self.inp.re(m::spaced_add_op).map(|s| &s[..1]);
      if op.is_none() && !is_spaced {
        if self.inp.char(b'+') {
          op = Some("+");
        } else if self.inp.char(b'-') {
          op = Some("-");
        }
      }
      let op = match op {
        Some(o) => o,
        None => break,
      };
      let a = match self.multiplication()? {
        Some(a) => a,
        None => break,
      };
      Self::set_parens_in_op(m0);
      Self::set_parens_in_op(a);
      operation = Some(Node::Operation(self.a.alloc(Operation {
        op,
        operands: [operation.unwrap_or(m0), a],
        is_spaced,
      })));
      is_spaced = self.inp.is_whitespace(-1);
    }
    Ok(Some(operation.unwrap_or(m0)))
  }

  fn conditions(&mut self) -> PR<'a> {
    let index = self.inp.i;
    let a = match self.condition(true, false)? {
      Some(a) => a,
      None => return Ok(None),
    };
    let mut condition: Option<Node<'a>> = None;
    loop {
      if !m::comma_condition_ahead(self.inp.rest_bytes()) || !self.inp.char(b',') {
        break;
      }
      let b = match self.condition(true, false)? {
        Some(b) => b,
        None => break,
      };
      condition =
        Some(self.make_condition("or", condition.unwrap_or(a), b, self.pos(index), false));
    }
    Ok(Some(condition.unwrap_or(a)))
  }

  fn make_condition(
    &self,
    op: &'a str,
    l: Node<'a>,
    r: Node<'a>,
    pos: Pos,
    negate: bool,
  ) -> Node<'a> {
    Node::Condition(self.a.alloc(Condition {
      op,
      lvalue: l,
      rvalue: r,
      pos,
      negate: Cell::new(negate),
    }))
  }

  fn condition(&mut self, needs_parens: bool, allow_condition_operands: bool) -> PR<'a> {
    let result = match self.condition_and(needs_parens, allow_condition_operands)? {
      Some(r) => r,
      None => return Ok(None),
    };
    if self.inp.str("or") {
      let next = self.condition(needs_parens, allow_condition_operands)?;
      match next {
        Some(n) => return Ok(Some(self.make_condition("or", result, n, Pos::NONE, false))),
        None => return Ok(None),
      }
    }
    Ok(Some(result))
  }

  fn condition_and(&mut self, needs_parens: bool, allow_condition_operands: bool) -> PR<'a> {
    let mut result = self.negated_condition(needs_parens, allow_condition_operands)?;
    if result.is_none() {
      result = self.parenthesis_condition(needs_parens, allow_condition_operands)?;
    }
    if result.is_none() && !needs_parens {
      result = self.atomic_condition(needs_parens, None, false)?;
    }
    let mut result = match result {
      Some(r) => r,
      None => return Ok(None),
    };
    if allow_condition_operands
      && let Some(r) =
        self.atomic_condition(needs_parens, Some(result), allow_condition_operands)?
    {
      result = r;
    }
    if self.inp.str("and") {
      let next = self.condition_and(needs_parens, allow_condition_operands)?;
      match next {
        Some(n) => {
          return Ok(Some(self.make_condition(
            "and",
            result,
            n,
            Pos::NONE,
            false,
          )));
        }
        None => return Ok(None),
      }
    }
    Ok(Some(result))
  }

  fn negated_condition(&mut self, needs_parens: bool, allow_condition_operands: bool) -> PR<'a> {
    if self.inp.str("not") {
      if let Some(r) = self.parenthesis_condition(needs_parens, allow_condition_operands)? {
        if let Node::Condition(c) = r {
          c.negate.set(!c.negate.get());
        }
        return Ok(Some(r));
      }
      let index = self.inp.i;
      let mut a = self.keyword();
      if a.is_none() {
        a = self.entities_variable()?;
      }
      if a.is_none() {
        a = self.quoted(false)?;
      }
      if a.is_none() {
        a = self.mixin_lookup()?;
      }
      if let Some(a) = a {
        let t = self.a.keyword("true");
        return Ok(Some(self.make_condition("=", a, t, self.pos(index), true)));
      }
    }
    Ok(None)
  }

  fn parenthesis_condition(
    &mut self,
    needs_parens: bool,
    allow_condition_operands: bool,
  ) -> PR<'a> {
    self.enter()?;
    let r = if self.depth.is_multiple_of(8) {
      crate::eval::grow(|| self.parenthesis_condition_inner(needs_parens, allow_condition_operands))
    } else {
      self.parenthesis_condition_inner(needs_parens, allow_condition_operands)
    };
    self.leave();
    r
  }

  fn parenthesis_condition_inner(
    &mut self,
    needs_parens: bool,
    allow_condition_operands: bool,
  ) -> PR<'a> {
    self.inp.save();
    if !self.inp.str("(") {
      self.inp.restore();
      return Ok(None);
    }
    // tryConditionFollowedByParenthesis
    self.inp.save();
    let body = self.condition(needs_parens, allow_condition_operands)?;
    let mut found = None;
    match body {
      None => self.inp.restore(),
      Some(b) => {
        if !self.inp.char(b')') {
          self.inp.restore();
        } else {
          self.inp.forget();
          found = Some(b);
        }
      }
    }
    if let Some(b) = found {
      self.inp.forget();
      return Ok(Some(b));
    }
    let body = match self.atomic_condition(needs_parens, None, false)? {
      Some(b) => b,
      None => {
        self.inp.restore();
        return Ok(None);
      }
    };
    if !self.inp.char(b')') {
      let msg = format!("expected ')' got '{}'", self.inp.cur_str());
      self.inp.restore_msg(Some(&msg));
      return Ok(None);
    }
    self.inp.forget();
    Ok(Some(body))
  }

  fn atomic_condition(
    &mut self,
    needs_parens: bool,
    preparsed: Option<Node<'a>>,
    allow_condition_operands: bool,
  ) -> PR<'a> {
    let index = self.inp.i;
    let a = match preparsed {
      Some(p) => Some(p),
      None => self.cond_operand(needs_parens, allow_condition_operands)?,
    };
    let a = match a {
      Some(a) => a,
      None => return Ok(None),
    };
    let mut op: Option<&'static str> = None;
    if self.inp.char(b'>') {
      op = Some(if self.inp.char(b'=') { ">=" } else { ">" });
    } else if self.inp.char(b'<') {
      op = Some(if self.inp.char(b'=') { "<=" } else { "<" });
    } else if self.inp.char(b'=') {
      op = Some(if self.inp.char(b'>') {
        "=>"
      } else if self.inp.char(b'<') {
        "=<"
      } else {
        "="
      });
    }
    if let Some(op) = op {
      let b = self.cond_operand(needs_parens, allow_condition_operands)?;
      match b {
        Some(b) => return Ok(Some(self.make_condition(op, a, b, self.pos(index), false))),
        None => return self.error("expected expression", None),
      }
    } else if preparsed.is_none() {
      let t = self.a.keyword("true");
      return Ok(Some(self.make_condition("=", a, t, self.pos(index), false)));
    }
    Ok(None)
  }

  fn cond_operand(&mut self, needs_parens: bool, allow_condition_operands: bool) -> PR<'a> {
    if allow_condition_operands && let Some(c) = self.parenthesis_condition(needs_parens, false)? {
      return Ok(Some(c));
    }
    if let Some(a) = self.addition()? {
      return Ok(Some(a));
    }
    if let Some(k) = self.keyword() {
      return Ok(Some(k));
    }
    if let Some(q) = self.quoted(false)? {
      return Ok(Some(q));
    }
    self.mixin_lookup()
  }

  fn operand(&mut self) -> PR<'a> {
    let mut negate = false;
    if matches!(self.inp.rest_bytes(), [b'-', b'@' | b'$' | b'(', ..]) {
      negate = self.inp.char(b'-');
    }
    let mut o = self.sub()?;
    if o.is_none() {
      o = self.dimension();
    }
    if o.is_none() {
      o = self.color();
    }
    if o.is_none() {
      o = self.entities_variable()?;
    }
    if o.is_none() {
      o = self.entities_property();
    }
    if o.is_none() {
      o = self.entities_call()?;
    }
    if o.is_none() {
      o = self.quoted(true)?;
    }
    if o.is_none() {
      o = self.color_keyword();
    }
    if o.is_none() {
      o = self.color_operand();
    }
    if o.is_none() {
      o = self.mixin_lookup()?;
    }
    if negate {
      match o {
        Some(n) => {
          Self::set_parens_in_op(n);
          o = Some(Node::Negative(self.a.alloc(Negative { value: n })));
        }
        None => {
          return Err(LessError::new(
            "Syntax",
            "Cannot set properties of undefined (setting 'parensInOp')",
          ));
        }
      }
    }
    Ok(o)
  }

  pub(crate) fn expression(&mut self) -> PR<'a> {
    let index = self.inp.i;
    let mut entities: smallvec::SmallVec<[Node<'a>; 8]> = smallvec::SmallVec::new();
    let mut seen_comments: smallvec::SmallVec<[(Pos, usize); 4]> = smallvec::SmallVec::new();
    loop {
      if let Some(c) = self.comment()
        && let Node::Comment(cm) = c
        && !cm.is_line_comment
      {
        // Divergence: less.js re-absorbs the same comments
        // forever when a failed entity attempt backtracks
        // over them (and runs out of memory); stop when a
        // comment comes back without progress.
        let key = (cm.pos, self.inp.i);
        if seen_comments.contains(&key) {
          break;
        }
        seen_comments.push(key);
        entities.push(c);
        continue;
      }
      let mut e = self.addition()?;
      if e.is_none() {
        e = self.entity()?;
      }
      if let Some(Node::Comment(_)) = e {
        e = None;
      }
      match e {
        Some(n) => {
          entities.push(n);
          if !matches!(self.inp.rest_bytes(), [b'/', b'/' | b'*', ..]) && self.inp.char(b'/') {
            entities.push(self.a.anonymous("/", self.pos(index)));
          }
        }
        None => break,
      }
    }
    if !entities.is_empty() {
      return Ok(Some(Node::Expression(self.a.expression(&entities, None))));
    }
    Ok(None)
  }

  fn property(&mut self) -> Option<&'a str> {
    let (len, name_len) = m::property(self.inp.rest_bytes())?;
    let t = self.inp.consume(len);
    Some(&t[..name_len])
  }

  fn rule_property(&mut self) -> LResult<Option<smallvec::SmallVec<[Node<'a>; 4]>>> {
    Ok(match self.rule_property_name()? {
      Some(PropName::Simple(s)) => Some(smallvec::smallvec![self.a.keyword(s)]),
      Some(PropName::Parts(p)) => Some(p),
      None => None,
    })
  }

  /// `rule_property` without materializing a plain name as a keyword.
  fn rule_property_name(&mut self) -> LResult<Option<PropName<'a>>> {
    self.inp.save();
    if let Some((len, name_len)) = m::simple_property(self.inp.rest_bytes()) {
      let t = self.inp.consume(len);
      self.inp.forget();
      return Ok(Some(PropName::Simple(&t[..name_len])));
    }
    let mut name: smallvec::SmallVec<[&'a str; 8]> = smallvec::SmallVec::new();
    let mut index: smallvec::SmallVec<[usize; 8]> = smallvec::SmallVec::new();
    // match(/^(\*?)/)
    {
      let i = self.inp.i;
      let n = if self.inp.cur() == b'*' { 1 } else { 0 };
      let t = self.inp.consume(n);
      index.push(i);
      name.push(t);
    }
    loop {
      let i = self.inp.i;
      match self.inp.re(m::rule_property_particle) {
        Some(t) => {
          index.push(i);
          name.push(t);
        }
        None => break,
      }
    }
    if name.len() > 1 {
      let i = self.inp.i;
      if let Some((len, mlen)) = m::merge_colon(self.inp.rest_bytes()) {
        let t = self.inp.consume(len);
        index.push(i);
        name.push(&t[..mlen]);
        self.inp.forget();
        if name[0].is_empty() {
          name.remove(0);
          index.remove(0);
        }
        let mut out = smallvec::SmallVec::new();
        for (k, s) in name.iter().enumerate() {
          let sigil = s.as_bytes().first().copied();
          if sigil != Some(b'@') && sigil != Some(b'$') {
            out.push(self.a.keyword(s));
            continue;
          }
          let raw = &s[2..s.len() - 1];
          if sigil == Some(b'@') {
            let (n, _) = m::split_lookups(raw);
            self.warn_numeric_variable_name(n, index[k]);
            self.warn_dash_only_variable_name(n, index[k]);
            out.push(resolve_interpolated_variable(
              self.a,
              raw,
              self.pos(index[k]),
            ));
          } else {
            out.push(resolve_interpolated_property(
              self.a,
              raw,
              self.pos(index[k]),
            ));
          }
        }
        return Ok(Some(PropName::Parts(out)));
      }
    }
    self.inp.restore();
    Ok(None)
  }
}

#[derive(Copy, Clone)]
enum UntilTok {
  Semi,
  SemiOrClose,
  OpenOrSemi,
}

enum AtValue<'a> {
  One(Option<Node<'a>>),
  List(Vec<Node<'a>>),
}

enum PropOrCond<'a> {
  Prop(&'a str),
  Cond(&'a Condition<'a>),
}

fn is_js_space(b: u8) -> bool {
  matches!(b, b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c)
}

/// Removes `@{...}` and `${...}` interpolations from `text`.
fn strip_interpolations(text: &str) -> String {
  let s = text.as_bytes();
  let mut out = String::with_capacity(text.len());
  let mut i = 0;
  let mut last = 0;
  while i < s.len() {
    if s[i] == b'@'
      && let Some(end) = m::interpolation_at(s, i)
    {
      out.push_str(&text[last..i]);
      i = end;
      last = end;
      continue;
    }
    i += 1;
  }
  out.push_str(&text[last..]);
  let s2 = out.clone();
  let b = s2.as_bytes();
  let mut out2 = String::with_capacity(out.len());
  let mut i = 0;
  let mut last = 0;
  while i < b.len() {
    if b[i] == b'$'
      && let Some(end) = m::prop_interpolation_at(b, i)
    {
      out2.push_str(&s2[last..i]);
      i = end;
      last = end;
      continue;
    }
    i += 1;
  }
  out2.push_str(&s2[last..]);
  out2
}

/// `/@([\w-]+)/` (or `$`) test.
fn has_bare_at_word(text: &str, sigil: u8) -> bool {
  let b = text.as_bytes();
  (0..b.len()).any(|i| b[i] == sigil && b.get(i + 1).map(|&c| m::is_word_dash(c)).unwrap_or(false))
}

/// `resolveInterpolatedVariable(raw, index, fileInfo)`.
pub(crate) fn resolve_interpolated_variable<'a>(a: Alloc<'a>, raw: &str, pos: Pos) -> Node<'a> {
  let (name, lookups) = m::split_lookups(raw);
  let mut s = bumpalo::collections::String::with_capacity_in(name.len() + 1, a.bump);
  s.push('@');
  s.push_str(name);
  let var_name: &'a str = s.into_bump_str();
  match lookups {
    None => a.variable(var_name, pos),
    Some(l) => {
      let call = Node::VariableCall(a.alloc(VariableCall {
        vis: Vis::default(),
        variable: var_name,
        pos,
      }));
      let l: Vec<&'a str> = l.iter().map(|s| a.str(s)).collect();
      Node::NamespaceValue(a.alloc(NamespaceValue {
        value: call,
        lookups: a.slice(&l),
        pos,
      }))
    }
  }
}

pub(crate) fn resolve_interpolated_property<'a>(a: Alloc<'a>, raw: &str, pos: Pos) -> Node<'a> {
  Node::Property(a.alloc(Property {
    name: a.str(&format!("${}", raw)),
    pos,
    evaluating: Cell::new(false),
  }))
}

/// `new tree.mixin.Definition(name, params, rules, condition, variadic, frames, visibilityInfo)`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn make_mixin_definition<'a>(
  a: Alloc<'a>,
  name: Option<&'a str>,
  params: &[MixinArg<'a>],
  rules: BVec<'a, Node<'a>>,
  condition: Option<Node<'a>>,
  variadic: bool,
  frames: Option<&'a [&'a Ruleset<'a>]>,
  vis: VisInfo,
) -> Node<'a> {
  let display_name = name.filter(|n| !n.is_empty()).unwrap_or("anonymous mixin");
  let mut optional: Vec<&'a str> = Vec::new();
  let mut required = 0;
  for p in params {
    if p.name.is_none() || p.value.is_none() {
      required += 1;
    } else {
      optional.push(p.name.expect("optional parameter has a name"));
    }
  }
  let def = a.alloc(MixinDef {
    name: display_name,
    params: a.slice(params),
    condition,
    variadic,
    arity: params.len(),
    required,
    optional_parameters: a.slice(&optional),
    frames,
  });
  // `new Element(null, name, ...)`: a null name yields an empty element.
  let el = a.element(
    Combinator::EMPTY,
    ElValue::Str(name.unwrap_or("")),
    false,
    Pos::NONE,
    VisInfo::default(),
  );
  let sel = a.selector(&[el], None, None, Pos::NONE, VisInfo::default());
  let rs = a.ruleset_full(
    Some(a.slice(&[sel])),
    Some(rules),
    false,
    vis,
    Some(def),
    Pos::NONE,
  );
  Node::MixinDefinition(rs)
}

/// `new tree.Import(path, features, options, index, fileInfo)`.
pub(crate) fn make_import<'a>(
  a: Alloc<'a>,
  path: Node<'a>,
  features: Option<Node<'a>>,
  options: ImportOptions,
  pos: Pos,
  vis: VisInfo,
) -> &'a Import<'a> {
  let imp = a.alloc(Import {
    vis: Vis::new(vis),
    path,
    features,
    options,
    pos,
    css: Cell::new(None),
    layer_css: Cell::new(false),
    root: Cell::new(None),
    imported_filename: Cell::new(None),
    imported_file: Cell::new(None),
    skip: Cell::new(Skip::No),
    error: Cell::new(None),
    replacement: Cell::new(None),
    features_override: Cell::new(None),
    path_reference: Cell::new(false),
  });
  if options.less.is_some() || options.inline {
    imp
      .css
      .set(Some(!options.less.unwrap_or(false) || options.inline));
  } else if let Some(p) = imp.get_path()
    && is_css_path(p)
  {
    imp.css.set(Some(true));
  }
  imp
}

/// `/[#.&?]css([?;].*)?$/`
fn is_css_path(p: &str) -> bool {
  let b = p.as_bytes();
  for i in 0..b.len() {
    if matches!(b[i], b'#' | b'.' | b'&' | b'?') && b[i + 1..].starts_with(b"css") {
      let rest = &b[i + 4..];
      if rest.is_empty() || matches!(rest[0], b'?' | b';') && !rest.contains(&b'\n') {
        return true;
      }
    }
  }
  false
}

/// `new Media(rules, features, index, fileInfo)` / `new Container(...)`.
pub(crate) fn make_media<'a>(
  a: Alloc<'a>,
  kind: AtRuleKind,
  rules: Vec<Node<'a>>,
  features: Vec<Node<'a>>,
  pos: Pos,
  vis: VisInfo,
  debug: Option<DebugInfo<'a>>,
) -> Node<'a> {
  let selectors = a.empty_selectors(pos);
  let body = a.ruleset(Some(selectors), Some(a.vec_from(&rules)));
  body.allow_imports.set(true);
  let at = a.alloc(AtRule {
    kind,
    vis: Vis::new(vis),
    name: if kind == AtRuleKind::Media {
      "@media"
    } else {
      "@container"
    },
    value: None,
    features: Cell::new(Some(a.value(&features))),
    rules: Cell::new(Some(a.slice(&[Node::Ruleset(body)]))),
    declarations: Cell::new(None),
    simple_block: false,
    is_rooted: false,
    debug_info: debug,
    pos,
    evaluated: false,
  });
  at.node()
}

fn declarations_block(rules: &[Node<'_>], mergeable: bool) -> bool {
  rules.iter().all(|n| match n {
    Node::Declaration(d) => mergeable || d.merge.get() == Merge::None,
    Node::Comment(_) => true,
    _ => false,
  })
}

/// `new AtRule(name, value, rules, index, fileInfo, debugInfo, isRooted, visibilityInfo)`.
///
/// `rules` is either a list (`Some(vec)`) or a single ruleset passed as a
/// one-element list holding `Node::Ruleset` with `single = true` semantics
/// when it comes from the parser.
#[allow(clippy::too_many_arguments)]
pub(crate) fn make_at_rule<'a>(
  a: Alloc<'a>,
  name: &'a str,
  value: Option<Node<'a>>,
  rules: Option<Vec<Node<'a>>>,
  pos: Pos,
  debug: Option<DebugInfo<'a>>,
  is_rooted: bool,
  vis: VisInfo,
) -> Node<'a> {
  make_at_rule_ext(
    a,
    name,
    value,
    rules.map(AtRules::List),
    pos,
    debug,
    is_rooted,
    vis,
  )
}

pub(crate) enum AtRules<'a> {
  List(Vec<Node<'a>>),
  Single(&'a Ruleset<'a>),
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn make_at_rule_ext<'a>(
  a: Alloc<'a>,
  name: &'a str,
  value: Option<Node<'a>>,
  rules: Option<AtRules<'a>>,
  pos: Pos,
  debug: Option<DebugInfo<'a>>,
  is_rooted: bool,
  vis: VisInfo,
) -> Node<'a> {
  let mut simple_block = false;
  let mut declarations: Option<&'a [Node<'a>]> = None;
  let mut out_rules: Option<&'a [Node<'a>]> = None;
  match rules {
    Some(AtRules::List(list)) => {
      let all_declarations = declarations_block(&list, false);
      let mut all_ruleset_declarations = true;
      for r in &list {
        if let Node::Ruleset(rs) = r
          && rs.has_rules()
        {
          let rv = rs.rules_vec();
          all_ruleset_declarations = all_ruleset_declarations && declarations_block(&rv, true);
        }
      }
      if all_declarations && !is_rooted {
        simple_block = true;
        declarations = Some(a.slice(&list));
      } else if all_ruleset_declarations && list.len() == 1 && !is_rooted && value.is_none() {
        simple_block = true;
        declarations = Some(match list[0] {
          Node::Ruleset(rs) if rs.has_rules() => a.slice(&rs.rules_vec()),
          _ => a.slice(&list),
        });
      } else {
        out_rules = Some(a.slice(&list));
      }
    }
    Some(AtRules::Single(rs)) => {
      let rv = rs.rules_vec();
      let all_declarations = declarations_block(&rv, false);
      if all_declarations && !is_rooted && value.is_none() {
        simple_block = true;
        declarations = Some(a.slice(&rv));
      } else {
        rs.selectors.set(Some(a.empty_selectors(pos)));
        out_rules = Some(a.slice(&[Node::Ruleset(rs)]));
      }
    }
    None => {}
  }
  if !simple_block && let Some(r) = out_rules {
    for n in r {
      if let Some(rs) = n.as_ruleset() {
        rs.allow_imports.set(true);
      }
    }
  }
  let at = a.alloc(AtRule {
    kind: AtRuleKind::AtRule,
    vis: Vis::new(vis),
    name,
    value,
    features: Cell::new(None),
    rules: Cell::new(out_rules),
    declarations: Cell::new(declarations),
    simple_block,
    is_rooted,
    debug_info: debug,
    pos,
    evaluated: false,
  });
  Node::AtRule(at)
}
