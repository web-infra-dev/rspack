//! A serde [`Deserializer`] over swc AST expressions.
//!
//! Parser plugins frequently need to read options objects from call
//! expressions (e.g. `import.meta.glob(pattern, options)`). Instead of
//! hand-rolling property lookups for every option, the options can be
//! declared as a plain `#[derive(Deserialize)]` struct and extracted with
//! [`from_expr`], similar to validating with zod in TypeScript:
//!
//! ```ignore
//! #[derive(Debug, Default, Deserialize)]
//! #[serde(rename_all = "camelCase", default)]
//! struct GlobOptions {
//!   #[serde(deserialize_with = "lenient")]
//!   eager: bool,
//!   #[serde(deserialize_with = "lenient")]
//!   import: Option<String>,
//! }
//!
//! let options = from_expr::<GlobOptions>(&arg.expr).unwrap_or_default();
//! ```
//!
//! The semantics intentionally match the previous hand-rolled behavior:
//! - only statically resolvable values are recognized: string/bool/number
//!   literals, template literals without substitutions, regex literals and
//!   nested object literals; anything else is invisible to the schema;
//! - object properties whose key is not statically resolvable are skipped;
//! - with the [`lenient`] field attribute, an unrecognized value falls back
//!   to the field's default instead of failing the whole options object.

use std::{fmt, marker::PhantomData, slice};

use serde::{
  Deserialize, Deserializer,
  de::{
    self, DeserializeSeed, Error, MapAccess, Visitor,
    value::{BorrowedStrDeserializer, StringDeserializer, U32Deserializer},
  },
  forward_to_deserialize_any,
};
use swc_experimental_ecma_ast::{Expr, Lit, ObjectLit, PropName, PropOrSpread, Regex, Span};

use crate::visitors::static_string_from_expr;

/// Error type of [`ExprDeserializer`].
#[derive(Debug)]
pub struct AstDeError {
  message: String,
}

impl fmt::Display for AstDeError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "failed to deserialize options object: {}", self.message)
  }
}

impl std::error::Error for AstDeError {}

impl de::Error for AstDeError {
  fn custom<T: fmt::Display>(msg: T) -> Self {
    Self {
      message: msg.to_string(),
    }
  }
}

/// Deserialize `T` from an AST expression, typically the options argument of
/// a call expression. Returns an error if the expression is not statically
/// deserializable as `T`; call sites usually apply `.unwrap_or_default()` to
/// fall back to the default options, matching the previous behavior where a
/// non-object argument was simply ignored.
pub fn from_expr<'de, 'a, T>(expr: &'de Expr<'a>) -> Result<T, AstDeError>
where
  T: Deserialize<'de>,
{
  T::deserialize(ExprDeserializer {
    expr,
    _ast: PhantomData,
  })
}

/// A `deserialize_with` helper for options fields: an unrecognized value
/// falls back to `T::default()` instead of failing the whole options object,
/// matching the previous behavior where unmatched properties were ignored.
pub fn lenient<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
  D: Deserializer<'de>,
  T: Deserialize<'de> + Default,
{
  Ok(T::deserialize(deserializer).unwrap_or_default())
}

struct ExprDeserializer<'de, 'a> {
  expr: &'de Expr<'a>,
  _ast: PhantomData<&'a ()>,
}

