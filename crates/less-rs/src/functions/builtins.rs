//! Math, number, string, list, type, boolean and misc functions.

use std::cell::Cell;

use cow_utils::CowUtils;

use super::Registry;
use crate::{
  css,
  error::{LResult, LessError},
  eval::{
    EvalCtx,
    units::{convert_to_unit, unify},
  },
  num,
  tree::*,
};

type R<'a> = LResult<Option<Node<'a>>>;

pub(super) fn register(r: &mut Registry) {
  // boolean.js
  r.add("boolean", boolean);
  r.add_raw("if", if_fn);
  r.add_raw("isdefined", isdefined);
  // default.js
  r.add("default", default_fn);
  // list.js
  r.add("_self", self_fn);
  r.add("~", tilde);
  r.add("extract", extract);
  r.add("length", length);
  r.add("range", range);
  r.add("each", each);
  // math.js
  r.add("ceil", |c, p, a| {
    math_helper(c, p, a, f64::ceil, None, false)
  });
  r.add("floor", |c, p, a| {
    math_helper(c, p, a, f64::floor, None, false)
  });
  r.add("sqrt", |c, p, a| {
    math_helper(c, p, a, f64::sqrt, None, true)
  });
  r.add("abs", |c, p, a| math_helper(c, p, a, f64::abs, None, true));
  r.add("tan", |c, p, a| {
    math_helper(c, p, a, f64::tan, Some(""), true)
  });
  r.add("sin", |c, p, a| {
    math_helper(c, p, a, f64::sin, Some(""), true)
  });
  r.add("cos", |c, p, a| {
    math_helper(c, p, a, f64::cos, Some(""), true)
  });
  r.add("atan", |c, p, a| {
    math_helper(c, p, a, f64::atan, Some("rad"), true)
  });
  r.add("asin", |c, p, a| {
    math_helper(c, p, a, f64::asin, Some("rad"), true)
  });
  r.add("acos", |c, p, a| {
    math_helper(c, p, a, f64::acos, Some("rad"), true)
  });
  r.add("round", round);
  // number.js
  r.add("min", min);
  r.add("max", max);
  r.add("convert", convert);
  r.add("pi", pi);
  r.add("mod", modf);
  r.add("pow", pow);
  r.add("percentage", percentage);
  // string.js
  r.add("e", e_fn);
  r.add("escape", escape);
  r.add("replace", replace);
  r.add("%", format_fn);
  // types.js
  r.add("isruleset", |c, _, a| {
    Ok(Some(kw(
      c,
      matches!(a.first(), Some(Node::DetachedRuleset(_))),
    )))
  });
  r.add("iscolor", |c, _, a| {
    Ok(Some(kw(c, matches!(a.first(), Some(Node::Color(_))))))
  });
  r.add("isnumber", |c, _, a| {
    Ok(Some(kw(c, matches!(a.first(), Some(Node::Dimension(_))))))
  });
  r.add("isstring", |c, _, a| {
    Ok(Some(kw(c, matches!(a.first(), Some(Node::Quoted(_))))))
  });
  r.add("iskeyword", |c, _, a| {
    Ok(Some(kw(c, matches!(a.first(), Some(Node::Keyword(_))))))
  });
  r.add("isurl", |c, _, a| {
    Ok(Some(kw(c, matches!(a.first(), Some(Node::Url(_))))))
  });
  r.add("ispixel", |c, _, a| isunit_str(c, a.first().copied(), "px"));
  r.add("ispercentage", |c, _, a| {
    isunit_str(c, a.first().copied(), "%")
  });
  r.add("isem", |c, _, a| isunit_str(c, a.first().copied(), "em"));
  r.add("isunit", isunit);
  r.add("unit", unit_fn);
  r.add("get-unit", get_unit);
  // style.js
  r.add("style", style);
}

fn kw<'a>(c: &EvalCtx<'a, '_>, b: bool) -> Node<'a> {
  c.a.keyword(if b { "true" } else { "false" })
}

pub(crate) fn undefined_value(what: &str) -> Box<LessError> {
  LessError::native(format!(
    "Cannot read properties of undefined (reading '{}')",
    what
  ))
}

