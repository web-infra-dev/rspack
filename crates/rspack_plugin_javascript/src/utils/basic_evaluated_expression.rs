use std::borrow::Cow;

use rspack_core::{DependencyLocation, SpanExt};
use swc_core::common::Spanned;
use swc_core::ecma::ast;

#[derive(Debug, Clone)]
enum TemplateStringKind {
  Cooked,
  // Raw,
}

#[derive(Debug, Clone)]
enum Ty {
  Unknown,
  // TypeUndefined,
  // TypeNull,
  String,
  // TypeNumber,
  #[allow(dead_code)] // FIXME: remove this later
  Boolean,
  // TypeRegExp,
  Conditional,
  // TypeArray,
  // TypeConstArray,
  // TypeIdentifier,
  // TypeWrapped,
  TemplateString,
}

type Lit = ast::Lit;
type Expr = ast::Expr;
type Boolean = bool;
// type Number = f64;
// type Bigint = num_bigint::BigInt;
// type Regexp = rspack_regex::RspackRegex;
type String<'a> = Cow<'a, str>;

#[derive(Debug)]
pub struct BasicEvaluatedExpression<'a> {
  ty: Ty,
  range: Option<DependencyLocation>,
  falsy: bool,
  truthy: bool,
  side_effects: bool,
  nullish: Option<bool>,
  bool: Option<Boolean>,
  // number: Option<Number>,
  string: Option<String<'a>>,
  // bigint: Option<Bigint>,
  quasis: Option<Vec<BasicEvaluatedExpression<'a>>>,
  parts: Option<Vec<BasicEvaluatedExpression<'a>>>,
  template_string_kind: Option<TemplateStringKind>,

  options: Option<Vec<BasicEvaluatedExpression<'a>>>,
}

impl<'a> BasicEvaluatedExpression<'a> {
  fn new() -> Self {
    Self {
      ty: Ty::Unknown,
      range: None,
      falsy: false,
      truthy: false,
      side_effects: true,
      nullish: None,
      bool: None,
      // number: None,
      // bigint: None,
      quasis: None,
      parts: None,
      template_string_kind: None,
      options: None,
      string: None,
    }
  }

  pub fn is_unknown(&self) -> bool {
    matches!(self.ty, Ty::Unknown)
  }

  pub fn is_conditional(&self) -> bool {
    matches!(self.ty, Ty::Conditional)
  }

  pub fn is_string(&self) -> bool {
    matches!(self.ty, Ty::String)
  }

  pub fn is_bool(&self) -> bool {
    matches!(self.ty, Ty::Boolean)
  }

  pub fn as_string(&self) -> Option<std::string::String> {
    if self.is_bool() {
      Some(self.bool().to_string())
    } else if self.is_string() {
      Some(self.string().to_string())
    } else {
      None
    }
  }

  pub fn as_bool(&self) -> Option<Boolean> {
    if self.truthy {
      Some(true)
    } else if self.falsy || self.nullish == Some(true) {
      Some(false)
    } else {
      self.bool
    }
  }

  pub fn could_have_side_effects(&self) -> bool {
    self.side_effects
  }

  fn set_side_effects(&mut self, side_effects: bool) {
    self.side_effects = side_effects
  }

  pub fn options(&self) -> &Vec<BasicEvaluatedExpression<'a>> {
    self.options.as_ref().expect("options should not empty")
  }

  fn set_options(&mut self, options: Option<Vec<BasicEvaluatedExpression<'a>>>) {
    self.ty = Ty::Conditional;
    self.options = options;
    self.side_effects = true;
  }

  fn add_options(&mut self, options: Vec<BasicEvaluatedExpression<'a>>) {
    if let Some(old) = &mut self.options {
      old.extend(options);
    } else {
      self.ty = Ty::Conditional;
      self.options = Some(options);
      self.side_effects = true;
    }
  }

  pub fn range(&self) -> (u32, u32) {
    let range = self.range.expect("range should not empty");
    (range.start(), range.end())
  }

  /// Please refrain from declaring `set_range`` as public,
  /// as it should remain encapsulated for verification during evaluations.
  fn set_range(&mut self, start: u32, end: u32) {
    self.range = Some(DependencyLocation::new(start, end))
  }

  fn set_template_string(
    &mut self,
    quasis: Vec<BasicEvaluatedExpression<'a>>,
    parts: Vec<BasicEvaluatedExpression<'a>>,
    kind: TemplateStringKind,
  ) {
    self.ty = Ty::TemplateString;
    self.quasis = Some(quasis);
    self.side_effects = parts.iter().any(|p| p.side_effects);
    self.parts = Some(parts);
    self.template_string_kind = Some(kind);
  }

  fn set_string(&mut self, string: String<'a>) {
    self.ty = Ty::String;
    self.string = Some(string);
    self.side_effects = false;
  }

  pub fn string(&self) -> &String {
    self.string.as_ref().expect("make sure string exists")
  }

  pub fn bool(&self) -> Boolean {
    self.bool.expect("make sure bool exists")
  }
}

