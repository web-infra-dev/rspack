//! CSS generation: ports of the `genCSS` / `toCSS` methods of every node.

use cow_utils::CowUtils;

use crate::{
  color,
  error::{LResult, LessError},
  num,
  options::DumpLineNumbers,
  tree::*,
};

/// The `context` object passed to `genCSS`.
#[derive(Clone, Debug, Default)]
pub struct CssCtx {
  pub compress: bool,
  pub strict_units: bool,
  pub num_precision: Option<u32>,
  pub tab_level: i32,
  pub first_selector: bool,
  /// Output the next selector's first `' '` combinator as `''` (ToCSS's
  /// `p[0].elements[0].combinator = new Combinator('')`, applied lazily).
  pub blank_first_combinator: bool,
  pub last_rule: bool,
  pub dump_line_numbers: Option<DumpLineNumbers>,
  /// Reusable buffer for number formatting.
  pub scratch: String,
  /// Reusable buffers for nested element text (`:not(...)` recurses).
  pub bufs: Vec<String>,
  /// Expected output size (pre-sizes the output buffer).
  pub output_hint: usize,
}

impl CssCtx {
  /// `toCSS({})`
  pub fn empty() -> CssCtx {
    CssCtx::default()
  }
  pub fn fround(&self, v: f64) -> f64 {
    num::fround(v, self.num_precision)
  }
}

/// The `output` object (`{ add, isEmpty }`).
pub trait Output {
  fn add(&mut self, chunk: &str, pos: Option<Pos>, map_lines: bool);
  fn is_empty(&self) -> bool;
}

#[derive(Default)]
pub struct StrOut {
  pub s: String,
  any: bool,
}

impl Output for StrOut {
  #[inline]
  fn add(&mut self, chunk: &str, _pos: Option<Pos>, _map_lines: bool) {
    if !self.any && self.s.capacity() == 0 {
      self.s.reserve(64);
    }
    self.any = true;
    self.s.push_str(chunk);
  }
  fn is_empty(&self) -> bool {
    !self.any
  }
}

#[inline]
fn add(out: &mut dyn Output, s: &str) {
  out.add(s, None, false);
}

#[inline]
fn add_at(out: &mut dyn Output, s: &str, pos: Pos) {
  out.add(s, pos.opt(), false);
}

/// `node.toCSS(context)`
pub fn to_css(n: Node<'_>, ctx: &mut CssCtx) -> LResult<String> {
  let mut o = StrOut::default();
  if let Node::Ruleset(r) = n
    && r.first_root.get()
  {
    o.s.reserve(ctx.output_hint.max(64 * 1024));
  }
  gen_css(n, ctx, &mut o)?;
  Ok(o.s)
}

/// `node.toCSS(context)` appended to `s`.
pub fn append_css(n: Node<'_>, ctx: &mut CssCtx, s: &mut String) -> LResult<()> {
  let mut o = StrOut {
    s: std::mem::take(s),
    any: false,
  };
  let r = gen_css(n, ctx, &mut o);
  *s = o.s;
  r
}

/// `toCSS({})` (used for keys and comparisons).
pub fn to_css_plain(n: Node<'_>) -> LResult<String> {
  to_css(n, &mut CssCtx::empty())
}

pub fn unit_to_string(u: &Unit<'_>) -> String {
  let mut s = u.numerator.join("*");
  for d in u.denominator {
    s.push('/');
    s.push_str(d);
  }
  s
}

pub fn unit_output<'a>(u: &Unit<'a>, strict_units: bool) -> &'a str {
  if u.numerator.len() == 1 {
    u.numerator[0]
  } else if !strict_units && u.backup_unit.is_some() {
    u.backup_unit.expect("backup unit is present")
  } else if !strict_units && !u.denominator.is_empty() {
    u.denominator[0]
  } else {
    ""
  }
}

pub fn unit_is_length(u: &Unit<'_>) -> bool {
  let s = unit_output(u, false).cow_to_ascii_lowercase();
  matches!(
    s.as_ref(),
    "px"
      | "em"
      | "ex"
      | "ch"
      | "rem"
      | "in"
      | "cm"
      | "mm"
      | "pc"
      | "pt"
      | "vw"
      | "vh"
      | "vmin"
      | "vmax"
  )
}

