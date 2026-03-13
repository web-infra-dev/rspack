mod eval_array_expr;
mod eval_binary_expr;
mod eval_call_expr;
mod eval_cond_expr;
mod eval_lit_expr;
mod eval_member_expr;
mod eval_new_expr;
mod eval_prop_name;
mod eval_source;
mod eval_tpl_expr;
mod eval_unary_expr;

use bitflags::bitflags;
use num_bigint::BigInt;
use rspack_core::DependencyRange;
use swc_core::{atoms::Atom, common::Span, ecma::ast::Expr};

pub use self::{
  eval_array_expr::eval_array_expression,
  eval_binary_expr::eval_binary_expression,
  eval_call_expr::eval_call_expression,
  eval_cond_expr::eval_cond_expression,
  eval_lit_expr::{eval_bigint, eval_bool, eval_lit_expr, eval_number, eval_str},
  eval_member_expr::eval_member_expression,
  eval_new_expr::eval_new_expression,
  eval_prop_name::eval_prop_name,
  eval_source::eval_source,
  eval_tpl_expr::{TemplateStringKind, eval_tagged_tpl_expression, eval_tpl_expression},
  eval_unary_expr::eval_unary_expression,
};
use crate::visitors::ExportedVariableInfo;

type Boolean = bool;
type Number = f64;
type Bigint = num_bigint::BigInt;
// type Array<'a> = &'a ast::ArrayLit;
type String = std::string::String;
type Regexp = (String, String); // (expr, flags)

#[derive(Debug, Clone)]
struct IdentifierData {
  identifier: Atom,
  root_info: ExportedVariableInfo,
  members: Option<Vec<Atom>>,
  members_optionals: Option<Vec<bool>>,
  member_ranges: Option<Vec<Span>>,
}

#[derive(Debug, Clone)]
struct WrappedData<'a> {
  prefix: Option<Box<BasicEvaluatedExpression<'a>>>,
  postfix: Option<Box<BasicEvaluatedExpression<'a>>>,
  inner_expressions: Vec<BasicEvaluatedExpression<'a>>,
}

#[derive(Debug, Clone)]
struct TemplateStringData<'a> {
  quasis: Vec<BasicEvaluatedExpression<'a>>,
  parts: Vec<BasicEvaluatedExpression<'a>>,
  kind: TemplateStringKind,
}

