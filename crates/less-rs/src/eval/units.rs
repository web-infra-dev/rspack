//! Unit arithmetic (`tree/unit.js`, `Dimension.operate/convertTo/unify`).

use cow_utils::CowUtils;

use super::{EvalCtx, js_operate, numeric_compare};
use crate::{
  data::{UNIT_GROUPS, unit_factor},
  error::{LResult, LessError},
  tree::*,
};

/// A mutable working copy of a unit.
#[derive(Clone, Debug)]
pub(crate) struct WUnit<'a> {
  pub numerator: smallvec::SmallVec<[&'a str; 2]>,
  pub denominator: smallvec::SmallVec<[&'a str; 2]>,
  pub backup_unit: Option<&'a str>,
}

impl<'a> WUnit<'a> {
  pub fn from(u: &Unit<'a>) -> WUnit<'a> {
    // `unit.clone()` → new Unit(copy(num), copy(den), backupUnit)
    let mut n: smallvec::SmallVec<[&'a str; 2]> = u.numerator.iter().copied().collect();
    n.sort();
    let mut d: smallvec::SmallVec<[&'a str; 2]> = u.denominator.iter().copied().collect();
    d.sort();
    let backup = u.backup_unit.or_else(|| u.numerator.first().copied());
    WUnit {
      numerator: n,
      denominator: d,
      backup_unit: backup,
    }
  }

  pub fn alloc(&self, a: Alloc<'a>) -> &'a Unit<'a> {
    a.unit_raw(&self.numerator, &self.denominator, self.backup_unit)
  }

  /// `unit.cancel()`
  pub fn cancel(&mut self) {
    // one unit on one side: nothing cancels, already sorted
    if (self.denominator.is_empty() && self.numerator.len() <= 1)
      || (self.numerator.is_empty() && self.denominator.len() <= 1)
    {
      return;
    }
    let mut counter: smallvec::SmallVec<[(&'a str, i32); 4]> = smallvec::SmallVec::new();
    for u in &self.numerator {
      match counter.iter_mut().find(|(k, _)| k == u) {
        Some(c) => c.1 += 1,
        None => counter.push((u, 1)),
      }
    }
    for u in &self.denominator {
      match counter.iter_mut().find(|(k, _)| k == u) {
        Some(c) => c.1 -= 1,
        None => counter.push((u, -1)),
      }
    }
    self.numerator.clear();
    self.denominator.clear();
    for (u, c) in counter {
      if c > 0 {
        for _ in 0..c {
          self.numerator.push(u);
        }
      } else if c < 0 {
        for _ in 0..(-c) {
          self.denominator.push(u);
        }
      }
    }
    self.numerator.sort();
    self.denominator.sort();
  }

  pub fn to_string(&self) -> String {
    let mut s = String::new();
    for (i, n) in self.numerator.iter().enumerate() {
      if i > 0 {
        s.push('*');
      }
      s.push_str(n);
    }
    for d in &self.denominator {
      s.push('/');
      s.push_str(d);
    }
    s
  }
}

/// `unit.usedUnits()` → (group, unit) in group order.
pub(crate) fn used_units<'a>(u: &Unit<'a>) -> Vec<(&'static str, &'a str)> {
  let mut result: Vec<(&'static str, &'a str)> = Vec::new();
  for (group_name, group) in UNIT_GROUPS {
    for atomic in u.numerator.iter().chain(u.denominator.iter()) {
      if unit_factor(group, atomic).is_some() && !result.iter().any(|(g, _)| g == group_name) {
        result.push((group_name, atomic));
      }
    }
  }
  result
}

/// `Dimension.convertTo(conversions)` with conversions as (group, target).
pub(crate) fn convert_to<'a>(
  value: f64,
  unit: &Unit<'a>,
  conversions: &[(&str, &'a str)],
) -> (f64, WUnit<'a>) {
  let mut value = value;
  let mut u = WUnit::from(unit);
  for (group_name, target) in conversions {
    let group = match UNIT_GROUPS.iter().find(|(g, _)| g == group_name) {
      Some((_, g)) => *g,
      None => continue,
    };
    let tf = unit_factor(group, target).unwrap_or(f64::NAN);
    for atomic in u.numerator.iter_mut() {
      if let Some(f) = unit_factor(group, atomic) {
        value *= f / tf;
        *atomic = target;
      }
    }
    for atomic in u.denominator.iter_mut() {
      if let Some(f) = unit_factor(group, atomic) {
        value /= f / tf;
        *atomic = target;
      }
    }
  }
  u.cancel();
  (value, u)
}

/// `convertTo(string)`: finds the group of the target unit.
pub(crate) fn convert_to_unit<'a>(
  value: f64,
  unit: &Unit<'a>,
  target: &'a str,
) -> (f64, WUnit<'a>) {
  let mut conv: Vec<(&str, &'a str)> = Vec::new();
  for (g, group) in UNIT_GROUPS {
    if unit_factor(group, target).is_some() {
      conv = vec![(g, target)];
    }
  }
  convert_to(value, unit, &conv)
}

pub(crate) fn unify<'a>(d: &Dimension<'a>) -> (f64, WUnit<'a>) {
  convert_to(
    d.value,
    d.unit,
    &[("length", "px"), ("duration", "s"), ("angle", "rad")],
  )
}

/// `Dimension.compare(other)`
pub(crate) fn dimension_compare(a: &Dimension<'_>, b: &Dimension<'_>) -> Option<i32> {
  let a_empty = a.unit.numerator.is_empty() && a.unit.denominator.is_empty();
  let b_empty = b.unit.numerator.is_empty() && b.unit.denominator.is_empty();
  if a_empty || b_empty {
    return numeric_compare(a.value, b.value);
  }
  let (av, au) = unify(a);
  let (bv, bu) = unify(b);
  if au.to_string().cow_to_uppercase() != bu.to_string().cow_to_uppercase() {
    return None;
  }
  numeric_compare(av, bv)
}

/// `Dimension.operate(context, op, other)`
pub(crate) fn dimension_operate<'a>(
  ctx: &mut EvalCtx<'a, '_>,
  op: &str,
  this: &'a Dimension<'a>,
  other: &'a Dimension<'a>,
) -> LResult<Node<'a>> {
  // Fast path: the result's unit is an operand's unit as `unit.clone()`
  // (sorted lists, backup unit set) would copy it, so that unit is shared.
  let normal = |u: &Unit<'_>| {
    u.numerator.len() <= 1
      && u.denominator.len() <= 1
      && (u.backup_unit.is_some() || u.numerator.is_empty())
  };
  let unitless = |u: &Unit<'_>| u.numerator.is_empty() && u.denominator.is_empty();
  let same = match op {
    "+" | "-" => {
      if unitless(this.unit) {
        (this.unit.backup_unit.is_none() && normal(other.unit)).then_some(other.unit)
      } else if other.unit.numerator.is_empty() && this.unit.denominator.is_empty() {
        normal(this.unit).then_some(this.unit)
      } else {
        // equal units: `convertTo` changes nothing
        (normal(this.unit)
          && this.unit.numerator == other.unit.numerator
          && this.unit.denominator == other.unit.denominator)
          .then_some(this.unit)
      }
    }
    "*" | "/" if unitless(other.unit) => normal(this.unit).then_some(this.unit),
    _ => None,
  };
  if let Some(unit) = same {
    return Ok(
      ctx
        .a
        .dimension(js_operate(op, this.value, other.value), unit),
    );
  }
  // `a / b` of equal simple units: they cancel, the backup unit stays
  if op == "/"
    && this.unit.numerator.len() == 1
    && this.unit.denominator.is_empty()
    && other.unit.denominator.is_empty()
    && this.unit.numerator == other.unit.numerator
  {
    let unit = ctx.a.unit_raw(
      &[],
      &[],
      this.unit.backup_unit.or(Some(this.unit.numerator[0])),
    );
    return Ok(
      ctx
        .a
        .dimension(js_operate(op, this.value, other.value), unit),
    );
  }
  let mut value = js_operate(op, this.value, other.value);
  let mut unit = WUnit::from(this.unit);
  if op == "+" || op == "-" {
    if unit.numerator.is_empty() && unit.denominator.is_empty() {
      unit = WUnit::from(other.unit);
      if let Some(b) = this.unit.backup_unit {
        unit.backup_unit = Some(b);
      }
    } else if other.unit.numerator.is_empty() && unit.denominator.is_empty() {
      // nothing
    } else {
      let used = used_units(this.unit);
      let (ov, ou) = convert_to(other.value, other.unit, &used);
      if ctx.s.opts.strict_units && ou.to_string() != unit.to_string() {
        return Err(LessError::native(format!(
          "Incompatible units. Change the units or use the unit function. Bad units: '{}' and '{}'.",
          unit.to_string(),
          ou.to_string()
        )));
      }
      value = js_operate(op, this.value, ov);
    }
  } else if (op == "*" || op == "/")
    && other.unit.numerator.is_empty()
    && other.unit.denominator.is_empty()
  {
    // by a unitless value: the unit is unchanged (`cancel` of a sorted,
    // already cancelled unit is the identity only for simple units)
    if !(unit.denominator.is_empty() && unit.numerator.len() <= 1) {
      unit.cancel();
    }
  } else if op == "*" {
    unit.numerator.extend_from_slice(other.unit.numerator);
    unit.numerator.sort();
    unit.denominator.extend_from_slice(other.unit.denominator);
    unit.denominator.sort();
    unit.cancel();
  } else if op == "/" {
    unit.numerator.extend_from_slice(other.unit.denominator);
    unit.numerator.sort();
    unit.denominator.extend_from_slice(other.unit.numerator);
    unit.denominator.sort();
    unit.cancel();
  }
  let u = unit.alloc(ctx.a);
  Ok(ctx.a.dimension(value, u))
}
