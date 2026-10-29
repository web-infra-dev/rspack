//! The evaluator: ports of the `eval` methods of `tree/*.js`.

mod atrule;
mod mixin;
pub(crate) mod ruleset;
pub(crate) mod units;

use std::cell::Cell;

use cow_utils::CowUtils;
pub(crate) use ruleset::merge_rules;

use crate::{
  css::{self, CssCtx},
  error::{LResult, LessError},
  num,
  options::Math,
  session::{DefaultState, Session},
  syntax::{
    matchers as m,
    parser::{resolve_interpolated_property, resolve_interpolated_variable},
  },
  tree::*,
};

/// `contexts.Eval`.
pub(crate) struct EvalCtx<'a, 's> {
  pub s: &'s Session<'a>,
  pub a: Alloc<'a>,
  /// `context.frames` (index 0 is the innermost frame).
  pub frames: Vec<&'a Ruleset<'a>>,
  pub media_blocks: Option<Vec<&'a AtRule<'a>>>,
  pub media_path: Option<Vec<&'a AtRule<'a>>>,
  pub math: Math,
  pub math_on: bool,
  pub in_calc: bool,
  pub calc_stack: u32,
  pub parens_stack: u32,
  pub import_multiple: bool,
  pub depth: u32,
}

const MAX_DEPTH: u32 = 1024;

/// Runs `f` with at least 128 KB of stack available, growing the stack on
/// the heap when needed (deep but valid nesting must not overflow).
#[inline]
pub(crate) fn grow<R>(f: impl FnOnce() -> R) -> R {
  #[cfg(not(target_arch = "wasm32"))]
  {
    stacker::maybe_grow(256 * 1024, 4 * 1024 * 1024, f)
  }
  #[cfg(target_arch = "wasm32")]
  {
    f()
  }
}

impl<'a, 's> EvalCtx<'a, 's> {
  pub fn new(s: &'s Session<'a>, frames: Vec<&'a Ruleset<'a>>) -> EvalCtx<'a, 's> {
    EvalCtx {
      s,
      a: s.a,
      frames,
      media_blocks: None,
      media_path: None,
      math: s.opts.math,
      math_on: true,
      in_calc: false,
      calc_stack: 0,
      parens_stack: 0,
      import_multiple: false,
      depth: 0,
    }
  }

  /// `new contexts.Eval(context, frames)`: copies the option-like
  /// properties only.
  pub fn derive(&self, frames: Vec<&'a Ruleset<'a>>) -> EvalCtx<'a, 's> {
    EvalCtx {
      s: self.s,
      a: self.a,
      frames,
      media_blocks: None,
      media_path: None,
      math: self.math,
      math_on: true,
      in_calc: false,
      calc_stack: 0,
      parens_stack: 0,
      import_multiple: self.import_multiple,
      depth: self.depth,
    }
  }

  pub fn is_math_on(&self, op: Option<&str>) -> bool {
    if !self.math_on {
      return false;
    }
    if op == Some("/") && self.math != Math::Always && self.parens_stack == 0 {
      return false;
    }
    if self.math > Math::ParensDivision {
      return self.parens_stack > 0;
    }
    true
  }

  pub fn enter_calc(&mut self) {
    self.calc_stack += 1;
    self.in_calc = true;
  }
  pub fn exit_calc(&mut self) {
    self.calc_stack = self.calc_stack.saturating_sub(1);
    if self.calc_stack == 0 {
      self.in_calc = false;
    }
  }

  /// The eval context as a CSS context (`toCSS(context)` during eval).
  pub fn css_ctx(&self) -> CssCtx {
    CssCtx {
      compress: self.s.opts.compress,
      strict_units: self.s.opts.strict_units,
      ..CssCtx::default()
    }
  }

  pub fn to_css(&self, n: Node<'a>) -> LResult<String> {
    css::to_css(n, &mut self.css_ctx())
  }