fn arg<'a>(a: &[Node<'a>], i: usize) -> Option<Node<'a>> {
  a.get(i).copied()
}

/// `n.value` for a Dimension (number) – JS coerces other values to NaN.
pub(crate) fn num_value(n: Option<Node<'_>>) -> LResult<f64> {
  match n {
    Some(Node::Dimension(d)) => Ok(d.value),
    Some(other) => Ok(match crate::eval::js_string_value(other) {
      Some(s) => s.trim().parse::<f64>().unwrap_or(f64::NAN),
      None => f64::NAN,
    }),
    None => Err(undefined_value("value")),
  }
}

// ---------------------------------------------------------------------------
// boolean.js / default.js
// ---------------------------------------------------------------------------

fn boolean<'a>(c: &mut EvalCtx<'a, '_>, _p: Pos, a: &[Node<'a>]) -> R<'a> {
  let b = match a.first() {
    Some(Node::Keyword(k)) => k.value == "true",
    Some(_) => true,
    None => false,
  };
  Ok(Some(kw(c, b)))
}

fn if_fn<'a>(c: &mut EvalCtx<'a, '_>, _p: Pos, a: &[Node<'a>]) -> R<'a> {
  let cond = match arg(a, 0) {
    Some(Node::Condition(cd)) => c.eval_condition(cd)?,
    Some(other) => {
      let v = c.eval(other)?;
      !matches!(v, Node::Keyword(k) if k.value == "false")
    }
    None => return Err(undefined_value("eval")),
  };
  if cond {
    match arg(a, 1) {
      Some(t) => Ok(Some(c.eval(t)?)),
      None => Err(undefined_value("eval")),
    }
  } else {
    match arg(a, 2) {
      Some(f) => Ok(Some(c.eval(f)?)),
      None => Ok(Some(c.a.null_anonymous())),
    }
  }
}

fn isdefined<'a>(c: &mut EvalCtx<'a, '_>, _p: Pos, a: &[Node<'a>]) -> R<'a> {
  let r = match arg(a, 0) {
    Some(v) => c.eval(v).is_ok(),
    None => false,
  };
  Ok(Some(kw(c, r)))
}

fn default_fn<'a>(c: &mut EvalCtx<'a, '_>, _p: Pos, _a: &[Node<'a>]) -> R<'a> {
  c.default_value()
}

// ---------------------------------------------------------------------------
// list.js
// ---------------------------------------------------------------------------

fn self_fn<'a>(_c: &mut EvalCtx<'a, '_>, _p: Pos, a: &[Node<'a>]) -> R<'a> {
  Ok(a.first().copied())
}

fn tilde<'a>(c: &mut EvalCtx<'a, '_>, _p: Pos, a: &[Node<'a>]) -> R<'a> {
  if a.len() == 1 {
    return Ok(Some(a[0]));
  }
  Ok(Some(c.a.value(a)))
}

fn items_of<'a>(n: Node<'a>) -> Vec<Node<'a>> {
  match n.array_value() {
    Some(v) => v.to_vec(),
    None => vec![n],
  }
}

fn extract<'a>(_c: &mut EvalCtx<'a, '_>, _p: Pos, a: &[Node<'a>]) -> R<'a> {
  let values = arg(a, 0).ok_or_else(|| undefined_value("value"))?;
  let index = num_value(arg(a, 1))? - 1.0;
  let items = items_of(values);
  if index >= 0.0 && index.fract() == 0.0 && (index as usize) < items.len() {
    Ok(Some(items[index as usize]))
  } else {
    Ok(None)
  }
}

fn length<'a>(c: &mut EvalCtx<'a, '_>, _p: Pos, a: &[Node<'a>]) -> R<'a> {
  let values = arg(a, 0).ok_or_else(|| undefined_value("value"))?;
  Ok(Some(
    c.a.dimension(items_of(values).len() as f64, &EMPTY_UNIT),
  ))
}

