use std::{error::Error, fmt, sync::Arc};

use rustc_hash::FxHashMap as HashMap;

type HelperFn = Arc<dyn Fn(&str) -> String + Send + Sync>;

#[derive(Clone, Debug)]
pub struct Template<'a> {
  segments: Vec<Segment<'a>>,
}

#[derive(Clone, Debug)]
enum Segment<'a> {
  Text(&'a str),
  Value(Value<'a>),
}

#[derive(Clone, Debug)]
enum Value<'a> {
  Variable(&'a str),
  Helper { helper: &'a str, argument: &'a str },
}

#[derive(Debug, Clone)]
pub enum TemplateError {
  UnclosedTag { value: String },
  InvalidTemplateString { value: String },
  UnknownHelper { name: String },
  MissingValue { name: String },
  TemplateNotFound { name: String },
}

impl fmt::Display for TemplateError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      TemplateError::UnclosedTag { value } => {
        write!(f, "unclosed tag in template string \"{value}\"")
      }
      TemplateError::InvalidTemplateString { value } => {
        write!(f, "invalid template string `{{{{ {value} }}}}`")
      }
      TemplateError::UnknownHelper { name } => write!(f, "invalid helper \"{name}\""),
      TemplateError::MissingValue { name } => write!(f, "missing value for `{name}`"),
      TemplateError::TemplateNotFound { name } => write!(f, "template `{name}` not found"),
    }
  }
}

impl Error for TemplateError {}

impl<'a> Template<'a> {
  pub fn parse(input: &'a str) -> Result<Self, TemplateError> {
    let mut segments = Vec::new();
    let mut cursor = 0usize;
    let s = input;

    while let Some(start) = find_subsequence(s, cursor, "{{") {
      if start > cursor {
        segments.push(Segment::Text(&input[cursor..start]));
      }

      let tag_start = start + 2;
      let end = find_subsequence(s, tag_start, "}}").ok_or_else(|| TemplateError::UnclosedTag {
        value: input.to_string(),
      })?;

      let expression = input[tag_start..end].trim();
      if expression.is_empty() {
        return Err(TemplateError::InvalidTemplateString {
          value: expression.to_string(),
        });
      }

      let segment = Value::parse(expression)?;
      segments.push(Segment::Value(segment));

      cursor = end + 2;
    }

    if cursor < input.len() {
      segments.push(Segment::Text(&input[cursor..]));
    }

    Ok(Self { segments })
  }

  fn render(
    &self,
    engine: &TemplateEngine<'a>,
    ctx: &HashMap<&'static str, String>,
  ) -> Result<String, TemplateError> {
    let mut output = String::new();
    for segment in &self.segments {
      match segment {
        Segment::Text(text) => output.push_str(text),
        Segment::Value(value) => match value {
          Value::Variable(name) => {
            let value = ctx.get(name).ok_or_else(|| TemplateError::MissingValue {
              name: (*name).to_string(),
            })?;
            output.push_str(value);
          }
          Value::Helper { helper, argument } => {
            let value = ctx
              .get(argument)
              .ok_or_else(|| TemplateError::MissingValue {
                name: (*argument).to_string(),
              })?;
            let helper_fn =
              engine
                .helpers
                .get(helper)
                .ok_or_else(|| TemplateError::UnknownHelper {
                  name: (*helper).to_string(),
                })?;
            let rendered = helper_fn(value);
            output.push_str(&rendered);
          }
        },
      }
    }
    Ok(output)
  }
}

impl<'a> Value<'a> {
  fn parse(expression: &'a str) -> Result<Value<'a>, TemplateError> {
    let mut parts = expression.split_whitespace();
    let first = parts
      .next()
      .ok_or_else(|| TemplateError::InvalidTemplateString {
        value: expression.to_string(),
      })?;
    let second = parts.next();

    if parts.next().is_some() {
      return Err(TemplateError::InvalidTemplateString {
        value: expression.to_string(),
      });
    }

    if let Some(arg) = second {
      Ok(Value::Helper {
        helper: first,
        argument: arg,
      })
    } else {
      Ok(Value::Variable(first))
    }
  }
}

#[derive(Clone, Default)]
pub struct TemplateEngine<'a> {
  helpers: HashMap<&'static str, HelperFn>,
  templates: HashMap<String, Template<'a>>,
}