  pub fn str(&self, s: &str) -> &'a str {
    self.a.str(s)
  }

  fn enter(&mut self) -> LResult<()> {
    self.depth += 1;
    // evaluation that grows without bound (less.js runs out of heap)
    let steps = self.s.eval_steps.get().wrapping_add(1);
    self.s.eval_steps.set(steps);
    if steps & 1023 == 0 {
      let budget = crate::error::arena_budget(self.s.source_len());
      if self.a.bump.allocated_bytes() > budget {
        return Err(crate::error::out_of_memory(budget));
      }
    }
    if self.depth > MAX_DEPTH {
      return Err(LessError::native("Maximum call stack size exceeded"));
    }
    Ok(())
  }
  fn leave(&mut self) {
    self.depth -= 1;
  }

  // ------------------------------------------------------------------
  // generic dispatch
  // ------------------------------------------------------------------

  #[inline]
  pub fn eval(&mut self, n: Node<'a>) -> LResult<Node<'a>> {
    let mut n = n;
    loop {
      match n {
        // these evaluate to themselves
        Node::Color(_)
        | Node::Dimension(_)
        | Node::Keyword(_)
        | Node::UnicodeDescriptor(_)
        | Node::Comment(_) => return Ok(n),
        // `Value.eval` with one item
        Node::Value(v) if v.value.len() == 1 => n = v.value[0],
        // `Expression.eval` with one item and no parens: the item's
        // value, with `returnValue.noSpacing ||= noSpacing`
        Node::Expression(e) if e.value.len() == 1 && !e.parens => {
          let rv = self.eval(e.value[0])?;
          let ns = if rv.no_spacing() == Some(true) {
            Some(true)
          } else {
            e.no_spacing.get()
          };
          rv.set_no_spacing(ns);
          return Ok(rv);
        }
        _ => return self.eval_node(n),
      }
    }
  }

  fn eval_node(&mut self, n: Node<'a>) -> LResult<Node<'a>> {
    self.enter()?;
    // check stack headroom every 16 levels (cheap enough, and each level
    // uses far less than the 128 KB red zone / 16)
    let r = if self.depth.is_multiple_of(16) {
      grow(|| self.eval_inner(n))
    } else {
      self.eval_inner(n)
    };
    self.leave();
    r
  }

  pub(crate) fn enter_pub(&mut self) -> LResult<()> {
    self.enter()
  }
  pub(crate) fn leave_pub(&mut self) {
    self.leave()
  }

  fn eval_inner(&mut self, n: Node<'a>) -> LResult<Node<'a>> {
    Ok(match n {
      Node::Anonymous(an) => {
        let x = self.a.alloc(Anonymous {
          vis: Vis::new(an.vis.info()),
          value: an.value,
          pos: an.pos,
          map_lines: an.map_lines,
          ruleset_like: an.ruleset_like,
          prevent_reparse: an.prevent_reparse,
          no_spacing: Cell::new(None),
          is_null: an.is_null,
          has_file: an.has_file,
        });
        Node::Anonymous(x)
      }
      Node::Assignment(a) => {
        let v = self.eval(a.value)?;
        Node::Assignment(self.a.alloc(Assignment {
          key: a.key,
          value: v,
        }))
      }
      Node::AtRule(a) => self.eval_at_rule(a)?,
      Node::Media(a) | Node::Container(a) => self.eval_media(a)?,
      Node::Attribute(a) => {
        let key = match a.key {
          StrOrNode::Node(k) => StrOrNode::Node(self.eval(k)?),
          s => s,
        };
        let value = match a.value {
          Some(StrOrNode::Node(v)) => Some(StrOrNode::Node(self.eval(v)?)),
          v => v,
        };
        Node::Attribute(self.a.alloc(Attribute {
          key,
          op: a.op,
          value,
          cif: a.cif,
        }))
      }
      Node::Call(c) => self.eval_call(c)?,
      Node::Color(_)
      | Node::Dimension(_)
      | Node::Keyword(_)
      | Node::UnicodeDescriptor(_)
      | Node::Comment(_) => n,
      Node::Condition(c) => {
        let b = self.eval_condition(c)?;
        self.a.keyword(if b { "true" } else { "false" })
      }
      Node::Declaration(d) => Node::Declaration(self.eval_declaration(d)?),
      Node::DetachedRuleset(d) => {
        let frames = match d.frames {
          Some(f) => f,
          None => {
            self.s.frames_captured.set(self.s.frames_captured.get() + 1);
            self.a.slice(&self.frames)
          }
        };
        Node::DetachedRuleset(self.a.alloc(DetachedRuleset {
          vis: Vis::new(d.vis.info()),
          ruleset: d.ruleset,
          frames: Some(frames),
        }))
      }
      Node::Element(e) => Node::Element(self.eval_element(e)?),
      Node::Expression(e) => self.eval_expression(e)?,
      Node::Extend(e) => Node::Extend(self.eval_extend(e)?),
      Node::Import(i) => {
        let r = self.eval_import(i)?;
        match r.len() {
          1 if !r.is_list => r.nodes[0],
          _ => Node::Rules(self.a.rules(&r.nodes)),
        }
      }
      Node::JavaScript(j) => self.eval_javascript(j)?,
      Node::MixinCall(mc) => {
        let r = self.eval_mixin_call(mc)?;
        Node::Rules(self.a.rules(&r))
      }
      Node::MixinDefinition(md) => self.eval_mixin_definition(md),
      Node::NamespaceValue(nv) => self.eval_namespace_value(nv)?,
      Node::Negative(ng) => {
        if self.is_math_on(Some("*")) {
          let minus_one = self.a.dimension(-1.0, &EMPTY_UNIT);
          let op = self.a.alloc(Operation {
            op: "*",
            operands: [minus_one, ng.value],
            is_spaced: false,
          });
          self.eval_operation(op)?
        } else {
          let v = self.eval(ng.value)?;
          Node::Negative(self.a.alloc(Negative { value: v }))
        }
      }
      Node::Operation(o) => self.eval_operation(o)?,
      Node::Paren(p) => {
        let v = self.eval(p.value)?;
        let np = self.a.alloc(Paren {
          value: v,
          no_spacing: Cell::new(None),
        });
        if p.no_spacing.get() == Some(true) {
          np.no_spacing.set(Some(true));
        }
        Node::Paren(np)
      }
      Node::Property(p) => self.eval_property(p)?,
      Node::QueryInParens(q) => {
        let l = self.eval(q.lvalue)?;
        let mv = self.eval(q.mvalue)?;
        let r = match q.rvalue {
          Some(r) => Some(self.eval(r)?),
          None => None,
        };
        Node::QueryInParens(self.a.alloc(QueryInParens {
          op: q.op,
          lvalue: l,
          mvalue: mv,
          op2: q.op2,
          rvalue: r,
          pos: q.pos,
        }))
      }
      Node::Quoted(q) => self.eval_quoted(q)?,
      Node::Ruleset(r) => Node::Ruleset(self.eval_ruleset(r)?),
      Node::Selector(s) => Node::Selector(self.eval_selector(s)?),
      Node::Url(u) => self.eval_url(u)?,
      Node::Value(v) => {
        if v.value.len() == 1 {
          self.eval(v.value[0])?
        } else {
          let mut out = BVec::with_capacity_in(v.value.len(), self.a.bump);
          for item in v.value {
            out.push(self.eval(*item)?);
          }
          self.a.value_owned(out.into_bump_slice())
        }
      }
      Node::Variable(v) => self.eval_variable(v)?,
      Node::VariableCall(vc) => self.eval_variable_call(vc)?,
      Node::Rules(r) => Node::Rules(r),
    })
  }

  // ------------------------------------------------------------------
  // values
  // ------------------------------------------------------------------

  fn eval_expression(&mut self, e: &'a Expression<'a>) -> LResult<Node<'a>> {
    let no_spacing = e.no_spacing.get();
    let math_on = self.is_math_on(None);
    let in_paren = e.parens;
    let mut double_paren = false;
    if in_paren {
      self.parens_stack += 1;
    }
    let rv = if e.value.len() > 1 {
      let mut out = BVec::with_capacity_in(e.value.len(), self.a.bump);
      for v in e.value {
        out.push(self.eval(*v)?);
      }
      Node::Expression(
        self
          .a
          .expression_owned(out.into_bump_slice(), e.no_spacing.get()),
      )
    } else if e.value.len() == 1 {
      if let Node::Expression(first) = e.value[0]
        && first.parens
        && !first.parens_in_op.get()
        && !self.in_calc
      {
        double_paren = true;
      }
      self.eval(e.value[0])?
    } else {
      Node::Expression(e)
    };
    if in_paren {
      self.parens_stack -= 1;
    }
    let rv = if e.parens
      && e.parens_in_op.get()
      && !math_on
      && !double_paren
      && !matches!(rv, Node::Dimension(_))
    {
      self.a.paren(rv)
    } else {
      rv
    };
    let ns = if rv.no_spacing() == Some(true) {
      Some(true)
    } else {
      no_spacing
    };
    rv.set_no_spacing(ns);
    Ok(rv)
  }

  fn eval_operation(&mut self, o: &'a Operation<'a>) -> LResult<Node<'a>> {
    let mut a = self.eval(o.operands[0])?;
    let mut b = self.eval(o.operands[1])?;
    if self.is_math_on(Some(o.op)) {
      let op = if o.op == "./" { "/" } else { o.op };
      if let (Node::Dimension(d), Node::Color(_)) = (a, b) {
        a = Node::Color(self.dimension_to_color(d));
      }
      if let (Node::Color(_), Node::Dimension(d)) = (a, b) {
        b = Node::Color(self.dimension_to_color(d));
      }
      let operable = |n: &Node<'a>| matches!(n, Node::Dimension(_) | Node::Color(_));
      if !operable(&a) || !operable(&b) {
        if (matches!(a, Node::Operation(_)) || matches!(b, Node::Operation(_)))
          && matches!(a, Node::Operation(x) if x.op == "/")
          && self.math == Math::ParensDivision
        {
          return Ok(Node::Operation(self.a.alloc(Operation {
            op: o.op,
            operands: [a, b],
            is_spaced: o.is_spaced,
          })));
        }
        return Err(LessError::new("Operation", "Operation on an invalid type"));
      }
      return match (a, b) {
        (Node::Dimension(x), Node::Dimension(y)) => units::dimension_operate(self, op, x, y),
        (Node::Color(x), Node::Color(y)) => Ok(self.color_operate(op, x, y)),
        _ => Err(LessError::new("Operation", "Operation on an invalid type")),
      };
    }
    Ok(Node::Operation(self.a.alloc(Operation {
      op: o.op,
      operands: [a, b],
      is_spaced: o.is_spaced,
    })))
  }

  fn dimension_to_color(&self, d: &Dimension<'a>) -> &'a Color<'a> {
    self.a.color([d.value, d.value, d.value], 1.0, None)
  }

  fn color_operate(&self, op: &str, x: &Color<'a>, y: &Color<'a>) -> Node<'a> {
    let alpha = x.alpha * (1.0 - y.alpha) + y.alpha;
    let mut rgb = [0.0; 3];
    for c in 0..3 {
      rgb[c] = js_operate(op, x.rgb[c], y.rgb[c]);
    }
    Node::Color(self.a.color(rgb, alpha, None))
  }

  // ------------------------------------------------------------------
  // variables and properties
  // ------------------------------------------------------------------

  pub(crate) fn eval_variable(&mut self, v: &'a Variable<'a>) -> LResult<Node<'a>> {
    let mut name: &'a str = v.name;
    if name.starts_with("@@") {
      let inner = self.a.alloc(Variable {
        name: &name[1..],
        pos: v.pos,
        evaluating: Cell::new(false),
        no_spacing: Cell::new(None),
      });
      let r = self.eval_variable(inner)?;
      name = self.str(&format!("@{}", js_value_string(r)));
    }
    if v.evaluating.get() {
      return Err(LessError::at(
        "Name",
        format!("Recursive variable definition for {}", name),
        v.pos,
      ));
    }
    v.evaluating.set(true);
    let mut fi = 0;
    while fi < self.frames.len() {
      let frame = self.frames[fi];
      fi += 1;
      if let Some(decl) = self.frame_variable(frame, name)? {
        let imp = decl.important();
        if !imp.is_empty()
          && let Some(last) = self.s.important_scope.borrow_mut().last_mut()
        {
          *last = Some(imp);
        }
        let r = if self.in_calc {
          let call = self.a.alloc(Call {
            vis: Vis::default(),
            name: "_SELF",
            args: self.a.slice(&[decl.value.get()]),
            pos: Pos::NONE,
            calc: false,
          });
          self.eval_call(call)?
        } else {
          self.eval(decl.value.get())?
        };
        v.evaluating.set(false);
        return Ok(r);
      }
    }
    Err(LessError::at(
      "Name",
      format!("variable {} is undefined", name),
      v.pos,
    ))
  }

  fn eval_property(&mut self, p: &'a Property<'a>) -> LResult<Node<'a>> {
    let name = p.name;
    if p.evaluating.get() {
      return Err(LessError::at(
        "Name",
        format!("Recursive property reference for {}", name),
        p.pos,
      ));
    }
    p.evaluating.set(true);
    let mut fi = 0;
    while fi < self.frames.len() {
      let frame = self.frames[fi];
      fi += 1;
      if let Some(decls) = self.frame_property(frame, name)? {
        let mut copies: Vec<&'a Declaration<'a>> = decls
          .iter()
          .map(|v| {
            self.a.alloc(Declaration {
              vis: Vis::default(),
              name: v.name,
              value: Cell::new(v.value.get()),
              important: Cell::new(v.important.get()),
              merge: Cell::new(v.merge.get()),
              pos: v.pos,
              inline: v.inline,
              variable: v.variable,
              flags: Cell::new(v.flags.get() & !Declaration::SHARED),
            }) as &'a Declaration<'a>
          })
          .collect();
        merge_rules_decls(self.a, &mut copies);
        let last = *copies.last().expect("merged declarations are nonempty");
        let imp = last.important();
        if !imp.is_empty()
          && let Some(l) = self.s.important_scope.borrow_mut().last_mut()
        {
          *l = Some(imp);
        }
        let r = self.eval(last.value.get())?;
        p.evaluating.set(false);
        return Ok(r);
      }
    }
    Err(LessError::at(
      "Name",
      format!("Property '{}' is undefined", name),
      p.pos,
    ))
  }

  // ------------------------------------------------------------------
  // strings and urls
  // ------------------------------------------------------------------

  fn eval_quoted(&mut self, q: &'a Quoted<'a>) -> LResult<Node<'a>> {
    /// Nested interpolation (`@{@{a}}`) settles in a few passes.
    const MAX_INTERPOLATION_PASSES: u32 = 1_000;
    if !q.value.bytes().any(|b| b == b'@' || b == b'$') {
      // nothing to interpolate
      return Ok(Node::Quoted(self.a.alloc(Quoted {
        vis: Vis::default(),
        quote: q.quote,
        value: q.value,
        escaped: q.escaped,
        pos: q.pos,
        permissive: false,
      })));
    }
    let mut value: String = q.value.to_string();
    // Variable interpolation until a fixpoint. A value that interpolates
    // itself never settles (less.js loops until it runs out of memory).
    let settled = |passes: u32, value: &str| -> LResult<()> {
      if passes > MAX_INTERPOLATION_PASSES {
        return Err(LessError::at(
          "Runtime",
          format!(
            "Interpolation does not settle after {} passes",
            MAX_INTERPOLATION_PASSES
          ),
          q.pos,
        ));
      }
      let budget = crate::error::arena_budget(self.s.source_len());
      if value.len() > budget / 8 {
        return Err(crate::error::out_of_memory(budget));
      }
      Ok(())
    };
    let mut passes = 0;
    loop {
      let replaced = self.replace_variables(&value, q)?;
      if replaced == value {
        break;
      }
      value = replaced;
      passes += 1;
      settled(passes, &value)?;
    }
    loop {
      let replaced = self.replace_properties(&value, q)?;
      if replaced == value {
        break;
      }
      value = replaced;
      passes += 1;
      settled(passes, &value)?;
    }
    let v = if value == q.value {
      q.value
    } else {
      self.str(&value)
    };
    Ok(Node::Quoted(self.a.alloc(Quoted {
      vis: Vis::default(),
      quote: q.quote,
      value: v,
      escaped: q.escaped,
      pos: q.pos,
      permissive: false,
    })))
  }

  /// Appends an interpolated value (`@{name}`, `${name}`) to `out`.
  fn interpolated_push(&mut self, n: Node<'a>, out: &mut String) -> LResult<()> {
    match self.eval(n)? {
      Node::Quoted(q) => out.push_str(q.value),
      other => css::append_css(other, &mut self.css_ctx(), out)?,
    }
    Ok(())
  }

  fn replace_variables(&mut self, value: &str, q: &'a Quoted<'a>) -> LResult<String> {
    let b = value.as_bytes();
    let mut out = String::with_capacity(value.len());
    let mut last = 0;
    let mut i = 0;
    while i < b.len() {
      if b[i] == b'@' {
        // `@{name[..]}`
        if let Some(end) = m::interpolation_at(b, i) {
          let raw = &value[i + 2..end - 1];
          out.push_str(&value[last..i]);
          let node = resolve_interpolated_variable(self.a, raw, q.pos);
          self.interpolated_push(node, &mut out)?;
          i = end;
          last = end;
          continue;
        }
        // permissive values also replace bare `@name[..]`
        if q.permissive
          && let Some(end) = m::bare_variable_with_lookups(b, i)
        {
          let raw = &value[i + 1..end];
          out.push_str(&value[last..i]);
          let node = resolve_interpolated_variable(self.a, raw, q.pos);
          self.interpolated_push(node, &mut out)?;
          i = end;
          last = end;
          continue;
        }
      }
      i += 1;
    }
    out.push_str(&value[last..]);
    Ok(out)
  }

  fn replace_properties(&mut self, value: &str, q: &'a Quoted<'a>) -> LResult<String> {
    let b = value.as_bytes();
    let mut out = String::with_capacity(value.len());
    let mut last = 0;
    let mut i = 0;
    while i < b.len() {
      if b[i] == b'$' {
        if let Some(end) = m::prop_interpolation_at(b, i) {
          let raw = &value[i + 2..end - 1];
          out.push_str(&value[last..i]);
          let node = resolve_interpolated_property(self.a, raw, q.pos);
          self.interpolated_push(node, &mut out)?;
          i = end;
          last = end;
          continue;
        }
        if q.permissive {
          let n = b[i + 1..]
            .iter()
            .take_while(|&&c| m::is_word_dash(c))
            .count();
          if n > 0 {
            let raw = &value[i + 1..i + 1 + n];
            out.push_str(&value[last..i]);
            let node = resolve_interpolated_property(self.a, raw, q.pos);
            self.interpolated_push(node, &mut out)?;
            i += 1 + n;
            last = i;
            continue;
          }
        }
      }
      i += 1;
    }
    out.push_str(&value[last..]);
    Ok(out)
  }

  fn eval_url(&mut self, u: &'a Url<'a>) -> LResult<Node<'a>> {
    let val = self.eval(u.value)?;
    let mut val = val;
    if !u.is_evald {
      let info = self.s.pos_file_info(u.pos);
      if let Some(sv) = js_string_value(val) {
        let mut new_value: String;
        let rootpath = info.as_ref().map(|i| i.rootpath);
        if let (Some(rootpath), true) = (rootpath, self.path_requires_rewrite(sv)) {
          let mut rp = rootpath.to_string();
          let quoted = matches!(val, Node::Quoted(q) if !q.quote.is_empty());
          if !quoted {
            rp = escape_path(&rp);
          }
          new_value = rewrite_path(sv, &rp);
        } else {
          new_value = normalize_path(sv);
        }
        let url_args = &self.s.opts.url_args;
        if !url_args.is_empty() && !new_value.trim_start().starts_with("data:") {
          let delim = if new_value.contains('?') { "&" } else { "?" };
          let ua = format!("{}{}", delim, url_args);
          if let Some(p) = new_value.find('#') {
            new_value.insert_str(p, &ua);
          } else {
            new_value.push_str(&ua);
          }
        }
        val = with_string_value(self.a, val, self.a.str(&new_value));
      }
    }
    Ok(Node::Url(self.a.alloc(Url {
      value: val,
      pos: u.pos,
      is_evald: true,
    })))
  }

  pub fn path_requires_rewrite(&self, path: &str) -> bool {
    use crate::options::RewriteUrls;
    if self.s.opts.rewrite_urls == RewriteUrls::Local {
      is_path_local_relative(path)
    } else {
      is_path_relative(path)
    }
  }

  // ------------------------------------------------------------------
  // calls
  // ------------------------------------------------------------------

  pub(crate) fn eval_call(&mut self, c: &'a Call<'a>) -> LResult<Node<'a>> {
    let current_math = self.math_on;
    self.math_on = !c.calc;
    let calc_entered = c.calc || self.in_calc;
    if calc_entered {
      self.enter_calc();
    }
    let lower: std::borrow::Cow<'_, str> = if c.name.bytes().any(|b| b.is_ascii_uppercase()) {
      c.name.cow_to_ascii_lowercase()
    } else {
      c.name.into()
    };
    let func = self.s.functions.get(&lower);
    let mut result: Option<Node<'a>> = None;
    if let Some(func) = func {
      match crate::functions::call(self, func, &lower, c, c.args) {
        Ok(r) => {
          result = r;
          if c.calc || self.in_calc {
            self.exit_calc();
          }
          self.math_on = current_math;
        }
        Err(e) => {
          if e.has_line_col {
            return Err(e);
          }
          let msg = if e.message.is_empty() {
            format!("Error evaluating function `{}`", c.name)
          } else {
            format!("Error evaluating function `{}`: {}", c.name, e.message)
          };
          let typ = if e.typ.is_empty() {
            "Runtime"
          } else {
            e.typ.as_str()
          };
          let mut ne = LessError::at(typ, msg, c.pos);
          ne.has_line_col = true;
          return Err(ne);
        }
      }
    }
    if let Some(r) = result {
      // `result._index = this._index; result._fileInfo = this._fileInfo`
      let r = match r {
        Node::Anonymous(an) if !c.pos.is_none() => Node::Anonymous(self.a.alloc(Anonymous {
          vis: Vis::new(an.vis.info()),
          value: an.value,
          pos: c.pos,
          map_lines: an.map_lines,
          ruleset_like: an.ruleset_like,
          prevent_reparse: an.prevent_reparse,
          no_spacing: Cell::new(an.no_spacing.get()),
          is_null: an.is_null,
          has_file: true,
        })),
        Node::Quoted(q) if q.pos != c.pos && !c.pos.is_none() => {
          Node::Quoted(self.a.alloc(Quoted {
            vis: Vis::new(q.vis.info()),
            quote: q.quote,
            value: q.value,
            escaped: q.escaped,
            pos: c.pos,
            permissive: q.permissive,
          }))
        }
        other => other,
      };
      set_node_pos(r, c.pos);
      return Ok(r);
    }
    let mut args: smallvec::SmallVec<[Node<'a>; 8]> =
      smallvec::SmallVec::with_capacity(c.args.len());
    for a in c.args {
      args.push(self.eval(*a)?);
    }
    if c.calc || self.in_calc {
      self.exit_calc();
    }
    self.math_on = current_math;
    Ok(Node::Call(self.a.alloc(Call {
      vis: Vis::default(),
      name: c.name,
      args: self.a.slice(&args),
      pos: c.pos,
      calc: c.calc,
    })))
  }

  fn eval_javascript(&mut self, j: &'a JavaScript<'a>) -> LResult<Node<'a>> {
    if !self.s.opts.javascript_enabled {
      return Err(LessError::at(
        "",
        "Inline JavaScript is not enabled. Is it set in your options?",
        j.pos,
      ));
    }
    Err(LessError::at(
      "Syntax",
      "Inline JavaScript is not supported by less-rs (no JavaScript engine is available)",
      j.pos,
    ))
  }

  // ------------------------------------------------------------------
  // conditions and comparison
  // ------------------------------------------------------------------

  pub(crate) fn eval_condition(&mut self, c: &'a Condition<'a>) -> LResult<bool> {
    let result = match c.op {
      "and" | "or" => {
        let a = self.eval_truthy(c.lvalue)?;
        let b = self.eval_truthy(c.rvalue)?;
        if c.op == "and" { a && b } else { a || b }
      }
      op => {
        let a = self.eval(c.lvalue)?;
        let b = self.eval(c.rvalue)?;
        match compare(a, b) {
          Some(-1) => op == "<" || op == "=<" || op == "<=",
          Some(0) => op == "=" || op == ">=" || op == "=<" || op == "<=",
          Some(1) => op == ">" || op == ">=",
          _ => false,
        }
      }
    };
    Ok(if c.negate.get() { !result } else { result })
  }

  fn eval_truthy(&mut self, n: Node<'a>) -> LResult<bool> {
    match n {
      Node::Condition(c) => self.eval_condition(c),
      other => {
        self.eval(other)?;
        Ok(true)
      }
    }
  }

  // ------------------------------------------------------------------
  // declarations
  // ------------------------------------------------------------------

  pub(crate) fn eval_declaration(
    &mut self,
    d: &'a Declaration<'a>,
  ) -> LResult<&'a Declaration<'a>> {
    let r = self.eval_declaration_inner(d);
    r.map_err(|mut e| {
      if !e.index_set {
        e.set_pos(d.pos);
      }
      e
    })
  }

  fn eval_declaration_inner(&mut self, d: &'a Declaration<'a>) -> LResult<&'a Declaration<'a>> {
    let mut variable = d.variable;
    let name: &'a str = match d.name {
      DeclName::Str(s) => s,
      DeclName::Parts(parts) => {
        variable = false;
        if parts.len() == 1 {
          if let Node::Keyword(k) = parts[0] {
            k.value
          } else {
            self.eval_name(parts)?
          }
        } else {
          self.eval_name(parts)?
        }
      }
    };
    let mut prev_math = None;
    if name == "font" && self.math == Math::Always {
      prev_math = Some(self.math);
      self.math = Math::ParensDivision;
    }
    self.s.important_scope.borrow_mut().push(None);
    let res = match d.value.get() {
      // `Anonymous.eval` copies; as a whole declaration value nothing
      // later changes the copy's `noSpacing`, so it is the same node
      Node::Anonymous(an) if an.no_spacing.get().is_none() => Ok(Node::Anonymous(an)),
      v => self.eval(v),
    };
    if let Some(pm) = prev_math {
      self.math = pm;
    }
    let value = res?;
    if !d.variable && matches!(value, Node::DetachedRuleset(_) | Node::Rules(_)) {
      return Err(LessError::at(
        "",
        "Rulesets cannot be evaluated on a property.",
        d.pos,
      ));
    }
    let mut important = d.important();
    let scope = self.s.important_scope.borrow_mut().pop().flatten();
    if important.is_empty()
      && let Some(imp) = scope
    {
      important = imp;
    }
    let important = important_str(self.a, important);
    // the source's name when it is that string (a single-keyword name
    // reads the same everywhere: `name_str`, output, property maps)
    let dn: &'a DeclName<'a> = match *d.name {
      DeclName::Str(s) if std::ptr::eq(s, name) => d.name,
      DeclName::Parts([Node::Keyword(k)]) if std::ptr::eq(k.value, name) => d.name,
      _ => self.a.alloc(DeclName::Str(name)),
    };
    // the copy would equal the declaration (`new Declaration` gets default
    // visibility and is not `parsed`): share it (see `Declaration::SHARED`).
    // Merged declarations are excluded, `mergeRules` changes them in place.
    if std::ptr::eq(dn, d.name)
      && value.ptr_eq(&d.value.get())
      && important == d.important()
      && variable == d.variable
      && d.merge.get() == Merge::None
      && d.flags.get() & Declaration::PARSED == 0
      && d.vis.info() == VisInfo::default()
    {
      d.flags.set(Declaration::SHARED);
      return Ok(d);
    }
    let nd = self.a.declaration_ref(
      dn,
      value,
      important,
      d.merge.get(),
      d.pos,
      d.inline,
      Some(variable),
    );
    Ok(nd)
  }

  fn eval_name(&mut self, parts: &'a [Node<'a>]) -> LResult<&'a str> {
    let mut s = String::new();
    let mut ctx = self.css_ctx();
    for p in parts {
      let v = self.eval(*p)?;
      s.push_str(&css::to_css(v, &mut ctx)?);
    }
    Ok(self.str(&s))
  }

  // ------------------------------------------------------------------
  // selectors
  // ------------------------------------------------------------------

  pub(crate) fn eval_selector(&mut self, s: &'a Selector<'a>) -> LResult<&'a Selector<'a>> {
    let evald_condition = match s.condition() {
      Some(Node::Condition(c)) => Some(self.eval_condition(c)?),
      Some(other) => {
        self.eval(other)?;
        Some(true)
      }
      None => None,
    };
    if let Some(list) = s.list() {
      let mut items = Vec::with_capacity(list.len());
      for n in list {
        items.push(self.eval(*n)?);
      }
      let ns = self.a.alloc(Selector::new(
        self.a,
        &[],
        Some(crate::tree::SelAux::List(self.a.slice(&items))),
        s.extend_list(),
        s.pos,
        s.vis.info(),
      ));
      ns.evald_condition
        .set(evald_condition.unwrap_or(s.evald_condition.get()));
      ns.media_empty.set(s.media_empty.get());
      return Ok(ns);
    }
    // Plain elements evaluate to themselves; then the element list (and
    // its mixin tokens) can be shared with the source selector.
    let mut changed: Option<BVec<'a, &'a Element<'a>>> = None;
    for (i, e) in s.elements.iter().enumerate() {
      let ne = self.eval_element(e)?;
      match &mut changed {
        Some(v) => v.push(ne),
        None if !std::ptr::eq(ne, *e) => {
          let mut v = BVec::with_capacity_in(s.elements.len(), self.a.bump);
          v.extend_from_slice(&s.elements[..i]);
          v.push(ne);
          changed = Some(v);
        }
        None => {}
      }
    }
    let shared = changed.is_none();
    let elements = match changed {
      Some(v) => v.into_bump_slice(),
      None => s.elements,
    };
    let extend_list = match s.extend_list() {
      Some(list) => {
        let mut out = Vec::with_capacity(list.len());
        for e in list {
          out.push(self.eval_extend(e)?);
        }
        Some(self.a.slice(&out))
      }
      None => None,
    };
    let ns = create_derived_owned(self.a, s, elements, extend_list, evald_condition);
    if shared
      && !elements.is_empty()
      && let Some(t) = s.mixin_elements_cached()
    {
      let _ = ns.mixin_elements_cache(self.a).set(t);
    }
    Ok(ns)
  }

  pub(crate) fn eval_element(&mut self, e: &'a Element<'a>) -> LResult<&'a Element<'a>> {
    let value = match e.value {
      ElValue::Node(n) => ElValue::Node(self.eval(n)?),
      // A plain element evaluates to an identical element; share it.
      // (less.js shares the element's Combinator object between the
      // copies, and later passes never mutate shared elements here.)
      ElValue::Str(_) => return Ok(e),
    };
    Ok(self.a.alloc(Element {
      vis: Vis::new(e.vis.info()),
      combinator: Cell::new(e.combinator.get()),
      value,
      is_variable: e.is_variable,
      pos: e.pos,
    }))
  }

  fn eval_extend(&mut self, e: &'a Extend<'a>) -> LResult<&'a Extend<'a>> {
    let sel = self.eval_selector(e.selector)?;
    Ok(self.a.extend(sel, e.option, e.pos, e.vis.info()))
  }

  /// `defaultFunc.eval()`
  pub(crate) fn default_value(&self) -> LResult<Option<Node<'a>>> {
    match self.s.default_state.get() {
      DefaultState::Error => Err(LessError::new(
        "Syntax",
        "it is currently only allowed in parametric mixin guards,",
      )),
      DefaultState::Value(v) => Ok(Some(self.a.keyword(if v { "true" } else { "false" }))),
      DefaultState::Unset => Ok(None),
    }
  }
}