fn range<'a>(c: &mut EvalCtx<'a, '_>, _p: Pos, a: &[Node<'a>]) -> R<'a> {
  let start = arg(a, 0);
  let end = arg(a, 1);
  let step = arg(a, 2);
  let (from, to, step_value) = match end {
    Some(e) => {
      let from = num_value(start)?;
      let s = match step {
        Some(s) => num_value(Some(s))?,
        None => 1.0,
      };
      (from, e, s)
    }
    None => (1.0, start.ok_or_else(|| undefined_value("value"))?, 1.0),
  };
  let (to_value, to_unit) = match to {
    Node::Dimension(d) => (d.value, d.unit),
    _ => (f64::NAN, &EMPTY_UNIT as &Unit),
  };
  let mut list = Vec::new();
  let mut i = from;
  let mut guard = 0u32;
  while i <= to_value {
    list.push(c.a.dimension(i, to_unit));
    i += step_value;
    guard += 1;
    if guard > 1_000_000 {
      break;
    }
  }
  Ok(Some(Node::Expression(c.a.expression(&list, None))))
}

fn each<'a>(c: &mut EvalCtx<'a, '_>, pos: Pos, a: &[Node<'a>]) -> R<'a> {
  let list = arg(a, 0).ok_or_else(|| undefined_value("value"))?;
  let rs_arg = arg(a, 1).ok_or_else(|| undefined_value("params"))?;
  // iterator
  let mut iterator: Vec<Node<'a>> = Vec::new();
  let has_value = match list {
    Node::Quoted(_) => false,
    Node::Expression(_) | Node::Value(_) | Node::Rules(_) => true,
    Node::Keyword(_) | Node::Anonymous(_) | Node::UnicodeDescriptor(_) => {
      crate::eval::js_string_value(list)
        .map(|s| !s.is_empty())
        .unwrap_or(false)
    }
    Node::Dimension(d) => d.value != 0.0 && !d.value.is_nan(),
    Node::Color(col) => col.value.get().map(|v| !v.is_empty()).unwrap_or(false),
    _ => false,
  };
  if has_value {
    match list.array_value() {
      Some(items) => {
        for it in items {
          iterator.push(c.eval(*it)?);
        }
      }
      None => {
        // `[tryEval(list.value)]`: the raw value is not a node.
        let s = match list {
          Node::Dimension(d) => num::to_js_string(d.value),
          other => crate::eval::js_value_string(other),
        };
        iterator.push(c.a.anonymous(c.str(&s), Pos::NONE));
      }
    }
  } else if let Node::DetachedRuleset(d) = list {
    let rs = match d.ruleset {
      Node::Ruleset(r) => c.eval_ruleset(r)?,
      other => match c.eval(other)? {
        Node::Ruleset(r) => r,
        _ => return Ok(None),
      },
    };
    iterator = rs.rules_vec();
  } else if let Some(r) = list.as_ruleset() {
    for it in r.rules_vec() {
      iterator.push(c.eval(it)?);
    }
  } else {
    iterator.push(c.eval(list)?);
  }

  let mut value_name: Option<&'a str> = Some("@value");
  let mut key_name: Option<&'a str> = Some("@key");
  let mut index_name: Option<&'a str> = Some("@index");
  let rs: &'a Ruleset<'a> = match rs_arg {
    Node::MixinDefinition(md) => {
      let def = md.mixin.expect("mixin definition has metadata");
      value_name = def.params.first().and_then(|p| p.name);
      key_name = def.params.get(1).and_then(|p| p.name);
      index_name = def.params.get(2).and_then(|p| p.name);
      match md.rules_vec().first() {
        Some(Node::Ruleset(r)) => r,
        _ => md,
      }
    }
    Node::DetachedRuleset(d) => match d.ruleset {
      Node::Ruleset(r) => r,
      _ => return Err(undefined_value("rules")),
    },
    _ => return Err(undefined_value("rules")),
  };

  let mut rules: Vec<Node<'a>> = Vec::new();
  for (i, item) in iterator.iter().enumerate() {
    let (key, value): (Node<'a>, Node<'a>) = if let Node::Declaration(d) = item {
      let k = match d.name {
        DeclName::Str(s) => s,
        DeclName::Parts(p) => match p.first() {
          Some(Node::Keyword(k)) => k.value,
          _ => "",
        },
      };
      (c.a.anonymous(k, Pos::NONE), d.value.get())
    } else {
      (c.a.dimension((i + 1) as f64, &EMPTY_UNIT), *item)
    };
    if item.is_comment() {
      continue;
    }
    let mut new_rules = rs.rules_vec();
    if let Some(vn) = value_name {
      new_rules.push(Node::Declaration(c.a.declaration(
        DeclName::Str(vn),
        value,
        "",
        Merge::None,
        pos,
        false,
        None,
      )));
    }
    if let Some(inn) = index_name {
      let d = c.a.dimension((i + 1) as f64, &EMPTY_UNIT);
      new_rules.push(Node::Declaration(c.a.declaration(
        DeclName::Str(inn),
        d,
        "",
        Merge::None,
        pos,
        false,
        None,
      )));
    }
    if let Some(kn) = key_name {
      new_rules.push(Node::Declaration(c.a.declaration(
        DeclName::Str(kn),
        key,
        "",
        Merge::None,
        pos,
        false,
        None,
      )));
    }
    let sel = c.a.selector_amp(Pos::NONE);
    let r = c.a.ruleset_full(
      Some(c.a.slice(&[sel])),
      Some(c.a.vec_from(&new_rules)),
      rs.strict_imports,
      rs.vis.info(),
      None,
      Pos::NONE,
    );
    rules.push(Node::Ruleset(r));
  }
  let sel = c.a.selector_amp(Pos::NONE);
  let outer = c.a.ruleset_full(
    Some(c.a.slice(&[sel])),
    Some(c.a.vec_from(&rules)),
    rs.strict_imports,
    rs.vis.info(),
    None,
    Pos::NONE,
  );
  Ok(Some(Node::Ruleset(c.eval_ruleset(outer)?)))
}