pub fn unit_is_singular(u: &Unit<'_>) -> bool {
  u.numerator.len() <= 1 && u.denominator.is_empty()
}

/// `Dimension.genCSS` into `out` (without strict-units check).
pub fn dimension_write(d: &Dimension<'_>, ctx: &CssCtx, out: &mut String) {
  let value = ctx.fround(d.value);
  if value != 0.0 && value < 0.000001 && value > -0.000001 {
    out.push_str(&dimension_str(d, ctx));
    return;
  }
  let start = out.len();
  num::write_js(out, value);
  if ctx.compress {
    if value == 0.0 && unit_is_length(d.unit) {
      return;
    }
    if value > 0.0 && value < 1.0 {
      out.remove(start);
    }
  }
  out.push_str(unit_output(d.unit, ctx.strict_units));
}

/// `Dimension.genCSS` into a string (without strict-units check).
pub fn dimension_str(d: &Dimension<'_>, ctx: &CssCtx) -> String {
  let value = ctx.fround(d.value);
  let mut s = num::to_js_string(value);
  if value != 0.0 && value < 0.000001 && value > -0.000001 {
    s = num::to_fixed(value, 20);
    while s.ends_with('0') {
      s.pop();
    }
  }
  if ctx.compress {
    if value == 0.0 && unit_is_length(d.unit) {
      return s;
    }
    if value > 0.0 && value < 1.0 {
      s = s[1..].to_string();
    }
  }
  s.push_str(unit_output(d.unit, ctx.strict_units));
  s
}

fn debug_info_str(ctx: &CssCtx, info: Option<DebugInfo<'_>>, line_separator: &str) -> String {
  let info = match info {
    Some(i) => i,
    None => return String::new(),
  };
  if ctx.compress {
    return String::new();
  }
  let as_comment = || format!("/* line {}, {} */\n", info.line_number, info.file_name);
  let as_media = || {
    let mut f = info.file_name.to_string();
    let has_proto = {
      let b = f.as_bytes();
      let n = b.iter().take_while(|c| c.is_ascii_alphabetic()).count();
      n > 0 && b[n..].starts_with(b"://")
    };
    if !has_proto {
      f = format!("file://{}", f);
    }
    let mut esc = String::new();
    for ch in f.chars() {
      match ch {
        '.' | ':' | '/' => {
          esc.push('\\');
          esc.push(ch);
        }
        '\\' => {
          esc.push('\\');
          esc.push('/');
        }
        c => esc.push(c),
      }
    }
    format!(
      "@media -sass-debug-info{{filename{{font-family:{}}}line{{font-family:\\00003{}}}}}\n",
      esc, info.line_number
    )
  };
  match ctx.dump_line_numbers {
    Some(DumpLineNumbers::Comments) => as_comment(),
    Some(DumpLineNumbers::MediaQuery) => as_media(),
    Some(DumpLineNumbers::All) => format!("{}{}{}", as_comment(), line_separator, as_media()),
    None => String::new(),
  }
}

pub fn element_to_css(e: &Element<'_>, ctx: &mut CssCtx) -> LResult<String> {
  element_to_css_with(e, e.combinator.get(), ctx)
}

fn element_to_css_with(e: &Element<'_>, c: Combinator<'_>, ctx: &mut CssCtx) -> LResult<String> {
  let first = ctx.first_selector;
  let value = match e.value {
    ElValue::Str(s) => s.to_string(),
    ElValue::Node(n) => {
      if let Node::Paren(_) = n {
        ctx.first_selector = true;
      }
      let v = to_css(n, ctx)?;
      ctx.first_selector = first;
      v
    }
  };
  ctx.first_selector = first;
  let comb = combinator_css(&c, ctx);
  Ok(format!("{}{}", comb, value))
}

pub fn combinator_css(c: &Combinator<'_>, ctx: &CssCtx) -> String {
  let no_space = ctx.compress || matches!(c.value(), "" | " " | "|");
  if no_space {
    c.value().to_string()
  } else {
    format!(" {} ", c.value())
  }
}