impl<'de, 'a> Deserializer<'de> for ExprDeserializer<'de, 'a> {
  type Error = AstDeError;

  fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    let expr = self.expr;
    if let Some(lit) = expr.as_lit() {
      return match lit {
        Lit::Bool(bool) => visitor.visit_bool(bool.value),
        Lit::Str(str) => visitor.visit_string(str.value.to_string_lossy().into_owned()),
        Lit::Num(num) => visitor.visit_f64(num.value),
        Lit::Null(_) => visitor.visit_unit(),
        Lit::Regex(regex) => visitor.visit_map(RegexMapAccess::new(regex)),
        Lit::BigInt(_) => Err(Error::custom("bigint literals are not supported")),
      };
    }
    if let Some(obj) = expr.as_object() {
      return visitor.visit_map(PropsAccess::new(obj));
    }
    if let Some(tpl) = expr.as_tpl()
      && tpl.exprs.is_empty()
      && tpl.quasis.len() == 1
      && let Some(el) = tpl.quasis.first()
    {
      return visitor.visit_string(el.raw.to_string());
    }
    if is_undefined(expr) {
      return visitor.visit_unit();
    }
    Err(Error::custom("expression is not statically deserializable"))
  }

  fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    if is_undefined(self.expr) || matches!(self.expr.as_lit(), Some(Lit::Null(_))) {
      visitor.visit_none()
    } else {
      visitor.visit_some(self)
    }
  }

  fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    if let Some(obj) = self.expr.as_object() {
      visitor.visit_map(PropsAccess::new(obj))
    } else {
      Err(Error::custom("expected an object literal"))
    }
  }

  fn deserialize_struct<V>(
    self,
    _name: &'static str,
    _fields: &'static [&'static str],
    visitor: V,
  ) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    self.deserialize_map(visitor)
  }

  forward_to_deserialize_any! {
    bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes byte_buf
    unit unit_struct newtype_struct seq tuple tuple_struct enum identifier
    ignored_any
  }
}

fn is_undefined(expr: &Expr) -> bool {
  expr
    .as_ident()
    .is_some_and(|ident| ident.sym == "undefined")
}

struct PropsAccess<'de, 'a> {
  props: slice::Iter<'de, PropOrSpread<'a>>,
  pending_value: Option<&'de Expr<'a>>,
}

impl<'de, 'a> PropsAccess<'de, 'a> {
  fn new(obj: &'de ObjectLit<'a>) -> Self {
    Self {
      props: obj.props.iter(),
      pending_value: None,
    }
  }
}

impl<'de, 'a> MapAccess<'de> for PropsAccess<'de, 'a> {
  type Error = AstDeError;

  fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
  where
    K: DeserializeSeed<'de>,
  {
    for prop in self.props.by_ref() {
      // Spread, shorthand, getter/setter and method properties are not
      // options entries.
      let Some(kv) = prop.as_prop().and_then(|prop| prop.as_key_value()) else {
        continue;
      };
      // Properties whose key is not statically resolvable are invisible.
      let Some(key) = static_prop_name(&kv.key) else {
        continue;
      };
      self.pending_value = Some(&kv.value);
      return seed
        .deserialize(StringDeserializer::<AstDeError>::new(key))
        .map(Some);
    }
    Ok(None)
  }

  fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
  where
    V: DeserializeSeed<'de>,
  {
    let value = self
      .pending_value
      .take()
      .expect("next_value_seed called before next_key_seed");
    seed.deserialize(ExprDeserializer {
      expr: value,
      _ast: PhantomData,
    })
  }
}

/// Statically resolve an object literal property name, mirroring the key
/// handling of the previous hand-rolled options parsing.
fn static_prop_name(key: &PropName) -> Option<String> {
  match key {
    PropName::Ident(ident) => Some(ident.sym.to_string()),
    PropName::Str(str) => Some(str.value.to_string_lossy().into_owned()),
    PropName::Num(num) => Some(num.value.to_string()),
    PropName::Computed(computed) => {
      if let Some(key) = static_string_from_expr(&computed.expr) {
        return Some(key);
      }
      match computed.expr.as_lit()? {
        Lit::Num(num) => Some(num.value.to_string()),
        Lit::Bool(bool) => Some(bool.value.to_string()),
        Lit::Null(_) => Some("null".to_string()),
        _ => None,
      }
    }
    PropName::BigInt(_) => None,
  }
}

/// A regex literal extracted from an options object, with its source span
/// preserved for later diagnostics.
#[derive(Debug)]
pub struct AstRegex {
  pub exp: String,
  pub flags: String,
  pub span: Span,
}

impl<'de> Deserialize<'de> for AstRegex {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    deserializer.deserialize_any(AstRegexVisitor)
  }
}

struct AstRegexVisitor;

