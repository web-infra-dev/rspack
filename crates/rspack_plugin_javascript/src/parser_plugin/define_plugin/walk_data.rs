use std::{borrow::Cow, sync::Arc};

use itertools::Itertools as _;
use memchr::memchr_iter;
use rspack_error::Diagnostic;
use rspack_util::json_stringify;
use rustc_hash::FxHashMap;
use serde_json::{Map, Value};
use swc_core::common::Span;

use super::{
  ConflictingValuesError, DefineValue,
  utils::{code_to_string, gen_const_dep},
};
use crate::{utils::eval::BasicEvaluatedExpression, visitors::JavascriptParser};

#[inline]
fn strip_typeof_operator_prefix(key: &str) -> Option<&str> {
  let rest = key.strip_prefix("typeof")?;
  let whitespace_end = rest
    .find(|ch: char| !ch.is_whitespace())
    .unwrap_or(rest.len());
  if whitespace_end == 0 {
    return None;
  }
  Some(&rest[whitespace_end..])
}

type OnEvaluateIdentifier = dyn Fn(
    &DefineRecord,
    &mut JavascriptParser,
    &str, /* Ident */
    u32,  /* start */
    u32,  /* end */
  ) -> Option<BasicEvaluatedExpression<'static>>
  + Send
  + Sync;

type OnEvaluateTypeof = dyn Fn(
    &DefineRecord,
    &mut JavascriptParser,
    u32, /* start */
    u32, /* end */
  ) -> Option<BasicEvaluatedExpression<'static>>
  + Send
  + Sync;

type OnExpression = dyn Fn(
    &DefineRecord,
    &mut JavascriptParser,
    Span,
    u32,  /* replace start */
    u32,  /* replace end */
    &str, /* for name */
  ) -> Option<bool>
  + Send
  + Sync;

type OnTypeof = dyn Fn(&DefineRecord, &mut JavascriptParser, u32 /* start */, u32 /* end */) -> Option<bool>
  + Send
  + Sync;

pub struct DefineRecord {
  code: Value,
  pub on_evaluate_identifier: Option<Box<OnEvaluateIdentifier>>,
  pub on_evaluate_typeof: Option<Box<OnEvaluateTypeof>>,
  pub on_expression: Option<Box<OnExpression>>,
  pub on_typeof: Option<Box<OnTypeof>>,
}

impl std::fmt::Debug for DefineRecord {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    macro_rules! debug_fn_field {
      ($field:expr) => {{
        if $field.is_some() {
          &"Some(..)"
        } else {
          &"None"
        }
      }};
    }
    f.debug_struct("DefineRecord")
      .field("code", &self.code)
      .field(
        "on_evaluate_identifier",
        debug_fn_field!(self.on_evaluate_identifier),
      )
      .field(
        "on_evaluate_typeof",
        debug_fn_field!(self.on_evaluate_typeof),
      )
      .field("on_expression", debug_fn_field!(self.on_expression))
      .field("on_typeof", debug_fn_field!(self.on_typeof))
      .finish_non_exhaustive()
  }
}

impl DefineRecord {
  fn from_code(code: Value) -> DefineRecord {
    Self {
      code,
      on_evaluate_identifier: None,
      on_evaluate_typeof: None,
      on_expression: None,
      on_typeof: None,
    }
  }

  fn with_on_evaluate_identifier(
    mut self,
    on_evaluate_identifier: Box<OnEvaluateIdentifier>,
  ) -> Self {
    self.on_evaluate_identifier = Some(on_evaluate_identifier);
    self
  }

  fn with_on_evaluate_typeof(mut self, on_evaluate_typeof: Box<OnEvaluateTypeof>) -> Self {
    self.on_evaluate_typeof = Some(on_evaluate_typeof);
    self
  }

  fn with_on_expression(mut self, on_expression: Box<OnExpression>) -> Self {
    self.on_expression = Some(on_expression);
    self
  }

  fn with_on_typeof(mut self, on_typeof: Box<OnTypeof>) -> Self {
    self.on_typeof = Some(on_typeof);
    self
  }
}

#[derive(Default)]
pub struct ObjectDefineRecord {
  object: Value,
  pub on_evaluate_identifier: Option<Box<OnObjectEvaluateIdentifier>>,
  pub on_expression: Option<Box<OnObjectExpression>>,
}

impl std::fmt::Debug for ObjectDefineRecord {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ObjectDefineRecord")
      .field("object", &self.object)
      .finish_non_exhaustive()
  }
}

type OnObjectEvaluateIdentifier = dyn Fn(
    &ObjectDefineRecord,
    &mut JavascriptParser,
    &str, /* Ident */
    u32,  /* start */
    u32,  /* end */
  ) -> Option<BasicEvaluatedExpression<'static>>
  + Send
  + Sync;