pub fn selector_gen_css(s: &Selector<'_>, ctx: &mut CssCtx, out: &mut dyn Output) -> LResult<()> {
  let blank = std::mem::take(&mut ctx.blank_first_combinator);
  if let Some(list) = s.list() {
    for n in list {
      gen_css(*n, ctx, out)?;
    }
    return Ok(());
  }
  if !ctx.first_selector
    && let Some(first) = s.elements.first()
    && first.combinator.get().value().is_empty()
  {
    add_at(out, " ", s.pos);
  }
  for (i, e) in s.elements.iter().enumerate() {
    let mut c = e.combinator.get();
    if i == 0 && blank && c.value() == " " {
      c = Combinator::EMPTY;
    }
    element_gen(e, c, ctx, out)?;
  }
  Ok(())
}

/// `Element.genCSS`: `output.add(this.toCSS(context), fileInfo, index)`.
/// Emitted as up to three chunks; only the first carries the mapping, which
/// yields the same source map as a single chunk.
fn element_gen(
  e: &Element<'_>,
  c: Combinator<'_>,
  ctx: &mut CssCtx,
  out: &mut dyn Output,
) -> LResult<()> {
  match e.value {
    ElValue::Str(v) => {
      let no_space = ctx.compress || matches!(c.value(), "" | " " | "|");
      let pos = e.pos.opt();
      let mut first = true;
      let mut emit = |s: &str, out: &mut dyn Output| {
        if s.is_empty() {
          return;
        }
        out.add(s, if first { pos } else { None }, false);
        first = false;
      };
      if !no_space {
        emit(" ", out);
      }
      emit(c.value(), out);
      if !no_space {
        emit(" ", out);
      }
      emit(v, out);
      Ok(())
    }
    ElValue::Node(n) => {
      // one chunk, as `output.add(this.toCSS(), ...)`
      let mut s = ctx.bufs.pop().unwrap_or_default();
      s.clear();
      if ctx.compress || matches!(c.value(), "" | " " | "|") {
        s.push_str(c.value());
      } else {
        s.push(' ');
        s.push_str(c.value());
        s.push(' ');
      }
      let first = ctx.first_selector;
      if let Node::Paren(_) = n {
        ctx.first_selector = true;
      }
      let r = append_css(n, ctx, &mut s);
      ctx.first_selector = first;
      if r.is_ok() {
        add_at(out, &s, e.pos);
      }
      ctx.bufs.push(s);
      r?;
      Ok(())
    }
  }
}