#[derive(Debug, Clone)]
enum Payload<'a> {
  Unknown,
  Undefined,
  Null,
  String(String),
  Number(Number),
  Boolean(Boolean),
  RegExp(Box<Regexp>),
  Conditional(Vec<BasicEvaluatedExpression<'a>>),
  Array(Vec<BasicEvaluatedExpression<'a>>),
  Wrapped(Box<WrappedData<'a>>),
  ConstArray(Vec<String>),
  BigInt(Bigint),
  Identifier(Box<IdentifierData>),
  TemplateString(Box<TemplateStringData<'a>>),
}

#[derive(Debug, Clone)]
pub struct BasicEvaluatedExpression<'a> {
  // For 'static-lifetime usage, any reference fields must originate from this owned expression.
  owned_expression: Option<Box<Expr>>,
  // During Tpl parsing, this may switch from Some(...) to None, hence separate from owned_expression.
  expression: Option<&'a Expr>,
  payload: Payload<'a>,
  range: Option<DependencyRange>,
  falsy: bool,
  truthy: bool,
  side_effects: bool,
  nullish: Option<bool>,
}

impl Default for BasicEvaluatedExpression<'_> {
  fn default() -> Self {
    Self::new()
  }
}

impl<'a> BasicEvaluatedExpression<'a> {
  pub fn new() -> Self {
    Self {
      owned_expression: None,
      expression: None,
      payload: Payload::Unknown,
      range: None,
      falsy: false,
      truthy: false,
      side_effects: true,
      nullish: None,
    }
  }

  pub fn with_range(start: u32, end: u32) -> Self {
    let mut expr = BasicEvaluatedExpression::new();
    expr.set_range(start, end);
    expr
  }

  // pub fn is_unknown(&self) -> bool {
  //   matches!(self.ty, Ty::Unknown)
  // }

  pub fn is_identifier(&self) -> bool {
    matches!(self.payload, Payload::Identifier(_))
  }

  pub fn is_null(&self) -> bool {
    matches!(self.payload, Payload::Null)
  }

  pub fn is_unknown(&self) -> bool {
    matches!(self.payload, Payload::Unknown)
  }

  pub fn is_undefined(&self) -> bool {
    matches!(self.payload, Payload::Undefined)
  }

  pub fn is_conditional(&self) -> bool {
    matches!(self.payload, Payload::Conditional(_))
  }

  pub fn is_string(&self) -> bool {
    matches!(self.payload, Payload::String(_))
  }

  pub fn is_bool(&self) -> bool {
    matches!(self.payload, Payload::Boolean(_))
  }

  pub fn is_array(&self) -> bool {
    matches!(self.payload, Payload::Array(_))
  }

  pub fn is_const_array(&self) -> bool {
    matches!(self.payload, Payload::ConstArray(_))
  }

  pub fn is_wrapped(&self) -> bool {
    matches!(self.payload, Payload::Wrapped(_))
  }

  pub fn is_number(&self) -> bool {
    matches!(self.payload, Payload::Number(_))
  }

  pub fn is_bigint(&self) -> bool {
    matches!(self.payload, Payload::BigInt(_))
  }

  pub fn is_template_string(&self) -> bool {
    matches!(self.payload, Payload::TemplateString { .. })
  }

  pub fn is_regexp(&self) -> bool {
    matches!(self.payload, Payload::RegExp(_))
  }

  pub fn is_compile_time_value(&self) -> bool {
    matches!(
      self.payload,
      Payload::Undefined
        | Payload::Null
        | Payload::String(_)
        | Payload::Number(_)
        | Payload::Boolean(_)
        | Payload::RegExp(_)
        | Payload::ConstArray(_)
        | Payload::BigInt(_)
    )
  }

  pub fn is_nullish(&self) -> Option<bool> {
    self.nullish
  }

  pub fn is_primitive_type(&self) -> Option<bool> {
    match self.payload {
      Payload::Undefined
      | Payload::Null
      | Payload::String(_)
      | Payload::Number(_)
      | Payload::Boolean(_)
      | Payload::BigInt(_)
      | Payload::Wrapped(_)
      | Payload::TemplateString { .. } => Some(true),
      Payload::RegExp(_) | Payload::Array(_) | Payload::ConstArray(_) => Some(false),
      _ => None,
    }
  }

  pub fn as_number(&self) -> Option<f64> {
    if self.is_bool() {
      Some(if self.bool() { 1_f64 } else { 0_f64 })
    } else if self.is_null() {
      Some(0_f64)
    } else if self.is_string() {
      self.string().parse::<f64>().ok()
    } else if self.is_number() {
      Some(self.number())
    } else {
      None
    }
  }

  pub fn as_int(&self) -> Option<i32> {
    if self.is_bool() {
      Some(if self.bool() { 1_i32 } else { 0_i32 })
    } else if self.is_null() {
      Some(0_i32)
    } else if self.is_string() {
      self.string().parse::<i32>().ok()
    } else if self.is_number() {
      Some(self.number() as i32)
    } else {
      None
    }
  }

  pub fn as_string(&self) -> Option<std::string::String> {
    if self.is_bool() {
      Some(self.bool().to_string())
    } else if self.is_null() {
      Some("null".to_string())
    } else if self.is_undefined() {
      Some("undefined".to_string())
    } else if self.is_string() {
      Some(self.string().clone())
    } else if self.is_number() {
      Some(self.number().to_string())
    } else if self.is_array() {
      let mut arr = Vec::new();
      for item in self.items() {
        if let Some(item) = item.as_string() {
          arr.push(item)
        } else {
          return None;
        }
      }
      Some(format!("[{}]", arr.join(", ")))
    } else if self.is_template_string() {
      let mut s = String::new();
      for p in self.parts() {
        if let Some(p) = p.as_string() {
          s += &p;
        } else {
          return None;
        }
      }
      Some(s)
    } else {
      None
    }
  }

  pub fn as_bool(&self) -> Option<Boolean> {
    if self.truthy {
      Some(true)
    } else if self.falsy || self.nullish == Some(true) || self.is_null() || self.is_undefined() {
      Some(false)
    } else if self.is_bool() {
      Some(self.bool())
    } else if self.is_string() {
      Some(!self.string().is_empty())
    } else if self.is_number() {
      Some(self.number() != 0.0)
    } else {
      None
    }
  }

  pub fn as_nullish(&self) -> Option<bool> {
    let nullish = self.is_nullish();
    if nullish == Some(true) || self.is_null() || self.is_undefined() {
      Some(true)
    } else if nullish == Some(false)
      || self.is_bool()
      || self.is_string()
      || self.is_template_string()
    {
      Some(false)
    } else {
      None
    }
  }

  pub fn compare_compile_time_value(&self, b: &BasicEvaluatedExpression) -> bool {
    match (&self.payload, &b.payload) {
      (Payload::Undefined, Payload::Undefined) => true,
      (Payload::Null, Payload::Null) => true,
      (Payload::String(a), Payload::String(b)) => a == b,
      (Payload::Number(a), Payload::Number(b)) => a == b,
      (Payload::Boolean(a), Payload::Boolean(b)) => a == b,
      (Payload::RegExp(_), Payload::RegExp(_)) => false, // FIXME: maybe we can use std::ptr::eq
      (Payload::BigInt(a), Payload::BigInt(b)) => a == b,
      _ => false,
    }
  }

  pub fn could_have_side_effects(&self) -> bool {
    self.side_effects
  }

  pub fn set_side_effects(&mut self, side_effects: bool) {
    self.side_effects = side_effects
  }

  pub fn set_null(&mut self) {
    self.payload = Payload::Null;
    self.side_effects = false
  }

  pub fn set_undefined(&mut self) {
    self.payload = Payload::Undefined;
    self.side_effects = false;
  }

  pub fn set_number(&mut self, number: Number) {
    self.payload = Payload::Number(number);
    self.side_effects = false;
  }

  pub fn set_bigint(&mut self, bigint: BigInt) {
    self.payload = Payload::BigInt(bigint);
    self.side_effects = false;
  }

  pub fn set_truthy(&mut self) {
    self.falsy = false;
    self.truthy = true;
    self.nullish = Some(false);
  }

  pub fn set_falsy(&mut self) {
    self.falsy = true;
    self.truthy = false;
  }

  pub fn set_nullish(&mut self, nullish: bool) {
    self.nullish = Some(nullish);
    if nullish {
      self.set_falsy()
    }
  }

  pub fn set_items(&mut self, items: Vec<BasicEvaluatedExpression<'a>>) {
    self.side_effects = items.iter().any(|item| item.could_have_side_effects());
    self.payload = Payload::Array(items);
  }

  pub fn set_array(&mut self, array: Vec<String>) {
    self.payload = Payload::ConstArray(array);
    self.side_effects = false;
  }

  pub fn options(&self) -> &Vec<BasicEvaluatedExpression<'_>> {
    match &self.payload {
      Payload::Conditional(options) => options,
      _ => panic!("options should not empty"),
    }
  }

  pub fn set_options(&mut self, options: Option<Vec<BasicEvaluatedExpression<'a>>>) {
    self.payload = Payload::Conditional(options.unwrap_or_default());
    self.side_effects = true;
  }

  pub fn add_options(&mut self, options: Vec<BasicEvaluatedExpression<'a>>) {
    match &mut self.payload {
      Payload::Conditional(old) => old.extend(options),
      _ => {
        self.payload = Payload::Conditional(options);
        self.side_effects = true;
      }
    }
  }

  pub fn set_identifier(
    &mut self,
    name: Atom,
    root_info: ExportedVariableInfo,
    members: Option<Vec<Atom>>,
    members_optionals: Option<Vec<bool>>,
    member_ranges: Option<Vec<Span>>,
  ) {
    self.payload = Payload::Identifier(Box::new(IdentifierData {
      identifier: name,
      root_info,
      members,
      members_optionals,
      member_ranges,
    }));
    self.side_effects = true;
  }

  pub fn set_bool(&mut self, boolean: Boolean) {
    self.payload = Payload::Boolean(boolean);
    self.side_effects = false
  }

  pub fn set_range(&mut self, start: u32, end: u32) {
    self.range = Some(DependencyRange::new(start, end))
  }

  pub fn set_template_string(
    &mut self,
    quasis: Vec<BasicEvaluatedExpression<'a>>,
    parts: Vec<BasicEvaluatedExpression<'a>>,
    kind: TemplateStringKind,
  ) {
    self.side_effects = parts.iter().any(|p| p.side_effects);
    self.payload = Payload::TemplateString(Box::new(TemplateStringData {
      quasis,
      parts,
      kind,
    }));
  }

  pub fn set_string(&mut self, string: String) {
    self.payload = Payload::String(string);
    self.side_effects = false;
  }

  pub fn set_regexp(&mut self, regexp: String, flags: String) {
    self.payload = Payload::RegExp(Box::new((regexp, flags)));
    self.side_effects = false;
  }

  pub fn set_wrapped(
    &mut self,
    prefix: Option<BasicEvaluatedExpression<'a>>,
    postfix: Option<BasicEvaluatedExpression<'a>>,
    inner_expressions: Vec<BasicEvaluatedExpression<'a>>,
  ) {
    self.payload = Payload::Wrapped(Box::new(WrappedData {
      prefix: prefix.map(Box::new),
      postfix: postfix.map(Box::new),
      inner_expressions,
    }));
    self.side_effects = true;
  }

  pub fn string(&self) -> &String {
    match &self.payload {
      Payload::String(string) => string,
      _ => panic!("make sure string exist"),
    }
  }

  pub fn bigint(&self) -> Option<&BigInt> {
    match &self.payload {
      Payload::BigInt(bigint) => Some(bigint),
      _ => None,
    }
  }

  pub fn identifier(&self) -> &Atom {
    assert!(self.is_identifier());
    match &self.payload {
      Payload::Identifier(identifier) => &identifier.identifier,
      _ => panic!("make sure identifier exist"),
    }
  }

  pub fn root_info(&self) -> &ExportedVariableInfo {
    assert!(self.is_identifier());
    match &self.payload {
      Payload::Identifier(identifier) => &identifier.root_info,
      _ => panic!("make sure identifier exist"),
    }
  }

  pub fn members(&self) -> Option<&Vec<Atom>> {
    assert!(self.is_identifier());
    match &self.payload {
      Payload::Identifier(identifier) => identifier.members.as_ref(),
      _ => panic!("make sure identifier exist"),
    }
  }

  pub fn members_optionals(&self) -> Option<&Vec<bool>> {
    assert!(self.is_identifier());
    match &self.payload {
      Payload::Identifier(identifier) => identifier.members_optionals.as_ref(),
      _ => panic!("make sure identifier exist"),
    }
  }

  pub fn member_ranges(&self) -> Option<&Vec<Span>> {
    assert!(self.is_identifier());
    match &self.payload {
      Payload::Identifier(identifier) => identifier.member_ranges.as_ref(),
      _ => panic!("make sure identifier exist"),
    }
  }

  pub fn regexp(&self) -> &Regexp {
    match &self.payload {
      Payload::RegExp(regexp) => regexp,
      _ => panic!("make sure regexp exist"),
    }
  }

  pub fn bool(&self) -> Boolean {
    match &self.payload {
      Payload::Boolean(boolean) => *boolean,
      _ => panic!("make sure bool exist"),
    }
  }

  pub fn range(&self) -> (u32, u32) {
    let range = self.range.expect("range should not empty");
    (range.start, range.end)
  }

  pub fn prefix(&self) -> Option<&BasicEvaluatedExpression<'a>> {
    assert!(self.is_wrapped(), "prefix is only used in wrapped");
    match &self.payload {
      Payload::Wrapped(wrapped) => wrapped.prefix.as_deref(),
      _ => panic!("prefix is only used in wrapped"),
    }
  }

  pub fn postfix(&self) -> Option<&BasicEvaluatedExpression<'a>> {
    assert!(self.is_wrapped(), "postfix is only used in wrapped");
    match &self.payload {
      Payload::Wrapped(wrapped) => wrapped.postfix.as_deref(),
      _ => panic!("postfix is only used in wrapped"),
    }
  }

  pub fn wrapped_inner_expressions(&self) -> Option<&[BasicEvaluatedExpression<'a>]> {
    assert!(
      self.is_wrapped(),
      "wrapped_inner_expressions is only used in wrapped"
    );
    match &self.payload {
      Payload::Wrapped(wrapped) => Some(wrapped.inner_expressions.as_slice()),
      _ => panic!("wrapped_inner_expressions is only used in wrapped"),
    }
  }

  pub fn template_string_kind(&self) -> TemplateStringKind {
    assert!(self.is_template_string());
    match &self.payload {
      Payload::TemplateString(tpl) => tpl.kind,
      _ => panic!("make sure template string exist"),
    }
  }

  pub fn parts(&self) -> &Vec<BasicEvaluatedExpression<'a>> {
    assert!(self.is_template_string());
    match &self.payload {
      Payload::TemplateString(tpl) => &tpl.parts,
      _ => panic!("make sure template string exist"),
    }
  }

  pub fn quasis(&self) -> &Vec<BasicEvaluatedExpression<'a>> {
    assert!(self.is_template_string(),);
    match &self.payload {
      Payload::TemplateString(tpl) => &tpl.quasis,
      _ => panic!("quasis must exists for template string"),
    }
  }

  pub fn items(&self) -> &Vec<BasicEvaluatedExpression<'_>> {
    assert!(self.is_array());
    match &self.payload {
      Payload::Array(items) => items,
      _ => panic!("items must exists for array"),
    }
  }

  pub fn array(&self) -> &Vec<String> {
    assert!(self.is_const_array());
    match &self.payload {
      Payload::ConstArray(array) => array,
      _ => panic!("array must exists for const array"),
    }
  }

  pub fn number(&self) -> Number {
    assert!(self.is_number());
    match &self.payload {
      Payload::Number(number) => *number,
      _ => panic!("number must exists in ty::number"),
    }
  }

  pub fn into_options(self) -> Option<Vec<BasicEvaluatedExpression<'a>>> {
    match self.payload {
      Payload::Conditional(options) => Some(options),
      _ => None,
    }
  }

  pub fn into_wrapped(
    self,
  ) -> Option<(
    Option<BasicEvaluatedExpression<'a>>,
    Option<BasicEvaluatedExpression<'a>>,
    Vec<BasicEvaluatedExpression<'a>>,
  )> {
    match self.payload {
      Payload::Wrapped(tpl) => Some((
        tpl.prefix.map(|p| *p),
        tpl.postfix.map(|p| *p),
        tpl.inner_expressions,
      )),
      _ => None,
    }
  }

  pub fn range_ref(&self) -> Option<&DependencyRange> {
    self.range.as_ref()
  }

  pub fn set_expression(&mut self, expression: Option<&'a Expr>) {
    self.expression = expression;
  }

  pub fn with_expression(mut self, expression: Option<&'a Expr>) -> Self {
    self.expression = expression;
    self
  }

  pub fn expression(&self) -> Option<&'a Expr> {
    self.expression
  }

  pub fn with_owned_expression<F>(expr: Expr, f: F) -> Option<BasicEvaluatedExpression<'static>>
  where
    F: FnOnce(&Expr) -> Option<BasicEvaluatedExpression<'_>>,
  {
    let expr = Box::new(expr);
    let raw_ptr = Box::into_raw(expr);
    // SAFETY: We are the only owner of the Box, and we are converting it to a raw pointer
    let mut basic_evaluated_expression = f(unsafe { &*raw_ptr })?;

    if basic_evaluated_expression.owned_expression.is_none() {
      // SAFETY: If reference fields exist, they must originate from this owned expression.
      basic_evaluated_expression.owned_expression = Some(unsafe { Box::from_raw(raw_ptr) });
    }

    Some(basic_evaluated_expression)
  }
}