fn get_simplified_template_result(
  node: &ast::Tpl,
) -> (Vec<BasicEvaluatedExpression>, Vec<BasicEvaluatedExpression>) {
  let mut quasis: Vec<BasicEvaluatedExpression> = vec![];
  let mut parts: Vec<BasicEvaluatedExpression> = vec![];
  for i in 0..node.quasis.len() {
    let quasi_expr = &node.quasis[i];
    // FIXME: `quasi_exp.cooked` -> `quasi_exp[kind]`
    // and the kind is a argument
    let quasi = quasi_expr
      .cooked
      .as_ref()
      .expect("quasic should be not empty");
    if i > 0 {
      let len = parts.len();
      let prev_expr = &mut parts[len - 1];
      let expr = evaluate_expression(&node.exprs[i - 1]);
      if !expr.could_have_side_effects()
        && let Some(str) = expr.as_string()
      {
        prev_expr.set_string(Cow::Owned(format!(
          "{}{}{}",
          prev_expr.string(),
          str,
          quasi
        )));
        prev_expr.set_range(prev_expr.range().0, prev_expr.range().1);
        // prev_expr.set_expression(None);
        continue;
      }
      parts.push(expr);
    }

    let part = || {
      let mut part = BasicEvaluatedExpression::new();
      part.set_string(Cow::Borrowed(quasi.as_str()));
      part.set_range(quasi_expr.span().real_lo(), quasi_expr.span_hi().0);
      part
    };
    // part.set_expression(Some(quasi_expr));
    quasis.push(part());
    parts.push(part())
  }

  (quasis, parts)
}

// same as `JavascriptParser._initializeEvaluating` in webpack
fn evaluating(expr: &Expr) -> Option<BasicEvaluatedExpression<'_>> {
  match expr {
    Expr::Tpl(tpl) => {
      let (quasis, mut parts) = get_simplified_template_result(tpl);
      if parts.len() == 1 {
        let mut part = parts.remove(0);
        part.set_range(tpl.span().real_lo(), tpl.span().hi().0);
        Some(part)
      } else {
        let mut res = BasicEvaluatedExpression::new();
        res.set_range(tpl.span().real_lo(), tpl.span().hi().0);
        res.set_template_string(quasis, parts, TemplateStringKind::Cooked);
        Some(res)
      }
    }
    Expr::Lit(Lit::Str(str)) => {
      let mut res = BasicEvaluatedExpression::new();
      res.set_range(str.span().real_lo(), str.span_hi().0);
      res.set_string(Cow::Borrowed(str.value.as_str()));
      Some(res)
    }
    Expr::Cond(cond) => {
      let condition = evaluate_expression(&cond.test);
      let condition_value = condition.as_bool();
      let mut res;
      if let Some(bool) = condition_value {
        if bool {
          res = evaluate_expression(&cond.cons)
        } else {
          res = evaluate_expression(&cond.alt)
        };
        if condition.is_conditional() {
          res.set_side_effects(true)
        }
      } else {
        let cons = evaluate_expression(&cond.cons);
        let alt = evaluate_expression(&cond.alt);
        res = BasicEvaluatedExpression::new();
        if cons.is_conditional() {
          res.set_options(cons.options)
        } else {
          res.set_options(Some(vec![cons]))
        }
        if alt.is_conditional() {
          if let Some(options) = alt.options {
            res.add_options(options)
          }
        } else {
          res.add_options(vec![alt])
        }
      }
      res.set_range(cond.span.lo.0, cond.span.hi.0);
      Some(res)
    }
    _ => None,
  }
}

// same as `JavascriptParser.evaluateExpression` in webpack
pub fn evaluate_expression(expr: &Expr) -> BasicEvaluatedExpression<'_> {
  match evaluating(expr) {
    Some(evaluated) => evaluated,
    None => {
      let mut evaluated = BasicEvaluatedExpression::new();
      evaluated.set_range(expr.span().real_lo(), expr.span_hi().0);
      evaluated
    }
  }
}
