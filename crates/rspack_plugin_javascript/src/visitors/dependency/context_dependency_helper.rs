use std::borrow::Cow;

use itertools::Itertools;
use once_cell::sync::Lazy;
use regex::Regex;
use rspack_core::parse_resource;
use rspack_error::Severity;
use rspack_util::json_stringify;

use super::create_traceable_error;
use crate::utils::eval::{BasicEvaluatedExpression, TemplateStringKind};

// FIXME: delete this after `parserOptions.wrappedContextRegExp.source`
const DEFAULT_WRAPPED_CONTEXT_REGEXP: &str = ".*";

pub fn create_context_dependency(
  param: &BasicEvaluatedExpression,
  parser: &mut crate::visitors::JavascriptParser,
) -> ContextModuleScanResult {
  if param.is_template_string() {
    let quasis = param.quasis();
    let Some(prefix) = quasis.first() else {
      unreachable!("the len of quasis should be great than 0")
    };
    // SAFETY: the type of `quasis` must be string
    let prefix_raw = prefix.string();
    let postfix_raw = if quasis.len() > 1 {
      Cow::Borrowed(quasis[quasis.len() - 1].string())
    } else {
      Cow::Owned(String::new())
    };

    let (context, prefix) = split_context_from_prefix(prefix_raw.to_string());
    let (postfix, query, fragment) = match parse_resource(&postfix_raw) {
      Some(data) => (
        data.path.to_string_lossy().to_string(),
        data.query.unwrap_or_default(),
        data.fragment.unwrap_or_default(),
      ),
      None => (postfix_raw.to_string(), String::new(), String::new()),
    };

    let reg = format!(
      "^{}{}{}{}$",
      quote_meta(&prefix),
      DEFAULT_WRAPPED_CONTEXT_REGEXP,
      quasis[1..quasis.len() - 1]
        .iter()
        .map(|q| quote_meta(q.string().as_str()) + DEFAULT_WRAPPED_CONTEXT_REGEXP)
        .join(""),
      quote_meta(&postfix)
    );

    let mut replaces = Vec::new();
    let parts = param.parts();
    for (i, part) in parts.iter().enumerate() {
      if i % 2 == 0 {
        if i == 0 {
          let value = format!(
            "{}{prefix}",
            match param.template_string_kind() {
              TemplateStringKind::Cooked => "`",
              TemplateStringKind::Raw => "String.raw`",
            }
          );
          replaces.push((value, param.range().0, part.range().1));
        } else if i == parts.len() - 1 {
          let value = format!("{postfix}`");
          replaces.push((value, part.range().0, param.range().1));
        } else {
          let value = match param.template_string_kind() {
            TemplateStringKind::Cooked => {
              json_stringify(part.string()).trim_matches('"').to_owned()
            }
            TemplateStringKind::Raw => part.string().to_owned(),
          };
          let range = part.range();
          replaces.push((value, range.0, range.1));
        }
      }
    }

    if parser.javascript_options.wrapped_context_critical {
      let range = param.range();
      parser.warning_diagnostics.push(Box::new(
        create_traceable_error(
          "Critical dependency".into(),
          "a part of the request of a dependency is an expression".to_string(),
          parser.source_file,
          rspack_core::ErrorSpan::new(range.0, range.1),
        )
        .with_severity(Severity::Warn),
      ));
    }

    ContextModuleScanResult {
      context,
      reg,
      query,
      fragment,
      replaces,
    }
  } else if param.is_wrapped()
    && let prefix_is_string = param
      .prefix()
      .map(|prefix| prefix.is_string())
      .unwrap_or_default()
    && let postfix_is_string = param
      .postfix()
      .map(|postfix| postfix.is_string())
      .unwrap_or_default()
    && (prefix_is_string || postfix_is_string)
  {
    let (prefix_raw, prefix_range) = if prefix_is_string {
      let prefix = param.prefix().expect("must exist");
      (Cow::Borrowed(prefix.string()), Some(prefix.range()))
    } else {
      (Cow::Owned(String::new()), None)
    };
    let (postfix_raw, postfix_range) = if postfix_is_string {
      let postfix = param.postfix().expect("must exist");
      (Cow::Borrowed(postfix.string()), Some(postfix.range()))
    } else {
      (Cow::Owned(String::new()), None)
    };

    let (context, prefix) = split_context_from_prefix(prefix_raw.to_string());
    let (postfix, query, fragment) = match parse_resource(&postfix_raw) {
      Some(data) => (
        data.path.to_string_lossy().to_string(),
        data.query.unwrap_or_default(),
        data.fragment.unwrap_or_default(),
      ),
      None => (postfix_raw.to_string(), String::new(), String::new()),
    };

    let reg = format!(
      "^{}{DEFAULT_WRAPPED_CONTEXT_REGEXP}{}$",
      quote_meta(&prefix),
      quote_meta(&postfix)
    );

    let mut replaces = Vec::new();
    if let Some(prefix_range) = prefix_range {
      replaces.push((json_stringify(&prefix), prefix_range.0, prefix_range.1))
    }
    if let Some(postfix_range) = postfix_range {
      replaces.push((json_stringify(&postfix), postfix_range.0, postfix_range.1))
    }

    if parser.javascript_options.wrapped_context_critical {
      let range = param.range();
      parser.warning_diagnostics.push(Box::new(
        create_traceable_error(
          "Critical dependency".into(),
          "a part of the request of a dependency is an expression".to_string(),
          parser.source_file,
          rspack_core::ErrorSpan::new(range.0, range.1),
        )
        .with_severity(Severity::Warn),
      ));
    }

    ContextModuleScanResult {
      context,
      reg,
      query,
      fragment,
      replaces,
    }
    // TODO: handle `param.wrappedInnerExpressions`
  } else {
    if parser.javascript_options.expr_context_critical {
      let range = param.range();
      parser.warning_diagnostics.push(Box::new(
        create_traceable_error(
          "Critical dependency".into(),
          "the request of a dependency is an expression".to_string(),
          parser.source_file,
          rspack_core::ErrorSpan::new(range.0, range.1),
        )
        .with_severity(Severity::Warn),
      ));
    }
    ContextModuleScanResult {
      context: String::from("."),
      reg: String::new(),
      query: String::new(),
      fragment: String::new(),
      replaces: Vec::new(),
    }
  }
}

pub struct ContextModuleScanResult {
  pub context: String,
  pub reg: String,
  pub query: String,
  pub fragment: String,
  pub replaces: Vec<(String, u32, u32)>,
}

pub(super) fn split_context_from_prefix(prefix: String) -> (String, String) {
  if let Some(idx) = prefix.rfind('/') {
    (prefix[..idx].to_string(), format!(".{}", &prefix[idx..]))
  } else {
    (".".to_string(), prefix)
  }
}

static META_REG: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"[-\[\]\\/{}()*+?.^$|]").expect("Failed to initialize `MATCH_RESOURCE_REGEX`")
});

pub fn quote_meta(str: &str) -> Cow<str> {
  META_REG.replace_all(str, "\\$0")
}