// ---------------------------------------------------------------------------
// math.js / number.js
// ---------------------------------------------------------------------------

fn math_helper<'a>(
  c: &mut EvalCtx<'a, '_>,
  _p: Pos,
  a: &[Node<'a>],
  f: fn(f64) -> f64,
  unit: Option<&'static str>,
  css_evaluable: bool,
) -> R<'a> {
  let n = arg(a, 0);
  let d = match n {
    Some(Node::Dimension(d)) => d,
    Some(Node::Call(_)) if css_evaluable => return Ok(None),
    _ => return Err(LessError::new("Argument", "argument must be a number")),
  };
  match unit {
    None => Ok(Some(c.a.dimension(f(d.value), d.unit))),
    Some(u) => {
      let (v, _) = unify(d);
      Ok(Some(c.a.dimension_str(f(v), u)))
    }
  }
}

fn round<'a>(c: &mut EvalCtx<'a, '_>, _p: Pos, a: &[Node<'a>]) -> R<'a> {
  let fraction = match arg(a, 1) {
    None => 0.0,
    Some(f) => num_value(Some(f))?,
  };
  let d = match arg(a, 0) {
    Some(Node::Dimension(d)) => d,
    _ => return Err(LessError::new("Argument", "argument must be a number")),
  };
  let digits = if fraction.is_nan() || fraction < 0.0 {
    0
  } else {
    fraction as u32
  };
  Ok(Some(
    c.a.dimension(num::to_fixed_number(d.value, digits), d.unit),
  ))
}

fn min<'a>(c: &mut EvalCtx<'a, '_>, p: Pos, a: &[Node<'a>]) -> R<'a> {
  Ok(min_max(c, p, true, a).ok().flatten())
}
fn max<'a>(c: &mut EvalCtx<'a, '_>, p: Pos, a: &[Node<'a>]) -> R<'a> {
  Ok(min_max(c, p, false, a).ok().flatten())
}

