use std::borrow::Cow;

use itertools::Itertools;
use rspack_core::parse_resource;
use swc_core::ecma::ast::Expr;

use super::context_helper::{quote_meta, split_context_from_prefix};
use super::ContextModuleScanResult;
use crate::utils::eval::BasicEvaluatedExpression;

// FIXME: delete this after `parserOptions.wrappedContextRegExp.source`
const DEFAULT_WRAPPED_CONTEXT_REGEXP: &str = ".*";

pub fn create_context_dependency(
  param: &BasicEvaluatedExpression,
  _expr: &Expr,
) -> Option<ContextModuleScanResult> {
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
        .map(|q| quote_meta(q.string().as_str()))
        .join(""),
      quote_meta(&postfix)
    );
    Some(ContextModuleScanResult {
      context,
      reg,
      query,
      fragment,
    })
    // TODO: `replaces` in context module
    // TODO: `critical` in context module
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
    let prefix_raw = if prefix_is_string {
      Cow::Borrowed(
        param
          .prefix()
          .map(|prefix| prefix.string())
          .expect("must exist"),
      )
    } else {
      Cow::Owned(String::new())
    };
    let postfix_raw = if postfix_is_string {
      Cow::Borrowed(
        param
          .postfix()
          .map(|prefix| prefix.string())
          .expect("must exist"),
      )
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
      "^{}{DEFAULT_WRAPPED_CONTEXT_REGEXP}{}$",
      quote_meta(&prefix),
      quote_meta(&postfix)
    );

    Some(ContextModuleScanResult {
      context,
      reg,
      query,
      fragment,
    })

    // TODO: `replaces` in context module
    // TODO: `critical` in context module
    // TODO: handle `param.wrappedInnerExpressions`
  } else {
    None
  }
}