pub fn evaluate_to_string<'a>(value: String, start: u32, end: u32) -> BasicEvaluatedExpression<'a> {
  let mut eval = BasicEvaluatedExpression::with_range(start, end);
  eval.set_string(value);
  eval
}

pub fn evaluate_to_number<'a>(value: f64, start: u32, end: u32) -> BasicEvaluatedExpression<'a> {
  let mut eval = BasicEvaluatedExpression::with_range(start, end);
  eval.set_number(value);
  eval
}

pub fn evaluate_to_boolean<'a>(value: bool, start: u32, end: u32) -> BasicEvaluatedExpression<'a> {
  let mut eval = BasicEvaluatedExpression::with_range(start, end);
  eval.set_bool(value);
  eval
}

pub fn evaluate_to_null<'a>(start: u32, end: u32) -> BasicEvaluatedExpression<'a> {
  let mut eval = BasicEvaluatedExpression::with_range(start, end);
  eval.set_null();
  eval
}

pub fn evaluate_to_undefined<'a>(start: u32, end: u32) -> BasicEvaluatedExpression<'a> {
  let mut eval = BasicEvaluatedExpression::with_range(start, end);
  eval.set_undefined();
  eval
}

pub fn evaluate_to_identifier<'a>(
  identifier: Atom,
  root_info: Atom,
  truthy: Option<bool>,
  start: u32,
  end: u32,
) -> BasicEvaluatedExpression<'a> {
  let mut eval = BasicEvaluatedExpression::with_range(start, end);
  eval.set_identifier(
    identifier,
    ExportedVariableInfo::Name(root_info),
    None,
    None,
    None,
  );
  eval.set_side_effects(false);
  match truthy {
    Some(v) => {
      if v {
        eval.set_truthy();
      } else {
        eval.set_falsy();
      }
    }
    None => eval.set_nullish(true),
  };
  eval
}

bitflags! {
  struct RegExpFlag: u8 {
    const FLAG_Y = 1 << 0;
    const FLAG_M = 1 << 1;
    const FLAG_I = 1 << 2;
    const FLAG_G = 1 << 3;
  }
}

pub fn is_valid_reg_exp_flags(flags: &str) -> bool {
  if flags.is_empty() {
    true
  } else if flags.len() > 4 {
    false
  } else {
    let mut remaining = RegExpFlag::empty();
    for c in flags.as_bytes() {
      match *c {
        b'g' => {
          if remaining.contains(RegExpFlag::FLAG_G) {
            return false;
          }
          remaining.insert(RegExpFlag::FLAG_G);
        }
        b'i' => {
          if remaining.contains(RegExpFlag::FLAG_I) {
            return false;
          }
          remaining.insert(RegExpFlag::FLAG_I);
        }
        b'm' => {
          if remaining.contains(RegExpFlag::FLAG_M) {
            return false;
          }
          remaining.insert(RegExpFlag::FLAG_M);
        }
        b'y' => {
          if remaining.contains(RegExpFlag::FLAG_Y) {
            return false;
          }
          remaining.insert(RegExpFlag::FLAG_Y);
        }
        _ => return false,
      }
    }
    true
  }
}
