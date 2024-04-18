use std::borrow::Cow;

use once_cell::sync::Lazy;
use regex::Regex;
use rspack_core::{parse_resource, rspack_sources::ReplaceSource, SpanExt};
use rspack_util::json_stringify;
use swc_core::ecma::ast::{
  BinExpr, BinaryOp, CallExpr, Callee, Expr, Lit, MemberProp, TaggedTpl, Tpl,
};

use crate::utils::eval::TemplateStringKind;

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

/// FIXME: remove this function
pub fn scanner_context_module(expr: &Expr) -> Option<ContextModuleScanResult> {
  match expr {
    Expr::Tpl(tpl) if !tpl.exprs.is_empty() => {
      Some(scan_context_module_tpl(tpl, TemplateStringKind::Cooked))
    }
    Expr::Bin(bin) => scan_context_module_bin(bin),
    Expr::Call(call) => scan_context_module_concat_call(call),
    Expr::TaggedTpl(t_tpl) => Some(scan_context_module_tagged_tpl(t_tpl)),
    _ => None,
  }
}

static META_REG: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"[-\[\]\\/{}()*+?.^$|]").expect("Failed to initialize `MATCH_RESOURCE_REGEX`")
});

pub fn quote_meta(str: &str) -> Cow<str> {
  META_REG.replace_all(str, "\\$0")
}

// require(`./${a}.js`)
fn scan_context_module_tpl(tpl: &Tpl, kind: TemplateStringKind) -> ContextModuleScanResult {
  let prefix_raw = tpl
    .quasis
    .first()
    .expect("should have one quasis")
    .raw
    .to_string();
  let postfix_raw = if tpl.quasis.len() > 1 {
    tpl
      .quasis
      .last()
      .expect("should have last quasis")
      .raw
      .to_string()
  } else {
    String::new()
  };
  let (context, prefix) = split_context_from_prefix(prefix_raw);
  let inner_reg = tpl
    .quasis
    .iter()
    .skip(tpl.quasis.len())
    .skip(1)
    .map(|s| {
      match kind {
        TemplateStringKind::Raw => s.raw.as_ref(),
        TemplateStringKind::Cooked => s.cooked.as_ref().unwrap_or(&s.raw),
      }
      .to_string()
        + ".*"
    })
    .collect::<Vec<String>>()
    .join("");

  let (postfix, query, fragment) = match parse_resource(&postfix_raw) {
    Some(data) => (
      data.path.to_string_lossy().to_string(),
      data.query.unwrap_or_default(),
      data.fragment.unwrap_or_default(),
    ),
    None => (postfix_raw, String::new(), String::new()),
  };

  let reg = format!(
    "^{prefix}.*{inner_reg}{postfix_raw}$",
    prefix = quote_meta(&prefix),
    postfix_raw = quote_meta(&postfix)
  );
  ContextModuleScanResult {
    context,
    reg,
    query,
    fragment,
    replaces: Vec::new(),
  }
}

// require("./" + a + ".js")
fn scan_context_module_bin(bin: &BinExpr) -> Option<ContextModuleScanResult> {
  if !is_add_op_bin_expr(bin) {
    return None;
  }
  let (prefix_raw, prefix_range) =
    if let Some((prefix, start, end)) = find_expr_prefix_string(&bin.left) {
      (prefix, Some((start, end)))
    } else {
      ("".to_string(), None)
    };
  let (postfix_raw, postfix_range) =
    if let Some((postfix, start, end)) = find_expr_prefix_string(&bin.right) {
      (postfix, Some((start, end)))
    } else {
      ("".to_string(), None)
    };

  if prefix_raw.is_empty() && postfix_raw.is_empty() {
    return None;
  }

  let (context, prefix) = split_context_from_prefix(prefix_raw);

  let (postfix, query, fragment) = match parse_resource(&postfix_raw) {
    Some(data) => (
      data.path.to_string_lossy().to_string(),
      data.query.unwrap_or_default(),
      data.fragment.unwrap_or_default(),
    ),
    None => (postfix_raw, String::new(), String::new()),
  };

  let reg = format!(
    "^{prefix}.*{postfix_raw}$",
    prefix = quote_meta(&prefix),
    postfix_raw = quote_meta(&postfix)
  );

  let mut replaces = Vec::new();
  if let Some((start, end)) = prefix_range {
    replaces.push((json_stringify(&prefix), start, end));
  }
  if let Some((start, end)) = postfix_range {
    replaces.push((json_stringify(&postfix), start, end));
  }

  Some(ContextModuleScanResult {
    context,
    reg,
    query,
    fragment,
    replaces,
  })
}