type OnObjectExpression = dyn Fn(
    &ObjectDefineRecord,
    &mut JavascriptParser,
    Span,
    u32,  /* replace start */
    u32,  /* replace end */
    &str, /* for name */
  ) -> Option<bool>
  + Send
  + Sync;

impl ObjectDefineRecord {
  fn from_code(obj: Value) -> Self {
    assert!(matches!(obj, Value::Array(_) | Value::Object(_)));
    Self {
      object: obj,
      on_evaluate_identifier: None,
      on_expression: None,
    }
  }

  fn with_on_evaluate_identifier(
    mut self,
    on_evaluate_identifier: Box<OnObjectEvaluateIdentifier>,
  ) -> Self {
    self.on_evaluate_identifier = Some(on_evaluate_identifier);
    self
  }

  fn with_on_expression(mut self, on_expression: Box<OnObjectExpression>) -> Self {
    self.on_expression = Some(on_expression);
    self
  }
}

#[derive(Debug, Default)]
pub struct WalkData {
  pub tiling_definitions: FxHashMap<String, String>,
  pub diagnostics: Vec<Diagnostic>,
  pub can_rename: FxHashMap<Arc<str>, Option<Arc<str>>>,
  pub define_record: FxHashMap<Arc<str>, DefineRecord>,
  pub typeof_define_record: FxHashMap<Arc<str>, DefineRecord>,
  pub object_define_record: FxHashMap<Arc<str>, ObjectDefineRecord>,
}

impl WalkData {
  pub fn new(definitions: &DefineValue) -> Self {
    let mut data = Self::default();
    data.setup_value_cache(definitions.iter(), "".into());
    data.setup_record(definitions);
    data
  }

  fn setup_value_cache<'d, 's>(
    &mut self,
    definitions: impl Iterator<Item = (&'d String, &'d Value)>,
    prefix: Cow<'s, str>,
  ) {
    definitions.for_each(|(key, value)| {
      let name = format!("{prefix}{key}");
      let value_str = value.to_string();
      if let Some(prev) = self.tiling_definitions.get(&name)
        && !prev.eq(&value_str)
      {
        self.diagnostics.push(
          ConflictingValuesError(format!("{prefix}{key}"), prev.clone(), value_str)
            .into_diagnostic(),
        );
      } else {
        self.tiling_definitions.insert(name, value_str);
      }
      if let Some(value) = value.as_object() {
        self.setup_value_cache(value.iter(), Cow::Owned(format!("{prefix}{key}.")))
      } else if let Some(value) = value.as_array() {
        let indexes = (0..value.len())
          .map(|index| {
            let mut index_buffer = rspack_util::itoa::Buffer::new();
            index_buffer.format(index).to_string()
          })
          .collect_vec();
        let iter = indexes.iter().zip(value.iter());
        self.setup_value_cache(iter, Cow::Owned(format!("{prefix}{key}.")))
      }
    });
  }

  fn setup_record(&mut self, definitions: &DefineValue) {
    fn apply_define_key(prefix: &str, key: &str, walk_data: &mut WalkData) {
      let mut separator_indices = memchr_iter(b'.', key.as_bytes());
      let Some(first_index) = separator_indices.next() else {
        return;
      };

      let mut full_key = String::with_capacity(prefix.len() + key.len());
      full_key.push_str(prefix);
      let mut segment_start = 0;
      let first_key: Arc<str> = Arc::from(&key[..first_index]);

      for index in std::iter::once(first_index).chain(separator_indices) {
        full_key.push_str(&key[segment_start..index]);
        walk_data
          .can_rename
          .insert(Arc::from(full_key.as_str()), Some(first_key.clone()));
        full_key.push('.');
        segment_start = index + 1;
      }
    }

    fn apply_define(key: &str, code: &Value, walk_data: &mut WalkData) {
      let stripped_typeof_key = strip_typeof_operator_prefix(key);
      let is_typeof = stripped_typeof_key.is_some();
      let key: Arc<str> = Arc::from(stripped_typeof_key.unwrap_or(key));
      let mut define_record = DefineRecord::from_code(code.clone());
      if !is_typeof {
        walk_data.can_rename.insert(key.clone(), None);
        define_record = define_record
          .with_on_evaluate_identifier(Box::new(move |record, parser, _ident, start, end| {
            parser
              .evaluate(
                code_to_string(&record.code, None, None).into_owned(),
                "DefinePlugin",
              )
              .map(|mut evaluated| {
                evaluated.set_range(start, end);
                evaluated
              })
          }))
          .with_on_expression(Box::new(
            move |record, parser, span, start, end, for_name| {
              let code = code_to_string(&record.code, Some(!parser.is_asi_position(span.lo)), None);
              for dep in gen_const_dep(parser, code, for_name, start, end) {
                parser.add_presentational_dependency(dep);
              }

              Some(true)
            },
          ));
      }

      define_record = define_record
        .with_on_evaluate_typeof(Box::new(move |record, parser, start, end| {
          let code = code_to_string(&record.code, None, None);
          let typeof_code = if is_typeof {
            code
          } else {
            Cow::Owned(format!("typeof ({code})"))
          };
          parser
            .evaluate(typeof_code.into_owned(), "DefinePlugin")
            .map(|mut evaluated| {
              evaluated.set_range(start, end);
              evaluated
            })
        }))
        .with_on_typeof(Box::new(move |record, parser, start, end| {
          let code = code_to_string(&record.code, None, None);
          let typeof_code = if is_typeof {
            code
          } else {
            Cow::Owned(format!("typeof ({code})"))
          };
          parser
            .evaluate(typeof_code.to_string(), "DefinePlugin")
            .and_then(|evaluated| {
              if !evaluated.is_string() {
                return None;
              }
              debug_assert!(!parser.in_short_hand);
              for dep in gen_const_dep(
                parser,
                Cow::Owned(json_stringify(evaluated.string())),
                "",
                start,
                end,
              ) {
                parser.add_presentational_dependency(dep);
              }
              Some(true)
            })
        }));

      if is_typeof {
        walk_data.typeof_define_record.insert(key, define_record);
      } else {
        walk_data.define_record.insert(key, define_record);
      }
    }

    fn object_evaluate_identifier(start: u32, end: u32) -> BasicEvaluatedExpression<'static> {
      let mut evaluated = BasicEvaluatedExpression::new();
      evaluated.set_truthy();
      evaluated.set_side_effects(false);
      evaluated.set_range(start, end);
      evaluated
    }