impl<'de> Visitor<'de> for AstRegexVisitor {
  type Value = AstRegex;

  fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    formatter.write_str("a regex literal")
  }

  fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
  where
    A: MapAccess<'de>,
  {
    let mut exp = None;
    let mut flags = None;
    let mut start = None;
    let mut end = None;
    while let Some(key) = map.next_key::<String>()? {
      match key.as_str() {
        "exp" => exp = Some(map.next_value::<String>()?),
        "flags" => flags = Some(map.next_value::<String>()?),
        "start" => start = Some(map.next_value::<u32>()?),
        "end" => end = Some(map.next_value::<u32>()?),
        _ => {
          map.next_value::<de::IgnoredAny>()?;
        }
      }
    }
    Ok(AstRegex {
      exp: exp.ok_or_else(|| de::Error::missing_field("exp"))?,
      flags: flags.ok_or_else(|| de::Error::missing_field("flags"))?,
      span: Span::new(
        start.ok_or_else(|| de::Error::missing_field("start"))?,
        end.ok_or_else(|| de::Error::missing_field("end"))?,
      ),
    })
  }
}

/// A [`MapAccess`] that presents a regex literal as a synthesized map of
/// `exp`/`flags`/`start`/`end`, so that [`AstRegex`] can be deserialized
/// through the regular serde data model.
struct RegexMapAccess<'de, 'a> {
  regex: &'de Regex<'a>,
  pending: Option<u8>,
  index: u8,
}

impl<'de, 'a> RegexMapAccess<'de, 'a> {
  const KEYS: [&'static str; 4] = ["exp", "flags", "start", "end"];

  fn new(regex: &'de Regex<'a>) -> Self {
    Self {
      regex,
      pending: None,
      index: 0,
    }
  }
}

impl<'de, 'a> MapAccess<'de> for RegexMapAccess<'de, 'a> {
  type Error = AstDeError;

  fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
  where
    K: DeserializeSeed<'de>,
  {
    let Some(&key) = Self::KEYS.get(self.index as usize) else {
      return Ok(None);
    };
    self.pending = Some(self.index);
    self.index += 1;
    seed
      .deserialize(BorrowedStrDeserializer::<AstDeError>::new(key))
      .map(Some)
  }

  fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
  where
    V: DeserializeSeed<'de>,
  {
    match self.pending.take() {
      Some(0) => seed.deserialize(StringDeserializer::<AstDeError>::new(
        self.regex.exp.to_string(),
      )),
      Some(1) => seed.deserialize(StringDeserializer::<AstDeError>::new(
        self.regex.flags.to_string(),
      )),
      Some(2) => seed.deserialize(U32Deserializer::<AstDeError>::new(self.regex.span.start)),
      Some(3) => seed.deserialize(U32Deserializer::<AstDeError>::new(self.regex.span.end)),
      _ => unreachable!("next_value_seed called before next_key_seed"),
    }
  }
}

#[cfg(test)]
mod tests {
  use indexmap::IndexMap;
  use rustc_hash::FxBuildHasher;
  use serde::Deserialize;
  use swc_experimental_allocator::Allocator;
  use swc_experimental_ecma_ast::EsVersion;
  use swc_experimental_ecma_parser::{
    EsSyntax, Lexer, Parser, StringSource, Syntax, unstable::Capturing,
  };

  use super::*;

  fn parse_expr<'a>(allocator: &'a Allocator, source: &'a str) -> Expr<'a> {
    let lexer = Lexer::new(
      allocator,
      Syntax::Es(EsSyntax::default()),
      EsVersion::EsNext,
      StringSource::new(source),
      None,
    );
    let lexer = Capturing::new(lexer);
    let mut parser = Parser::new_from(allocator, lexer);
    parser
      .parse_expr()
      .expect("failed to parse test expression")
  }

  #[derive(Debug, Default, PartialEq, Deserialize)]
  #[serde(rename_all = "camelCase", default)]
  struct TestOptions {
    #[serde(deserialize_with = "lenient")]
    eager: bool,
    #[serde(deserialize_with = "lenient")]
    case_sensitive: Option<bool>,
    #[serde(deserialize_with = "lenient")]
    import: Option<String>,
  }