fn min_max<'a>(c: &mut EvalCtx<'a, '_>, _p: Pos, is_min: bool, args: &[Node<'a>]) -> R<'a> {
  let mut args: Vec<Node<'a>> = args.to_vec();
  if args.is_empty() {
    return Err(LessError::new("Argument", "one or more arguments required"));
  }
  let mut order: Vec<&'a Dimension<'a>> = Vec::new();
  let mut values: Vec<(String, usize)> = Vec::new();
  let mut unit_static: Option<String> = None;
  let mut unit_clone: Option<String> = None;
  let unify_with =
    |c: &EvalCtx<'a, '_>, d: &'a Dimension<'a>, uc: &Option<String>| -> (f64, String) {
      if d.unit.numerator.is_empty()
        && d.unit.denominator.is_empty()
        && let Some(u) = uc
      {
        let unit = c.a.unit(&[c.str(u)], &[], None);
        let tmp = Dimension {
          value: d.value,
          unit,
          pos: Cell::new(Pos::NONE),
        };
        let (v, wu) = unify(&tmp);
        return (v, wu.to_string());
      }
      let (v, wu) = unify(d);
      (v, wu.to_string())
    };
  let mut i = 0;
  while i < args.len() {
    let current = match args[i] {
      Node::Dimension(d) => d,
      other => {
        if let Some(items) = other.array_value() {
          args.extend_from_slice(items);
          i += 1;
          continue;
        }
        return Err(LessError::new("Argument", "incompatible types"));
      }
    };
    let (cur_val, cur_unit) = unify_with(c, current, &unit_clone);
    let unit = if cur_unit.is_empty() && unit_static.is_some() {
      unit_static.clone().expect("static unit is present")
    } else {
      cur_unit.clone()
    };
    let order0_unified_empty = order
      .first()
      .map(|o| unify(o).1.to_string().is_empty())
      .unwrap_or(false);
    unit_static = if (!unit.is_empty() && unit_static.is_none())
      || (!unit.is_empty() && order0_unified_empty)
    {
      Some(unit.clone())
    } else {
      unit_static
    };
    unit_clone = if !unit.is_empty() && unit_clone.is_none() {
      Some(css::unit_to_string(current.unit))
    } else {
      unit_clone
    };
    let empty_idx = values.iter().find(|(k, _)| k.is_empty()).map(|(_, v)| *v);
    let j = if empty_idx.is_some() && !unit.is_empty() && Some(&unit) == unit_static.as_ref() {
      empty_idx
    } else {
      values.iter().find(|(k, _)| *k == unit).map(|(_, v)| *v)
    };
    let j = match j {
      None => {
        if unit_static.is_some() && Some(&unit) != unit_static.as_ref() {
          return Err(LessError::new("Argument", "incompatible types"));
        }
        values.push((unit, order.len()));
        order.push(current);
        i += 1;
        continue;
      }
      Some(j) => j,
    };
    let (ref_val, _) = unify_with(c, order[j], &unit_clone);
    if (is_min && cur_val < ref_val) || (!is_min && cur_val > ref_val) {
      order[j] = current;
    }
    i += 1;
  }
  if order.len() == 1 {
    return Ok(Some(Node::Dimension(order[0])));
  }
  let mut ctx = c.css_ctx();
  let parts: Vec<String> = order
    .iter()
    .map(|d| css::to_css(Node::Dimension(d), &mut ctx).unwrap_or_default())
    .collect();
  let sep = if c.s.opts.compress { "," } else { ", " };
  let s = format!(
    "{}({})",
    if is_min { "min" } else { "max" },
    parts.join(sep)
  );
  Ok(Some(c.a.anonymous(c.str(&s), Pos::NONE)))
}

fn convert<'a>(c: &mut EvalCtx<'a, '_>, _p: Pos, a: &[Node<'a>]) -> R<'a> {
  let d = match arg(a, 0) {
    Some(Node::Dimension(d)) => d,
    _ => return Err(LessError::native("val.convertTo is not a function")),
  };
  let target = match arg(a, 1) {
    Some(n) => crate::eval::js_string_value(n).unwrap_or(""),
    None => return Err(undefined_value("value")),
  };
  let (v, u) = convert_to_unit(d.value, d.unit, target);
  Ok(Some(c.a.dimension(v, u.alloc(c.a))))
}