/// `Selector.createDerived(elements, extendList, evaldCondition)`
#[track_caller]
pub(crate) fn create_derived<'a>(
  a: Alloc<'a>,
  s: &Selector<'a>,
  elements: &[&'a Element<'a>],
  extend_list: Option<&'a [&'a Extend<'a>]>,
  evald_condition: Option<bool>,
) -> &'a Selector<'a> {
  create_derived_owned(a, s, a.slice(elements), extend_list, evald_condition)
}

#[track_caller]
pub(crate) fn create_derived_owned<'a>(
  a: Alloc<'a>,
  s: &Selector<'a>,
  elements: &'a [&'a Element<'a>],
  extend_list: Option<&'a [&'a Extend<'a>]>,
  evald_condition: Option<bool>,
) -> &'a Selector<'a> {
  let ns = a.selector_owned(
    elements,
    extend_list.or(s.extend_list()),
    None,
    s.pos,
    s.vis.info(),
  );
  ns.evald_condition
    .set(evald_condition.unwrap_or(s.evald_condition.get()));
  ns.media_empty.set(s.media_empty.get());
  ns
}

/// Sets `_index`/`_fileInfo` of a function result (only for node types whose
/// position is mutable in this implementation).
fn set_node_pos(n: Node<'_>, pos: Pos) {
  match n {
    Node::Dimension(d) => d.pos.set(pos),
    Node::Color(c) => c.pos.set(pos),
    Node::Keyword(k) => k.pos.set(pos),
    _ => {}
  }
}