fn find_expr_prefix_string(expr: &Expr) -> Option<(String, u32, u32)> {
  match &expr {
    Expr::Lit(Lit::Str(str)) => Some((
      str.value.to_string(),
      str.span.real_lo(),
      str.span.real_hi(),
    )),
    Expr::Lit(Lit::Num(num)) => Some((
      num.value.to_string(),
      num.span.real_lo(),
      num.span.real_hi(),
    )),
    Expr::Bin(bin) => find_expr_prefix_string(&bin.left),
    _ => None,
  }
}

fn is_add_op_bin_expr(bin: &BinExpr) -> bool {
  if !matches!(&bin.op, BinaryOp::Add) {
    return false;
  }
  match bin.left.as_ref() {
    Expr::Bin(bin) => is_add_op_bin_expr(bin),
    _ => true,
  }
}

// require("./".concat(a, ".js"))
// babel/swc will transform template literal to string concat, so we need to handle this case
// see https://github.com/webpack/webpack/pull/5679
fn scan_context_module_concat_call(expr: &CallExpr) -> Option<ContextModuleScanResult> {
  if !is_concat_call(expr) {
    return None;
  }
  let prefix_raw = if let Some(prefix) = find_concat_expr_prefix_string(expr) {
    prefix
  } else {
    "".to_string()
  };
  let postfix_raw = if let Some(postfix) = find_concat_expr_postfix_string(expr) {
    postfix
  } else {
    "".to_string()
  };

  if prefix_raw.is_empty() && postfix_raw.is_empty() {
    return None;
  }

  let (context, prefix) = split_context_from_prefix(prefix_raw);
  let (postfix, query, fragment) = match parse_resource(&postfix_raw) {
    Some(data) => (
      data.path.to_string_lossy().to_string(),
      data.query.unwrap_or_default(),
      data.fragment.unwrap_or_default(),
    ),
    None => (postfix_raw, String::new(), String::new()),
  };
  let reg = format!(
    "^{prefix}.*{postfix_raw}$",
    prefix = quote_meta(&prefix),
    postfix_raw = quote_meta(&postfix)
  );

  Some(ContextModuleScanResult {
    context,
    reg,
    query,
    fragment,
    replaces: Vec::new(),
  })
}

// require(String.raw`./${a}.js`)
fn scan_context_module_tagged_tpl(tpl: &TaggedTpl) -> ContextModuleScanResult {
  match tpl.tag.as_member() {
    Some(tag)
      if tag
        .obj
        .as_ident()
        .map(|ident| ident.sym == *"String")
        .unwrap_or(false)
        && tag
          .prop
          .as_ident()
          .map(|ident| ident.sym == *"raw")
          .unwrap_or(false) =>
    {
      scan_context_module_tpl(tpl.tpl.as_ref(), TemplateStringKind::Raw)
    }
    _ => ContextModuleScanResult {
      context: String::from("."),
      reg: String::new(),
      query: String::new(),
      fragment: String::new(),
      replaces: Vec::new(),
    },
  }
}

fn is_concat_call(expr: &CallExpr) -> bool {
  match &expr.callee {
    Callee::Expr(box Expr::Member(member_expr)) => {
      if let MemberProp::Ident(ident) = &member_expr.prop {
        if ident.sym != *"concat" {
          return false;
        }
      } else {
        return false;
      }

      if let Expr::Call(call) = member_expr.obj.as_ref() {
        return is_concat_call(call);
      }
      true
    }
    _ => false,
  }
}

fn find_concat_expr_prefix_string(expr: &CallExpr) -> Option<String> {
  match &expr.callee {
    Callee::Expr(box Expr::Member(member_expr)) => {
      if let Expr::Lit(Lit::Str(str)) = member_expr.obj.as_ref() {
        return Some(str.value.to_string());
      }
      if let Expr::Lit(Lit::Num(num)) = member_expr.obj.as_ref() {
        return Some(num.value.to_string());
      }
      if let Expr::Call(call) = member_expr.obj.as_ref() {
        return find_concat_expr_prefix_string(call);
      }
      None
    }
    _ => None,
  }
}

fn find_concat_expr_postfix_string(expr: &CallExpr) -> Option<String> {
  expr.args.last().and_then(|arg| {
    if let Expr::Lit(Lit::Str(str)) = arg.expr.as_ref() {
      return Some(str.value.to_string());
    }
    if let Expr::Lit(Lit::Num(num)) = arg.expr.as_ref() {
      return Some(num.value.to_string());
    }
    None
  })
}
