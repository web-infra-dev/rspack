mod eval_binary_expr;
mod eval_cond_expr;
mod eval_lit_expr;
mod eval_tpl_expr;
mod eval_unary_expr;

use rspack_core::DependencyLocation;

pub use self::eval_binary_expr::eval_binary_expression;
pub use self::eval_cond_expr::eval_cond_expression;
pub use self::eval_lit_expr::eval_lit_expr;
pub use self::eval_tpl_expr::{eval_tpl_expression, TemplateStringKind};
pub use self::eval_unary_expr::eval_unary_expression;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
enum Ty {
  Unknown,
  Undefined,
  Null,
  String,
  Number,
  Boolean,
  // RegExp,
  Conditional,
  // TypeArray,
  ConstArray,
  BigInt,
  // TypeIdentifier,
  // TypeWrapped,
  TemplateString,
}

type Boolean = bool;
type Number = f64;
type Bigint = num_bigint::BigInt;
// type Array<'a> = &'a ast::ArrayLit;
// type Regexp<'a> = &'a ast::Regex;
type String = std::string::String;

#[derive(Debug)]
pub struct BasicEvaluatedExpression {
  ty: Ty,
  range: Option<DependencyLocation>,
  falsy: bool,
  truthy: bool,
  side_effects: bool,
  nullish: Option<bool>,
  boolean: Option<Boolean>,
  number: Option<Number>,
  string: Option<String>,
  bigint: Option<Bigint>,
  // regexp: Option<Regexp<'a>>,
  quasis: Option<Vec<BasicEvaluatedExpression>>,
  parts: Option<Vec<BasicEvaluatedExpression>>,
  // array: Option<Array>
  template_string_kind: Option<TemplateStringKind>,

  options: Option<Vec<BasicEvaluatedExpression>>,
}

impl Default for BasicEvaluatedExpression {
  fn default() -> Self {
    Self::new()
  }
}

impl BasicEvaluatedExpression {
  pub fn new() -> Self {
    Self {
      ty: Ty::Unknown,
      range: None,
      falsy: false,
      truthy: false,
      side_effects: true,
      nullish: None,
      boolean: None,
      number: None,
      bigint: None,
      quasis: None,
      parts: None,
      template_string_kind: None,
      options: None,
      string: None,
      // regexp: None,
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

  pub fn is_conditional(&self) -> bool {
    matches!(self.ty, Ty::Conditional)
  }

  pub fn is_string(&self) -> bool {
    matches!(self.ty, Ty::String)
  }

  pub fn is_bool(&self) -> bool {
    matches!(self.ty, Ty::Boolean)
  }

  pub fn is_compile_time_value(&self) -> bool {
    matches!(
      self.ty,
      Ty::Undefined
        | Ty::Null
        | Ty::String
        | Ty::Number
        | Ty::Boolean
        // | Ty::RegExp
        | Ty::ConstArray
        | Ty::BigInt
    )
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
      self.boolean
    }
  }

  pub fn compare_compile_time_value(&self, b: &BasicEvaluatedExpression) -> bool {
    if self.ty != b.ty {
      false
    } else {
      match self.ty {
        Ty::Undefined => matches!(b.ty, Ty::Undefined),
        Ty::Null => matches!(b.ty, Ty::Null),
        Ty::String => {
          b.string.as_ref().expect("should not empty")
            == self.string.as_ref().expect("should not empty")
        }
        Ty::Number => {
          b.number.as_ref().expect("should not empty")
            == self.number.as_ref().expect("should not empty")
        }
        Ty::Boolean => {
          b.boolean.as_ref().expect("should not empty")
            == self.boolean.as_ref().expect("should not empty")
        }
        // Ty::RegExp => std::ptr::eq(
        //   b.regexp.as_ref().expect("should not empty"),
        //   self.regexp.as_ref().expect("should not empty"),
        // ),
        // Ty::ConstArray => {
        // },
        Ty::BigInt => {
          b.bigint.as_ref().expect("should not empty")
            == self.bigint.as_ref().expect("should not empty")
        }
        _ => unreachable!("can only compare compile-time values"),
      }
    }
  }

  pub fn could_have_side_effects(&self) -> bool {
    self.side_effects
  }

  pub fn set_side_effects(&mut self, side_effects: bool) {
    self.side_effects = side_effects
  }

  pub fn options(&self) -> &Vec<BasicEvaluatedExpression> {
    self.options.as_ref().expect("options should not empty")
  }

  pub fn set_options(&mut self, options: Option<Vec<BasicEvaluatedExpression>>) {
    self.ty = Ty::Conditional;
    self.options = options;
    self.side_effects = true;
  }

  pub fn add_options(&mut self, options: Vec<BasicEvaluatedExpression>) {
    if let Some(old) = &mut self.options {
      old.extend(options);
    } else {
      self.ty = Ty::Conditional;
      self.options = Some(options);
      self.side_effects = true;
    }
  }

  pub fn set_bool(&mut self, boolean: Boolean) {
    self.ty = Ty::Boolean;
    self.boolean = Some(boolean);
    self.side_effects = true
  }

  pub fn set_range(&mut self, start: u32, end: u32) {
    self.range = Some(DependencyLocation::new(start, end))
  }

  pub fn set_template_string(
    &mut self,
    quasis: Vec<BasicEvaluatedExpression>,
    parts: Vec<BasicEvaluatedExpression>,
    kind: TemplateStringKind,
  ) {
    self.ty = Ty::TemplateString;
    self.quasis = Some(quasis);
    self.side_effects = parts.iter().any(|p| p.side_effects);
    self.parts = Some(parts);
    self.template_string_kind = Some(kind);
  }

  pub fn set_string(&mut self, string: String) {
    self.ty = Ty::String;
    self.string = Some(string);
    self.side_effects = false;
  }

  pub fn string(&self) -> &String {
    self.string.as_ref().expect("make sure string exists")
  }

  pub fn bool(&self) -> Boolean {
    self.boolean.expect("make sure bool exists")
  }

  pub fn range(&self) -> (u32, u32) {
    let range = self.range.expect("range should not empty");
    (range.start(), range.end())
  }
}

pub fn evaluate_to_string(value: String, start: u32, end: u32) -> BasicEvaluatedExpression {
  let mut eval = BasicEvaluatedExpression::with_range(start, end);
  eval.set_string(value);
  eval
}