pub(crate) fn js_operate(op: &str, a: f64, b: f64) -> f64 {
  match op {
    "+" => a + b,
    "-" => a - b,
    "*" => a * b,
    "/" => a / b,
    _ => f64::NAN,
  }
}

/// `node.value` coerced to a string as JS template literals would.
pub(crate) fn js_value_string(n: Node<'_>) -> String {
  match n {
    Node::Dimension(d) => num::to_js_string(d.value),
    Node::Keyword(k) => k.value.to_string(),
    Node::Quoted(q) => q.value.to_string(),
    Node::Anonymous(a) => a.value.to_string(),
    Node::Color(c) => c
      .value
      .get()
      .map(|s| s.to_string())
      .unwrap_or_else(|| "undefined".into()),
    Node::UnicodeDescriptor(u) => u.value.to_string(),
    _ => "undefined".to_string(),
  }
}

/// `typeof val.value === 'string'` → the string.
pub(crate) fn js_string_value<'a>(n: Node<'a>) -> Option<&'a str> {
  match n {
    Node::Quoted(q) => Some(q.value),
    Node::Anonymous(a) if !a.is_null => Some(a.value),
    Node::Keyword(k) => Some(k.value),
    Node::UnicodeDescriptor(u) => Some(u.value),
    _ => None,
  }
}