fn pi<'a>(c: &mut EvalCtx<'a, '_>, _p: Pos, _a: &[Node<'a>]) -> R<'a> {
  Ok(Some(c.a.dimension(std::f64::consts::PI, &EMPTY_UNIT)))
}

fn modf<'a>(c: &mut EvalCtx<'a, '_>, _p: Pos, a: &[Node<'a>]) -> R<'a> {
  let x = num_value(arg(a, 0))?;
  let y = num_value(arg(a, 1))?;
  let unit = match arg(a, 0) {
    Some(Node::Dimension(d)) => d.unit,
    _ => &EMPTY_UNIT,
  };
  Ok(Some(c.a.dimension(x % y, unit)))
}

fn pow<'a>(c: &mut EvalCtx<'a, '_>, _p: Pos, a: &[Node<'a>]) -> R<'a> {
  match (arg(a, 0), arg(a, 1)) {
    (Some(Node::Dimension(x)), Some(Node::Dimension(y))) => {
      Ok(Some(c.a.dimension(x.value.powf(y.value), x.unit)))
    }
    _ => Err(LessError::new("Argument", "arguments must be numbers")),
  }
}

fn percentage<'a>(c: &mut EvalCtx<'a, '_>, _p: Pos, a: &[Node<'a>]) -> R<'a> {
  match arg(a, 0) {
    Some(Node::Dimension(d)) => {
      let (v, _) = unify(d);
      Ok(Some(c.a.dimension_str(v * 100.0, "%")))
    }
    _ => Err(LessError::new("Argument", "argument must be a number")),
  }
}

// ---------------------------------------------------------------------------
// string.js
// ---------------------------------------------------------------------------

fn e_fn<'a>(c: &mut EvalCtx<'a, '_>, _p: Pos, a: &[Node<'a>]) -> R<'a> {
  let s = match arg(a, 0) {
    Some(n) => match n {
      Node::Dimension(d) => c.str(&num::to_js_string(d.value)),
      other => crate::eval::js_string_value(other).unwrap_or("undefined"),
    },
    None => return Err(undefined_value("value")),
  };
  Ok(Some(Node::Quoted(c.a.quoted("\"", s, true, Pos::NONE))))
}

pub(crate) fn encode_uri_component(s: &str) -> String {
  encode_uri_impl(s, b"-_.!~*'()")
}

pub(crate) fn encode_uri(s: &str) -> String {
  encode_uri_impl(s, b"-_.!~*'();/?:@&=+$,#")
}

fn encode_uri_impl(s: &str, keep: &[u8]) -> String {
  let mut out = String::with_capacity(s.len());
  for ch in s.chars() {
    if ch.is_ascii_alphanumeric() || (ch.is_ascii() && keep.contains(&(ch as u8))) {
      out.push(ch);
    } else {
      let mut buf = [0u8; 4];
      for b in ch.encode_utf8(&mut buf).bytes() {
        out.push_str(&format!("%{:02X}", b));
      }
    }
  }
  out
}

fn escape<'a>(c: &mut EvalCtx<'a, '_>, _p: Pos, a: &[Node<'a>]) -> R<'a> {
  let v = match arg(a, 0) {
    Some(n) => crate::eval::js_value_string(n),
    None => return Err(undefined_value("value")),
  };
  let mut s = encode_uri(&v);
  s = s
    .cow_replace('=', "%3D")
    .cow_replace(':', "%3A")
    .cow_replace('#', "%23")
    .cow_replace(';', "%3B")
    .cow_replace('(', "%28")
    .cow_replace(')', "%29")
    .into_owned();
  let prevent = s.contains('%');
  let an = c.a.alloc(Anonymous {
    vis: Vis::default(),
    value: c.str(&s),
    pos: Pos::NONE,
    map_lines: false,
    ruleset_like: false,
    prevent_reparse: prevent,
    no_spacing: Cell::new(None),
    is_null: false,
    has_file: false,
  });
  Ok(Some(Node::Anonymous(an)))
}