  #[test]
  fn deserializes_camel_case_fields_and_defaults() {
    let allocator = Allocator::new();
    let expr = parse_expr(&allocator, "{ eager: true, caseSensitive: false }");
    let options = from_expr::<TestOptions>(&expr).unwrap();
    assert_eq!(
      options,
      TestOptions {
        eager: true,
        case_sensitive: Some(false),
        import: None,
      }
    );

    let expr = parse_expr(&allocator, "{}");
    assert_eq!(
      from_expr::<TestOptions>(&expr).unwrap(),
      TestOptions::default()
    );
  }

  #[test]
  fn unrecognized_values_fall_back_to_default() {
    let allocator = Allocator::new();
    let expr = parse_expr(&allocator, "{ eager: \"yes\", import: 1 }");
    assert_eq!(
      from_expr::<TestOptions>(&expr).unwrap(),
      TestOptions::default()
    );

    let expr = parse_expr(&allocator, "true");
    assert!(from_expr::<TestOptions>(&expr).is_err());
  }

  #[test]
  fn resolves_template_literal_and_static_computed_keys() {
    let allocator = Allocator::new();
    let expr = parse_expr(&allocator, "{ import: `default` }");
    assert_eq!(
      from_expr::<TestOptions>(&expr).unwrap().import,
      Some("default".to_string())
    );

    let expr = parse_expr(&allocator, "{ [\"eager\"]: true }");
    assert!(from_expr::<TestOptions>(&expr).unwrap().eager);
  }

  #[test]
  fn undefined_and_null_mean_absent() {
    let allocator = Allocator::new();
    let expr = parse_expr(&allocator, "{ import: undefined, caseSensitive: null }");
    let options = from_expr::<TestOptions>(&expr).unwrap();
    assert_eq!(options.import, None);
    assert_eq!(options.case_sensitive, None);
  }

  #[test]
  fn skips_properties_with_unresolvable_keys() {
    let allocator = Allocator::new();
    let expr = parse_expr(&allocator, "{ [Symbol()]: 1, eager: true }");
    assert!(from_expr::<TestOptions>(&expr).unwrap().eager);
  }

  #[derive(Debug, Deserialize)]
  #[serde(untagged)]
  enum TestScalar {
    String(String),
    Number(f64),
    Bool(bool),
  }

  #[test]
  fn record_preserves_source_order() {
    let allocator = Allocator::new();
    let expr = parse_expr(&allocator, "{ b: 1, a: 'x', c: true }");
    let record = from_expr::<IndexMap<String, TestScalar, FxBuildHasher>>(&expr).unwrap();
    let keys = record.keys().collect::<Vec<_>>();
    assert_eq!(keys, ["b", "a", "c"]);
    assert!(matches!(record["b"], TestScalar::Number(1.0)));
    assert!(matches!(&record["a"], TestScalar::String(value) if value == "x"));
    assert!(matches!(record["c"], TestScalar::Bool(true)));
  }

  #[derive(Debug, Default, Deserialize)]
  #[serde(rename_all = "camelCase", default)]
  struct TestRegexOptions {
    #[serde(deserialize_with = "lenient")]
    reg_exp: Option<AstRegex>,
  }

  #[test]
  fn deserializes_regex_literals_with_span() {
    let allocator = Allocator::new();
    let expr = parse_expr(&allocator, "{ regExp: /abc/gi }");
    let options = from_expr::<TestRegexOptions>(&expr).unwrap();
    let regex = options.reg_exp.expect("expected a regex");
    assert_eq!(regex.exp, "abc");
    assert_eq!(regex.flags, "gi");
    assert_eq!(regex.span.end - regex.span.start, "/abc/gi".len() as u32);

    let expr = parse_expr(&allocator, "{ regExp: \"nope\" }");
    assert!(
      from_expr::<TestRegexOptions>(&expr)
        .unwrap()
        .reg_exp
        .is_none()
    );
  }
}