/// Returns a copy of `n` whose string value is replaced (`val.value = ...`).
pub(crate) fn with_string_value<'a>(a: Alloc<'a>, n: Node<'a>, v: &'a str) -> Node<'a> {
  match n {
    Node::Quoted(q) => Node::Quoted(a.alloc(Quoted {
      vis: Vis::new(q.vis.info()),
      quote: q.quote,
      value: v,
      escaped: q.escaped,
      pos: q.pos,
      permissive: q.permissive,
    })),
    Node::Anonymous(x) => Node::Anonymous(a.alloc(Anonymous {
      vis: Vis::new(x.vis.info()),
      value: v,
      pos: x.pos,
      map_lines: x.map_lines,
      ruleset_like: x.ruleset_like,
      prevent_reparse: x.prevent_reparse,
      no_spacing: Cell::new(x.no_spacing.get()),
      is_null: false,
      has_file: x.has_file,
    })),
    Node::Keyword(_) => a.keyword(v),
    other => other,
  }
}

fn escape_path(p: &str) -> String {
  let mut s = String::with_capacity(p.len());
  for c in p.chars() {
    if matches!(c, '(' | ')' | '\'' | '"') || c.is_whitespace() {
      s.push('\\');
    }
    s.push(c);
  }
  s
}

pub(crate) fn is_path_relative(path: &str) -> bool {
  let b = path.as_bytes();
  if b.first() == Some(&b'/') || b.first() == Some(&b'#') {
    return false;
  }
  let n = b
    .iter()
    .take_while(|c| c.is_ascii_alphabetic() || **c == b'-')
    .count();
  !(n > 0 && b.get(n) == Some(&b':'))
}