fn quoted_parts<'a>(n: Option<Node<'a>>) -> (&'a str, &'a str, bool) {
  match n {
    Some(Node::Quoted(q)) => (q.value, q.quote, q.escaped),
    Some(other) => (crate::eval::js_string_value(other).unwrap_or(""), "", true),
    None => ("", "", true),
  }
}

fn replace<'a>(c: &mut EvalCtx<'a, '_>, _p: Pos, a: &[Node<'a>]) -> R<'a> {
  let (string, quote, escaped) = quoted_parts(arg(a, 0));
  let pattern = match arg(a, 1) {
    Some(n) => crate::eval::js_value_string(n),
    None => return Err(undefined_value("value")),
  };
  let replacement = match arg(a, 2) {
    Some(Node::Quoted(q)) => q.value.to_string(),
    Some(other) => css::to_css_plain(other)?,
    None => return Err(undefined_value("type")),
  };
  let flags = arg(a, 3)
    .map(|f| crate::eval::js_value_string(f))
    .unwrap_or_default();
  let result = js_regex_replace(string, &pattern, &replacement, &flags)?;
  Ok(Some(Node::Quoted(c.a.quoted(
    quote,
    c.str(&result),
    escaped,
    Pos::NONE,
  ))))
}

/// `String.prototype.replace(new RegExp(pattern, flags), replacement)`.
pub(crate) fn js_regex_replace(
  s: &str,
  pattern: &str,
  replacement: &str,
  flags: &str,
) -> LResult<String> {
  let mut b = regex::RegexBuilder::new(pattern);
  b.case_insensitive(flags.contains('i'));
  b.multi_line(flags.contains('m'));
  b.dot_matches_new_line(flags.contains('s'));
  let re = b
    .build()
    .map_err(|e| LessError::native(format!("Invalid regular expression: /{}/: {}", pattern, e)))?;
  let global = flags.contains('g');
  let mut out = String::new();
  let mut last = 0;
  for caps in re.captures_iter(s) {
    let m = caps
      .get(0)
      .expect("capture group zero contains the whole match");
    out.push_str(&s[last..m.start()]);
    expand_js_replacement(&caps, s, replacement, &mut out);
    last = m.end();
    if !global {
      break;
    }
  }
  out.push_str(&s[last..]);
  Ok(out)
}

fn expand_js_replacement(caps: &regex::Captures<'_>, input: &str, rep: &str, out: &mut String) {
  let b = rep.as_bytes();
  let mut i = 0;
  let m = caps
    .get(0)
    .expect("capture group zero contains the whole match");
  while i < b.len() {
    if b[i] == b'$' && i + 1 < b.len() {
      match b[i + 1] {
        b'$' => {
          out.push('$');
          i += 2;
          continue;
        }
        b'&' => {
          out.push_str(m.as_str());
          i += 2;
          continue;
        }
        b'`' => {
          out.push_str(&input[..m.start()]);
          i += 2;
          continue;
        }
        b'\'' => {
          out.push_str(&input[m.end()..]);
          i += 2;
          continue;
        }
        d if d.is_ascii_digit() => {
          let mut n = (d - b'0') as usize;
          let mut len = 2;
          if i + 2 < b.len() && b[i + 2].is_ascii_digit() {
            let n2 = n * 10 + (b[i + 2] - b'0') as usize;
            if n2 < caps.len() {
              n = n2;
              len = 3;
            }
          }
          if n > 0 && n < caps.len() {
            if let Some(g) = caps.get(n) {
              out.push_str(g.as_str());
            }
            i += len;
            continue;
          }
        }
        _ => {}
      }
    }
    let ch_len = crate::syntax::matchers::utf8_len(b[i]);
    out.push_str(&rep[i..i + ch_len]);
    i += ch_len;
  }
}