pub fn gen_css(n: Node<'_>, ctx: &mut CssCtx, out: &mut dyn Output) -> LResult<()> {
  match n {
    Node::Anonymous(a) => {
      let visible = !a.value.is_empty() && !a.is_null;
      a.vis.set_visible(Some(visible));
      if visible {
        out.add(
          a.value,
          if a.has_file { a.pos.opt() } else { None },
          a.map_lines,
        );
      }
    }
    Node::Assignment(a) => {
      add(out, &format!("{}=", a.key));
      gen_css(a.value, ctx, out)?;
    }
    Node::AtRule(a) => at_rule_gen_css(a, ctx, out)?,
    Node::Media(a) | Node::Container(a) => {
      add_at(
        out,
        if a.kind == AtRuleKind::Media {
          "@media "
        } else {
          "@container "
        },
        a.pos,
      );
      if let Some(f) = a.features.get() {
        gen_css(f, ctx, out)?;
      }
      let rules = a.rules.get().unwrap_or(&[]);
      output_ruleset(ctx, out, rules)?;
    }
    Node::Attribute(a) => {
      // one chunk, as `output.add(this.toCSS())`
      let mut s = std::mem::take(&mut ctx.scratch);
      s.clear();
      attribute_write(a, ctx, &mut s)?;
      add(out, &s);
      ctx.scratch = s;
    }
    Node::Call(c) => {
      add_at(out, c.name, c.pos);
      if c.name.is_empty() {
        add_at(out, "(", c.pos);
      } else {
        add(out, "(");
      }
      for (i, arg) in c.args.iter().enumerate() {
        gen_css(*arg, ctx, out)?;
        if i + 1 < c.args.len() {
          add(out, ", ");
        }
      }
      add(out, ")");
    }
    Node::Color(c) => {
      match c
        .value
        .get()
        .filter(|v| !v.is_empty() && !v.starts_with("rgb") && !v.starts_with("hsl"))
      {
        // keyword / original hex form: output as written
        Some(v) => add(out, v),
        None => {
          let mut s = std::mem::take(&mut ctx.scratch);
          s.clear();
          color::write_css(&mut s, c, ctx, false);
          add(out, &s);
          ctx.scratch = s;
        }
      }
    }
    Node::Comment(c) => {
      if let Some(d) = c.debug_info.get() {
        add_at(out, &debug_info_str(ctx, Some(d), ""), c.pos);
      }
      add(out, c.value);
    }
    Node::Declaration(d) => declaration_gen_css(d, ctx, out)?,
    Node::Dimension(d) => {
      if ctx.strict_units && !unit_is_singular(d.unit) {
        return Err(LessError::native(format!(
          "Multiple units in dimension. Correct the units or use the unit function. Bad unit: {}",
          unit_to_string(d.unit)
        )));
      }
      let mut s = std::mem::take(&mut ctx.scratch);
      s.clear();
      dimension_write(d, ctx, &mut s);
      add(out, &s);
      ctx.scratch = s;
    }
    Node::Element(e) => element_gen(e, e.combinator.get(), ctx, out)?,
    Node::Expression(e) => {
      let ns = e.no_spacing.get() == Some(true);
      for (i, v) in e.value.iter().enumerate() {
        gen_css(*v, ctx, out)?;
        if !ns && i + 1 < e.value.len() {
          let next = e.value[i + 1];
          let is_comma = matches!(next, Node::Anonymous(a) if a.value == ",");
          if !is_comma {
            add(out, " ");
          }
        }
      }
    }
    Node::Import(i) => {
      if i.css.get() == Some(true) && !i.path_reference.get() {
        add_at(out, "@import ", i.pos);
        gen_css(i.path, ctx, out)?;
        if let Some(f) = i.features() {
          add(out, " ");
          gen_css(f, ctx, out)?;
        }
        add(out, ";");
      }
    }
    Node::Keyword(k) => {
      if k.value == "%" {
        return Err(LessError::new("Syntax", "Invalid % without number"));
      }
      add(out, k.value);
    }
    Node::Negative(ng) => {
      add(out, "-");
      gen_css(ng.value, ctx, out)?;
    }
    Node::Operation(o) => {
      gen_css(o.operands[0], ctx, out)?;
      if o.is_spaced {
        add(out, " ");
      }
      add(out, o.op);
      if o.is_spaced {
        add(out, " ");
      }
      gen_css(o.operands[1], ctx, out)?;
    }
    Node::Paren(p) => {
      add(out, "(");
      gen_css(p.value, ctx, out)?;
      add(out, ")");
    }
    Node::QueryInParens(q) => {
      gen_css(q.lvalue, ctx, out)?;
      add(out, " ");
      add(out, q.op);
      add(out, " ");
      gen_css(q.mvalue, ctx, out)?;
      if let Some(r) = q.rvalue {
        add(out, &format!(" {} ", q.op2.unwrap_or("")));
        gen_css(r, ctx, out)?;
      }
    }
    Node::Quoted(q) => {
      if !q.escaped {
        add_at(out, q.quote, q.pos);
      }
      add(out, q.value);
      if !q.escaped {
        add(out, q.quote);
      }
    }
    Node::Ruleset(r) | Node::MixinDefinition(r) => ruleset_gen_css(r, ctx, out)?,
    Node::Selector(s) => selector_gen_css(s, ctx, out)?,
    Node::UnicodeDescriptor(u) => add(out, u.value),
    Node::Url(u) => {
      add(out, "url(");
      gen_css(u.value, ctx, out)?;
      add(out, ")");
    }
    Node::Value(v) => {
      for (i, item) in v.value.iter().enumerate() {
        gen_css(*item, ctx, out)?;
        if i + 1 < v.value.len() {
          add(out, if ctx.compress { "," } else { ", " });
        }
      }
    }
    Node::Rules(list) => {
      // A JS array converted to a string joins its items with ','.
      for (i, item) in list.iter().enumerate() {
        if i > 0 {
          add(out, ",");
        }
        gen_css(*item, ctx, out)?;
      }
    }
    Node::NamespaceValue(_) => add(out, "[object Object]"),
    // Nodes without genCSS output `this.value` (undefined → "").
    Node::Condition(_)
    | Node::DetachedRuleset(_)
    | Node::Extend(_)
    | Node::JavaScript(_)
    | Node::MixinCall(_)
    | Node::Property(_)
    | Node::Variable(_)
    | Node::VariableCall(_) => {}
  }
  Ok(())
}