pub(crate) fn is_path_local_relative(path: &str) -> bool {
  path.starts_with('.')
}

/// `contexts.Eval.prototype.normalizePath`
pub(crate) fn normalize_path(path: &str) -> String {
  let mut out: Vec<&str> = Vec::new();
  for seg in path.split('/') {
    match seg {
      "." => {}
      ".." => {
        if out.is_empty() || *out.last().expect("path segments are nonempty") == ".." {
          out.push(seg);
        } else {
          out.pop();
        }
      }
      _ => out.push(seg),
    }
  }
  out.join("/")
}

/// `contexts.Eval.prototype.rewritePath`
pub(crate) fn rewrite_path(path: &str, rootpath: &str) -> String {
  let mut new_path = normalize_path(&format!("{}{}", rootpath, path));
  if is_path_local_relative(path)
    && is_path_relative(rootpath)
    && !is_path_local_relative(&new_path)
  {
    new_path = format!("./{}", new_path);
  }
  new_path
}

/// `Node.compare(a, b)` → -1, 0, 1 or None (undefined).
pub(crate) fn compare<'a>(a: Node<'a>, b: Node<'a>) -> Option<i32> {
  let has_compare = |n: &Node<'_>| {
    matches!(
      n,
      Node::Anonymous(_) | Node::Color(_) | Node::Dimension(_) | Node::Quoted(_)
    )
  };
  if has_compare(&a) && !matches!(b, Node::Quoted(_) | Node::Anonymous(_)) {
    return node_compare(a, b);
  } else if has_compare(&b) {
    return node_compare(b, a).map(|v| -v);
  } else if a.type_name() != b.type_name() {
    return None;
  }
  match (a.array_value(), b.array_value()) {
    (Some(av), Some(bv)) => {
      if av.len() != bv.len() {
        return None;
      }
      for i in 0..av.len() {
        if compare(av[i], bv[i]) != Some(0) {
          return None;
        }
      }
      Some(0)
    }
    (None, None) => {
      let sa = plain_value(a);
      let sb = plain_value(b);
      match (sa, sb) {
        (Some(x), Some(y)) if x == y => Some(0),
        (None, None) if a.ptr_eq(&b) => Some(0),
        _ => None,
      }
    }
    _ => None,
  }
}