fn format_fn<'a>(c: &mut EvalCtx<'a, '_>, _p: Pos, a: &[Node<'a>]) -> R<'a> {
  let (string, quote, escaped) = quoted_parts(arg(a, 0));
  let mut result = string.to_string();
  for item in a.iter().skip(1) {
    // replace the first /%[sda]/i
    let rb = result.as_bytes();
    let mut found = None;
    for i in 0..rb.len().saturating_sub(1) {
      if rb[i] == b'%' && matches!(rb[i + 1], b's' | b'S' | b'd' | b'D' | b'a' | b'A') {
        found = Some(i);
        break;
      }
    }
    if let Some(i) = found {
      let token = rb[i + 1];
      let value = if matches!(item, Node::Quoted(_)) && (token == b's' || token == b'S') {
        crate::eval::js_string_value(*item)
          .unwrap_or("")
          .to_string()
      } else {
        css::to_css_plain(*item)?
      };
      let value = if token.is_ascii_uppercase() {
        encode_uri_component(&value)
      } else {
        value
      };
      result = format!("{}{}{}", &result[..i], value, &result[i + 2..]);
    }
  }
  let result = result.cow_replace("%%", "%").into_owned();
  Ok(Some(Node::Quoted(c.a.quoted(
    quote,
    c.str(&result),
    escaped,
    Pos::NONE,
  ))))
}

// ---------------------------------------------------------------------------
// types.js
// ---------------------------------------------------------------------------

fn isunit_str<'a>(c: &mut EvalCtx<'a, '_>, n: Option<Node<'a>>, unit: &str) -> R<'a> {
  let b = match n {
    Some(Node::Dimension(d)) => {
      css::unit_to_string(d.unit).cow_to_uppercase() == unit.cow_to_uppercase()
    }
    _ => false,
  };
  Ok(Some(kw(c, b)))
}

fn isunit<'a>(c: &mut EvalCtx<'a, '_>, _p: Pos, a: &[Node<'a>]) -> R<'a> {
  let unit = match arg(a, 1) {
    None => {
      return Err(LessError::new(
        "Argument",
        "missing the required second argument to isunit.",
      ));
    }
    Some(u) => match crate::eval::js_string_value(u) {
      Some(s) => s,
      None => {
        return Err(LessError::new(
          "Argument",
          "Second argument to isunit should be a unit or a string.",
        ));
      }
    },
  };
  isunit_str(c, arg(a, 0), unit)
}

fn unit_fn<'a>(c: &mut EvalCtx<'a, '_>, _p: Pos, a: &[Node<'a>]) -> R<'a> {
  let d = match arg(a, 0) {
    Some(Node::Dimension(d)) => d,
    other => {
      let hint = if matches!(other, Some(Node::Operation(_))) {
        ". Have you forgotten parenthesis?"
      } else {
        ""
      };
      return Err(LessError::new(
        "Argument",
        format!("the first argument to unit must be a number{}", hint),
      ));
    }
  };
  let unit: &'a str = match arg(a, 1) {
    Some(Node::Keyword(k)) => k.value,
    Some(other) => c.str(&css::to_css_plain(other)?),
    None => "",
  };
  Ok(Some(c.a.dimension_str(d.value, unit)))
}

fn get_unit<'a>(c: &mut EvalCtx<'a, '_>, _p: Pos, a: &[Node<'a>]) -> R<'a> {
  match arg(a, 0) {
    Some(Node::Dimension(d)) => {
      let s = css::unit_to_string(d.unit);
      Ok(Some(c.a.anonymous(c.str(&s), Pos::NONE)))
    }
    Some(_) => Ok(Some(c.a.null_anonymous())),
    None => Err(undefined_value("unit")),
  }
}

// ---------------------------------------------------------------------------
// style.js
// ---------------------------------------------------------------------------

fn style<'a>(c: &mut EvalCtx<'a, '_>, p: Pos, a: &[Node<'a>]) -> R<'a> {
  let first = match a.first() {
    Some(f) => *f,
    None => return Ok(None),
  };
  let name = match crate::eval::js_string_value(first) {
    Some(s) => s,
    None => return Ok(None),
  };
  let v = c.a.variable(name, p);
  let r = match c.eval(v) {
    Ok(r) => r,
    Err(_) => return Ok(None),
  };
  let s = match c.to_css(r) {
    Ok(s) => s,
    Err(_) => return Ok(None),
  };
  let out = format!("style({})", s);
  Ok(Some(c.a.anonymous(c.str(&out), Pos::NONE)))
}