pub fn attribute_to_css(a: &Attribute<'_>, ctx: &mut CssCtx) -> LResult<String> {
  let mut s = String::new();
  attribute_write(a, ctx, &mut s)?;
  Ok(s)
}

fn attribute_write(a: &Attribute<'_>, ctx: &mut CssCtx, s: &mut String) -> LResult<()> {
  s.push('[');
  match a.key {
    StrOrNode::Str(k) => s.push_str(k),
    StrOrNode::Node(n) => s.push_str(&to_css(n, ctx)?),
  }
  if let Some(op) = a.op {
    s.push_str(op);
    match a.value {
      Some(StrOrNode::Str(v)) => s.push_str(v),
      Some(StrOrNode::Node(n)) => s.push_str(&to_css(n, ctx)?),
      None => s.push_str("undefined"),
    }
  }
  if let Some(c) = a.cif {
    s.push(' ');
    s.push_str(c);
  }
  s.push(']');
  Ok(())
}

pub fn declaration_gen_css(
  d: &Declaration<'_>,
  ctx: &mut CssCtx,
  out: &mut dyn Output,
) -> LResult<()> {
  let name = d.name_str().unwrap_or("");
  let sep = if ctx.compress { ":" } else { ": " };
  if name.is_empty() {
    add_at(out, sep, d.pos);
  } else {
    add_at(out, name, d.pos);
    add(out, sep);
  }
  if let Err(mut e) = gen_css(d.value.get(), ctx, out) {
    e.set_pos(d.pos);
    return Err(e);
  }
  let end = if d.inline || (ctx.last_rule && ctx.compress) {
    ""
  } else {
    ";"
  };
  let imp = d.important();
  if imp.is_empty() {
    add_at(out, end, d.pos);
  } else {
    add_at(out, imp, d.pos);
    add(out, end);
  }
  Ok(())
}

pub fn at_rule_gen_css(a: &AtRule<'_>, ctx: &mut CssCtx, out: &mut dyn Output) -> LResult<()> {
  add_at(out, a.name, a.pos);
  if let Some(v) = a.value {
    add(out, " ");
    gen_css(v, ctx, out)?;
  }
  if a.simple_block {
    output_ruleset(ctx, out, a.declarations.get().unwrap_or(&[]))?;
  } else if let Some(r) = a.rules.get().or(a.declarations.get()) {
    output_ruleset(ctx, out, r)?;
  } else {
    add(out, ";");
  }
  Ok(())
}

pub fn output_ruleset(ctx: &mut CssCtx, out: &mut dyn Output, rules: &[Node<'_>]) -> LResult<()> {
  ctx.tab_level += 1;
  if ctx.compress {
    add(out, "{");
    for r in rules {
      gen_css(*r, ctx, out)?;
    }
    add(out, "}");
    ctx.tab_level -= 1;
    return Ok(());
  }
  let tab_set = format!("\n{}", "  ".repeat((ctx.tab_level - 1).max(0) as usize));
  let tab_rule = format!("{}  ", tab_set);
  if rules.is_empty() {
    add(out, &format!(" {{{}}}", tab_set));
  } else {
    add(out, &format!(" {{{}", tab_rule));
    gen_css(rules[0], ctx, out)?;
    for r in &rules[1..] {
      add(out, &tab_rule);
      gen_css(*r, ctx, out)?;
    }
    add(out, &format!("{}}}", tab_set));
  }
  ctx.tab_level -= 1;
  Ok(())
}

fn is_charset(n: &Node<'_>) -> bool {
  matches!(n, Node::AtRule(a) if a.is_charset())
}

fn tabs(n: i32) -> &'static str {
  const SPACES: &str = "                                                                                                                                ";
  let n = (n.max(0) as usize * 2).min(SPACES.len());
  &SPACES[..n]
}