fn plain_value<'a>(n: Node<'a>) -> Option<&'a str> {
  match n {
    Node::Keyword(k) => Some(k.value),
    Node::UnicodeDescriptor(u) => Some(u.value),
    _ => None,
  }
}

fn node_compare<'a>(a: Node<'a>, b: Node<'a>) -> Option<i32> {
  match a {
    Node::Anonymous(_) => {
      let x = css::to_css_plain(a).ok()?;
      let y = css::to_css_plain(b).ok()?;
      if x == y { Some(0) } else { None }
    }
    Node::Quoted(q) => {
      if let Node::Quoted(o) = b
        && !q.escaped
        && !o.escaped
      {
        return numeric_compare_str(q.value, o.value);
      }
      let x = css::to_css_plain(a).ok()?;
      let y = css::to_css_plain(b).ok()?;
      if x == y { Some(0) } else { None }
    }
    Node::Color(c) => match b {
      Node::Color(o) if o.rgb == c.rgb && o.alpha == c.alpha => Some(0),
      _ => None,
    },
    Node::Dimension(d) => match b {
      Node::Dimension(o) => units::dimension_compare(d, o),
      _ => None,
    },
    _ => None,
  }
}

fn numeric_compare_str(a: &str, b: &str) -> Option<i32> {
  // JS string comparison uses UTF-16 code units.
  let x: Vec<u16> = a.encode_utf16().collect();
  let y: Vec<u16> = b.encode_utf16().collect();
  Some(match x.cmp(&y) {
    std::cmp::Ordering::Less => -1,
    std::cmp::Ordering::Equal => 0,
    std::cmp::Ordering::Greater => 1,
  })
}

pub(crate) fn numeric_compare(a: f64, b: f64) -> Option<i32> {
  if a < b {
    Some(-1)
  } else if a == b {
    Some(0)
  } else if a > b {
    Some(1)
  } else {
    None
  }
}

/// `mergeRules` on a list of declarations (used by `Property.eval`).
fn merge_rules_decls<'a>(a: Alloc<'a>, decls: &mut Vec<&'a Declaration<'a>>) {
  let mut nodes: Vec<Node<'a>> = decls.iter().map(|d| Node::Declaration(d)).collect();
  merge_rules(a, &mut nodes);
  *decls = nodes
    .into_iter()
    .filter_map(|n| {
      if let Node::Declaration(d) = n {
        Some(d)
      } else {
        None
      }
    })
    .collect();
}
