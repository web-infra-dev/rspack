use once_cell::sync::Lazy;
use regex::Regex;
use swc_core::ecma::ast::{BinExpr, BinaryOp, CallExpr, Callee, Expr, Lit, MemberProp, Tpl};

#[inline]
fn split_context_from_prefix(prefix: String) -> (String, String) {
  if let Some(idx) = prefix.rfind('/') {
    (prefix[..idx].to_string(), format!(".{}", &prefix[idx..]))
  } else {
    (".".to_string(), prefix)
  }
}

pub fn scanner_context_module(expr: &Expr) -> Option<(String, String)> {
  match expr {
    Expr::Tpl(tpl) if !tpl.exprs.is_empty() => Some(scan_context_module_tpl(tpl)),
    Expr::Bin(bin) => scan_context_module_bin(bin),
    Expr::Call(call) => scan_context_module_concat_call(call),
    _ => None,
  }
}

static META_REG: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"[-\[\]\\/{}()*+?.^$|]").expect("Failed to initialize `MATCH_RESOURCE_REGEX`")
});

#[inline]
fn quote_meta(str: String) -> String {
  META_REG.replace_all(&str, "\\$0").to_string()
}

// require(`./${a}.js`)
fn scan_context_module_tpl(tpl: &Tpl) -> (String, String) {
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
    .map(|s| s.raw.to_string() + ".*")
    .collect::<Vec<String>>()
    .join("");
  let reg = format!(
    "^{prefix}.*{inner_reg}{postfix_raw}$",
    prefix = quote_meta(prefix),
    postfix_raw = quote_meta(postfix_raw)
  );
  (context, reg)
}

// require("./" + a + ".js")
fn scan_context_module_bin(bin: &BinExpr) -> Option<(String, String)> {
  if !is_add_op_bin_expr(bin) {
    return None;
  }
  let prefix_raw = if let Some(prefix) = find_expr_prefix_string(&bin.left) {
    prefix
  } else {
    "".to_string()
  };
  let postfix_raw = if let Some(postfix) = find_expr_prefix_string(&bin.right) {
    postfix
  } else {
    "".to_string()
  };

  if prefix_raw.is_empty() && postfix_raw.is_empty() {
    return None;
  }

  let (context, prefix) = split_context_from_prefix(prefix_raw);
  let reg = format!(
    "^{prefix}.*{postfix_raw}$",
    prefix = quote_meta(prefix),
    postfix_raw = quote_meta(postfix_raw)
  );

  Some((context, reg))
}

fn find_expr_prefix_string(expr: &Expr) -> Option<String> {
  match &expr {
    Expr::Lit(Lit::Str(str)) => Some(str.value.to_string()),
    Expr::Lit(Lit::Num(num)) => Some(num.value.to_string()),
    Expr::Bin(bin) => find_expr_prefix_string(&bin.left),
    _ => None,
  }
}

fn is_add_op_bin_expr(bin: &BinExpr) -> bool {
  if !matches!(&bin.op, BinaryOp::Add) {
    return false;
  }
  match &bin.left {
    box Expr::Bin(bin) => is_add_op_bin_expr(bin),
    _ => true,
  }
}

// require("./".concat(a, ".js"))
// babel/swc will transform template literal to string concat, so we need to handle this case
// see https://github.com/webpack/webpack/pull/5679
fn scan_context_module_concat_call(expr: &CallExpr) -> Option<(String, String)> {
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
  let reg = format!(
    "^{prefix}.*{postfix_raw}$",
    prefix = quote_meta(prefix),
    postfix_raw = quote_meta(postfix_raw)
  );

  Some((context, reg))
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

      if let box Expr::Call(call) = &member_expr.obj {
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
      if let box Expr::Lit(Lit::Str(str)) = &member_expr.obj {
        return Some(str.value.to_string());
      }
      if let box Expr::Lit(Lit::Num(num)) = &member_expr.obj {
        return Some(num.value.to_string());
      }
      if let box Expr::Call(call) = &member_expr.obj {
        return find_concat_expr_prefix_string(call);
      }
      None
    }
    _ => None,
  }
}

fn find_concat_expr_postfix_string(expr: &CallExpr) -> Option<String> {
  expr.args.last().and_then(|arg| {
    if let box Expr::Lit(Lit::Str(str)) = &arg.expr {
      return Some(str.value.to_string());
    }
    if let box Expr::Lit(Lit::Num(num)) = &arg.expr {
      return Some(num.value.to_string());
    }
    None
  })
}