fn nl_tabs(n: i32) -> &'static str {
  const S: &str = "\n                                                                                                                                ";
  let n = (n.max(0) as usize * 2).min(S.len() - 1);
  &S[..n + 1]
}

pub fn ruleset_gen_css(r: &Ruleset<'_>, ctx: &mut CssCtx, out: &mut dyn Output) -> LResult<()> {
  if !r.root.get() {
    ctx.tab_level += 1;
  }
  let level = ctx.tab_level;
  let rules_ref = r.rules.borrow();
  let rules: &[Node<'_>] = rules_ref.as_deref().unwrap_or(&[]);
  // charsets first, then imports (after leading comments); the common case
  // needs no reordering
  let needs_reorder = rules
    .iter()
    .any(|n| is_charset(n) || matches!(n, Node::Import(_)));
  // (rule indices: no copy of the rules)
  let order: Option<Vec<u32>> = needs_reorder.then(|| {
    let mut v: Vec<u32> = Vec::with_capacity(rules.len());
    let mut charset_index = 0;
    let mut import_index = 0;
    for (i, rule) in rules.iter().enumerate() {
      if rule.is_comment() {
        if import_index == i {
          import_index += 1;
        }
        v.push(i as u32);
      } else if is_charset(rule) {
        v.insert(charset_index, i as u32);
        charset_index += 1;
        import_index += 1;
      } else if matches!(rule, Node::Import(_)) {
        v.insert(import_index.min(v.len()), i as u32);
        import_index += 1;
      } else {
        v.push(i as u32);
      }
    }
    v
  });
  if !r.root.get() {
    let tab_set = if ctx.compress { "" } else { tabs(level - 1) };
    if ctx.dump_line_numbers.is_some() {
      let dbg = debug_info_str(ctx, r.debug_info().copied(), tab_set);
      if !dbg.is_empty() {
        add(out, &dbg);
        add(out, tab_set);
      }
    }
    let paths: &[SelectorPath<'_>] = r.paths.get().unwrap_or(&[]);
    for (i, path) in paths.iter().enumerate() {
      if path.is_empty() {
        continue;
      }
      if i > 0 {
        if ctx.compress {
          add(out, ",");
        } else {
          add(out, ",");
          add(out, nl_tabs(level - 1));
        }
      }
      ctx.first_selector = true;
      ctx.blank_first_combinator = true;
      selector_gen_css(path[0], ctx, out)?;
      ctx.first_selector = false;
      for s in &path[1..] {
        selector_gen_css(s, ctx, out)?;
      }
    }
    if ctx.compress {
      add(out, "{");
    } else {
      add(out, " {\n");
      add(out, tabs(level));
    }
  }
  let n = rules.len();
  for i in 0..n {
    let rule = match &order {
      Some(o) => &rules[o[i] as usize],
      None => &rules[i],
    };
    if i + 1 == n {
      ctx.last_rule = true;
    }
    let current_last_rule = ctx.last_rule;
    if rule.is_ruleset_like() {
      ctx.last_rule = false;
    }
    gen_css(*rule, ctx, out)?;
    ctx.last_rule = current_last_rule;
    if !ctx.last_rule && rule.is_visible() {
      if !ctx.compress {
        add(out, nl_tabs(level));
      }
    } else {
      ctx.last_rule = false;
    }
  }
  if !r.root.get() {
    if ctx.compress {
      add(out, "}");
    } else {
      add(out, nl_tabs(level - 1));
      add(out, "}");
    }
    ctx.tab_level -= 1;
  }
  if !out.is_empty() && !ctx.compress && r.first_root.get() {
    add(out, "\n");
  }
  Ok(())
}