    fn apply_array_define(key: &str, obj: &[Value], walk_data: &mut WalkData) {
      let key = Arc::<str>::from(key);
      walk_data.can_rename.insert(key.clone(), None);
      let define_record = ObjectDefineRecord::from_code(Value::Array(obj.to_owned()))
        .with_on_evaluate_identifier(Box::new(move |_, _, _, start, end| {
          Some(object_evaluate_identifier(start, end))
        }))
        .with_on_expression(Box::new(
          move |record, parser, span, start, end, for_name| {
            let code = code_to_string(&record.object, Some(!parser.is_asi_position(span.lo)), None);
            for dep in gen_const_dep(parser, code, for_name, start, end) {
              parser.add_presentational_dependency(dep);
            }
            Some(true)
          },
        ));
      walk_data.object_define_record.insert(key, define_record);
    }

    fn apply_object_define(key: &str, obj: &Map<String, Value>, walk_data: &mut WalkData) {
      let key = Arc::<str>::from(key);
      walk_data.can_rename.insert(key.clone(), None);
      let define_record = ObjectDefineRecord::from_code(Value::Object(obj.clone()))
        .with_on_evaluate_identifier(Box::new(move |_, _, _, start, end| {
          Some(object_evaluate_identifier(start, end))
        }))
        .with_on_expression(Box::new(
          move |record, parser, span, start, end, for_name| {
            let code = code_to_string(
              &record.object,
              Some(!parser.is_asi_position(span.lo)),
              parser.destructuring_assignment_properties.get(&span),
            );
            for dep in gen_const_dep(parser, code, for_name, start, end) {
              parser.add_presentational_dependency(dep);
            }
            Some(true)
          },
        ));
      walk_data.object_define_record.insert(key, define_record);
    }

    fn walk_code(code: &Value, prefix: &mut String, key: &str, walk_data: &mut WalkData) {
      let prefix_len = prefix.len();
      if let Some(array) = code.as_array() {
        prefix.push_str(key);
        let key_len = prefix.len();
        prefix.push('.');
        walk_array(array, prefix, walk_data);
        prefix.truncate(key_len);
        apply_array_define(prefix.as_str(), array, walk_data);
      } else if let Some(obj) = code.as_object() {
        prefix.push_str(key);
        let key_len = prefix.len();
        prefix.push('.');
        walk_object(obj, prefix, walk_data);
        prefix.truncate(key_len);
        apply_object_define(prefix.as_str(), obj, walk_data);
      } else {
        apply_define_key(prefix.as_str(), key, walk_data);
        prefix.push_str(key);
        apply_define(prefix.as_str(), code, walk_data);
      }
      prefix.truncate(prefix_len);
    }

    fn walk_array(arr: &[Value], prefix: &mut String, walk_data: &mut WalkData) {
      let mut index_buffer = rspack_util::itoa::Buffer::new();
      for (index, code) in arr.iter().enumerate() {
        walk_code(code, prefix, index_buffer.format(index), walk_data);
      }
    }

    fn walk_object(obj: &Map<String, Value>, prefix: &mut String, walk_data: &mut WalkData) {
      for (key, code) in obj {
        walk_code(code, prefix, key, walk_data);
      }
    }

    let mut prefix = String::new();
    for (key, code) in definitions {
      walk_code(code, &mut prefix, key, self);
    }
  }
}