impl<'a> TemplateEngine<'a> {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn register_helper<F>(&mut self, name: &'static str, helper: F)
  where
    F: Fn(&str) -> String + Send + Sync + 'static,
  {
    self.helpers.insert(name, Arc::new(helper));
  }

  pub fn register_template(&mut self, name: String, template: Template<'a>) {
    self.templates.insert(name, template);
  }

  pub fn render(
    &self,
    name: &str,
    ctx: &HashMap<&'static str, String>,
  ) -> Result<String, TemplateError> {
    let template = self
      .templates
      .get(name)
      .ok_or_else(|| TemplateError::TemplateNotFound {
        name: name.to_string(),
      })?;
    template.render(self, ctx)
  }
}

fn find_subsequence(haystack: &str, from: usize, needle: &str) -> Option<usize> {
  haystack[from..].find(needle).map(|off| off + from)
}

#[cfg(test)]
mod tests {
  use heck::ToKebabCase;

  use super::*;

  fn simple_ctx(value: &str) -> HashMap<&'static str, String> {
    let mut map = HashMap::default();
    map.insert("member", value.to_string());
    map
  }

  #[test]
  fn render_plain_and_variable() {
    let template = Template::parse("foo {{ member }} bar").unwrap();
    let mut engine = TemplateEngine::new();
    engine.register_template("test".to_string(), template);

    let rendered = engine.render("test", &simple_ctx("MyButton")).unwrap();
    assert_eq!(rendered, "foo MyButton bar");
  }

  #[test]
  fn render_with_helper() {
    let template = Template::parse("foo/{{ kebabCase member }}").unwrap();
    let mut engine = TemplateEngine::new();
    engine.register_helper("kebabCase", |value| value.to_kebab_case());
    engine.register_template("test".to_string(), template);

    let rendered = engine.render("test", &simple_ctx("MyButton")).unwrap();
    assert_eq!(rendered, "foo/my-button");
  }

  #[test]
  fn render_with_extra_whitespace() {
    let template = Template::parse("foo/{{    kebabCase   member   }}").unwrap();
    let mut engine = TemplateEngine::new();
    engine.register_helper("kebabCase", |value| value.to_kebab_case());
    engine.register_template("test".to_string(), template);

    let rendered = engine.render("test", &simple_ctx("MyButton")).unwrap();
    assert_eq!(rendered, "foo/my-button");
  }

  #[test]
  fn render_variable_with_padding() {
    let template = Template::parse("{{    member   }}").unwrap();
    let mut engine = TemplateEngine::new();
    engine.register_template("test".to_string(), template);

    let rendered = engine.render("test", &simple_ctx("Button")).unwrap();
    assert_eq!(rendered, "Button");
  }

  #[test]
  fn error_on_missing_value() {
    let template = Template::parse("{{ member }}").unwrap();
    let mut engine = TemplateEngine::new();
    engine.register_template("test".to_string(), template);

    let result = engine.render("test", &HashMap::default());
    assert!(matches!(result, Err(TemplateError::MissingValue { .. })));
  }

  #[test]
  fn error_on_unknown_helper() {
    let template = Template::parse("{{ snakeCase member }}").unwrap();
    let mut engine = TemplateEngine::new();
    engine.register_template("test".to_string(), template);

    let result = engine.render("test", &simple_ctx("MyButton"));
    assert!(matches!(result, Err(TemplateError::UnknownHelper { .. })));
  }

  #[test]
  fn error_on_unclosed_tag() {
    let result = Template::parse("foo {{ member");
    assert!(matches!(result, Err(TemplateError::UnclosedTag { .. })));
  }

  #[test]
  fn error_on_invalid_expression() {
    let result = Template::parse("{{ }}");
    assert!(matches!(
      result,
      Err(TemplateError::InvalidTemplateString { .. })
    ));
  }
}
