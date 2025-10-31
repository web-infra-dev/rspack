use std::borrow::Borrow;

use rustc_hash::{FxHashMap, FxHashSet};
use swc_core::{
  atoms::{Atom, Wtf8Atom, wtf8::Wtf8Buf},
  common::SyntaxContext,
  ecma::{
    ast::{
      BinExpr, ComputedPropName, Decl, ExportDecl, ExportSpecifier, Expr, Ident, Lit, MemberExpr,
      MemberProp, ModuleDecl, Program, Stmt, Tpl, TsEnumDecl, TsEnumMemberId, UnaryExpr, op,
    },
    utils::number::ToJsString,
    visit::Visit,
  },
};

type EnumKeyValueMap = FxHashMap<Wtf8Atom, EnumMemberValue>;

#[derive(Debug)]
pub struct ExportedEnumCollector<'a> {
  const_only: bool,
  export_idents: FxHashSet<Atom>,
  unresolved_ctxt: SyntaxContext,
  collected: &'a mut FxHashMap<Atom, EnumKeyValueMap>,
}

#[derive(Debug, Clone)]
pub enum EnumMemberValue {
  Number(f64),
  String(Wtf8Atom),
  Unknown,
}

impl<'a> ExportedEnumCollector<'a> {
  pub fn new(
    const_only: bool,
    unresolved_ctxt: SyntaxContext,
    collected: &'a mut FxHashMap<Atom, EnumKeyValueMap>,
  ) -> Self {
    Self {
      const_only,
      export_idents: Default::default(),
      unresolved_ctxt,
      collected,
    }
  }

  fn collect(&mut self, enum_decl: &TsEnumDecl) {
    if self.const_only && !enum_decl.is_const {
      return;
    }
    let enum_id = &enum_decl.id.sym;
    // remove existing enum members for enum merging
    let mut enum_members = self.collected.remove(enum_id).unwrap_or_default();
    // ref: https://github.com/evanw/esbuild/blob/f4159a7b823cd5fe2217da2c30e8873d2f319667/internal/js_parser/js_parser.go#L11263-L11320
    let mut next_numeric_value = 0.0;
    let mut has_numeric_value = true;
    for member in &enum_decl.members {
      let member_key = enum_member_id_atom(&member.id);
      let member_value = if let Some(expr) = &member.init {
        has_numeric_value = false;
        match self.evaluate_expr(expr, enum_id, &enum_members) {
          evaluated @ EnumMemberValue::Number(n) => {
            has_numeric_value = true;
            next_numeric_value = n + 1.0;
            evaluated
          }
          evaluated => evaluated,
        }
      } else if has_numeric_value {
        let value = next_numeric_value;
        next_numeric_value += 1.0;
        EnumMemberValue::Number(value)
      } else {
        // enum member value is undefined here, usually TypeScript isolatedModules will emit
        // an error if value is undefined: https://github.com/evanw/esbuild/issues/3868
        // we don't optimize this kind of enum member, so keep its origin content here
        EnumMemberValue::Unknown
      };
      enum_members.insert(member_key, member_value);
    }
    self.collected.insert(enum_id.clone(), enum_members);
  }

  // modified from https://github.com/swc-project/swc/blob/f328e4a560f7564d1c10b58bcb7d684ff6a7a3b1/crates/swc_ecma_transforms_typescript/src/ts_enum.rs#L105
  fn evaluate_expr(
    &self,
    expr: &Expr,
    enum_id: &Atom,
    existing_enum_members: &EnumKeyValueMap,
  ) -> EnumMemberValue {
    match expr {
      Expr::Lit(Lit::Str(s)) => EnumMemberValue::String(s.value.clone()),
      Expr::Lit(Lit::Num(n)) => EnumMemberValue::Number(n.value),
      Expr::Ident(Ident { ctxt, sym, .. }) if sym == "NaN" && *ctxt == self.unresolved_ctxt => {
        EnumMemberValue::Number(f64::NAN)
      }
      Expr::Ident(Ident { ctxt, sym, .. })
        if sym == "Infinity" && *ctxt == self.unresolved_ctxt =>
      {
        EnumMemberValue::Number(f64::INFINITY)
      }
      Expr::Ident(ident) => existing_enum_members
        .get(ident.sym.borrow())
        .map(|value| match value {
          EnumMemberValue::String(s) => EnumMemberValue::String(s.clone()),
          EnumMemberValue::Number(n) => EnumMemberValue::Number(*n),
          _ => EnumMemberValue::Unknown,
        })
        .unwrap_or_else(|| EnumMemberValue::Unknown),
      Expr::Paren(e) => self.evaluate_expr(&e.expr, enum_id, existing_enum_members),
      Expr::Unary(e) => self.evaluate_unary(e, enum_id, existing_enum_members),
      Expr::Bin(e) => self.evaluate_bin(e, enum_id, existing_enum_members),
      Expr::Member(e) => self.evaluate_member(e, enum_id, existing_enum_members),
      Expr::Tpl(e) => self.evaluate_tpl(e, enum_id, existing_enum_members),
      _ => EnumMemberValue::Unknown,
    }
  }

  fn evaluate_unary(
    &self,
    expr: &UnaryExpr,
    enum_id: &Atom,
    existing_enum_members: &EnumKeyValueMap,
  ) -> EnumMemberValue {
    if !matches!(expr.op, op!(unary, "+") | op!(unary, "-") | op!("~")) {
      return EnumMemberValue::Unknown;
    }

    let inner = self.evaluate_expr(&expr.arg, enum_id, existing_enum_members);

    let EnumMemberValue::Number(num) = inner else {
      return EnumMemberValue::Unknown;
    };

    match expr.op {
      op!(unary, "+") => EnumMemberValue::Number(num),
      op!(unary, "-") => EnumMemberValue::Number(-num),
      op!("~") => EnumMemberValue::Number(!js_number_to_int32(num) as f64),
      _ => unreachable!(),
    }
  }

  fn evaluate_bin(
    &self,
    expr: &BinExpr,
    enum_id: &Atom,
    existing_enum_members: &EnumKeyValueMap,
  ) -> EnumMemberValue {
    if !matches!(
      expr.op,
      op!(bin, "+")
        | op!(bin, "-")
        | op!("*")
        | op!("/")
        | op!("%")
        | op!("**")
        | op!("<<")
        | op!(">>")
        | op!(">>>")
        | op!("|")
        | op!("&")
        | op!("^"),
    ) {
      return EnumMemberValue::Unknown;
    }

    let left = self.evaluate_expr(&expr.left, enum_id, existing_enum_members);
    let right = self.evaluate_expr(&expr.right, enum_id, existing_enum_members);

    match (left, right, expr.op) {
      (EnumMemberValue::Number(left), EnumMemberValue::Number(right), op) => {
        let value = match op {
          op!(bin, "+") => left + right,
          op!(bin, "-") => left - right,
          op!("*") => left * right,
          op!("/") => left / right,
          op!("%") => left % right,
          op!("**") => {
            if right.is_nan() || (left.abs() == 1f64 && right.is_infinite()) {
              f64::NAN
            } else {
              left.powf(right)
            }
          }
          op!("<<") => js_number_to_int32(left).wrapping_shl(js_number_to_uint32(right)) as f64,
          op!(">>") => js_number_to_int32(left).wrapping_shr(js_number_to_uint32(right)) as f64,
          op!(">>>") => js_number_to_uint32(left).wrapping_shr(js_number_to_uint32(right)) as f64,
          op!("|") => (js_number_to_int32(left) | js_number_to_int32(right)) as f64,
          op!("&") => (js_number_to_int32(left) & js_number_to_int32(right)) as f64,
          op!("^") => (js_number_to_int32(left) ^ js_number_to_int32(right)) as f64,
          _ => unreachable!(),
        };

        EnumMemberValue::Number(value)
      }
      (EnumMemberValue::String(left), EnumMemberValue::String(right), op!(bin, "+")) => {
        let mut res = Wtf8Buf::new();
        res.push_wtf8(left.as_wtf8());
        res.push_wtf8(right.as_wtf8());
        EnumMemberValue::String(Wtf8Atom::new(res))
      }
      (EnumMemberValue::Number(left), EnumMemberValue::String(right), op!(bin, "+")) => {
        let mut res = Wtf8Buf::new();
        res.push_str(left.to_js_string().as_str());
        res.push_wtf8(right.as_wtf8());
        EnumMemberValue::String(Wtf8Atom::new(res))
      }
      (EnumMemberValue::String(left), EnumMemberValue::Number(right), op!(bin, "+")) => {
        let mut res = Wtf8Buf::new();
        res.push_wtf8(left.as_wtf8());
        res.push_str(right.to_js_string().as_str());
        EnumMemberValue::String(Wtf8Atom::new(res))
      }
      _ => EnumMemberValue::Unknown,
    }
  }

  fn evaluate_member(
    &self,
    expr: &MemberExpr,
    enum_id: &Atom,
    existing_enum_members: &EnumKeyValueMap,
  ) -> EnumMemberValue {
    if matches!(expr.prop, MemberProp::PrivateName(..)) {
      return EnumMemberValue::Unknown;
    }
    let member_name = match &expr.prop {
      MemberProp::Ident(ident) => ident.sym.borrow(),
      MemberProp::Computed(ComputedPropName { expr, .. }) => {
        let Expr::Lit(Lit::Str(s)) = &**expr else {
          return EnumMemberValue::Unknown;
        };

        &s.value
      }
      _ => return EnumMemberValue::Unknown,
    };
    let Expr::Ident(ident) = &*expr.obj else {
      return EnumMemberValue::Unknown;
    };
    // Only support referencing properties inside the same enum decl for now
    if &ident.sym != enum_id {
      return EnumMemberValue::Unknown;
    }
    if let Some(value) = existing_enum_members.get(member_name) {
      return value.clone();
    }
    EnumMemberValue::Unknown
  }

  fn evaluate_tpl(
    &self,
    expr: &Tpl,
    enum_id: &Atom,
    existing_enum_members: &EnumKeyValueMap,
  ) -> EnumMemberValue {
    let Tpl { exprs, quasis, .. } = expr;

    let mut quasis_iter = quasis.iter();

    let Some(mut string) = quasis_iter.next().map(|q| q.raw.to_string()) else {
      return EnumMemberValue::Unknown;
    };

    for (q, expr) in quasis_iter.zip(exprs) {
      let expr = self.evaluate_expr(expr, enum_id, existing_enum_members);

      let expr = match expr {
        EnumMemberValue::String(s) => s.to_string_lossy().into(),
        EnumMemberValue::Number(n) => n.to_js_string(),
        _ => return EnumMemberValue::Unknown,
      };

      string.push_str(&expr);
      string.push_str(&q.raw);
    }

    EnumMemberValue::String(string.into())
  }
}

fn enum_member_id_atom(member_id: &TsEnumMemberId) -> Wtf8Atom {
  match member_id {
    TsEnumMemberId::Ident(ident) => Wtf8Atom::new(ident.sym.as_str()),
    TsEnumMemberId::Str(str) => str.value.clone(),
  }
}

fn js_number_to_uint32(n: f64) -> u32 {
  if !n.is_finite() {
    return 0;
  }

  // pow(2, 32) = 4294967296
  n.trunc().rem_euclid(4294967296.0) as u32
}

fn js_number_to_int32(n: f64) -> i32 {
  js_number_to_uint32(n) as i32
}

impl Visit for ExportedEnumCollector<'_> {
  fn visit_program(&mut self, node: &Program) {
    let Program::Module(node) = node else {
      return;
    };
    for decl in node.body.iter().filter_map(|item| item.as_module_decl()) {
      match decl {
        ModuleDecl::ExportDecl(ExportDecl {
          decl: Decl::TsEnum(enum_decl),
          ..
        }) => {
          self.collect(enum_decl);
        }
        ModuleDecl::ExportNamed(named_export) => {
          if named_export.src.is_some() {
            return;
          }
          for specifier in &named_export.specifiers {
            match specifier {
              ExportSpecifier::Named(specifier) => {
                if specifier.is_type_only {
                  continue;
                }
                self
                  .export_idents
                  .insert(specifier.orig.atom().into_owned());
              }
              _ => continue,
            }
          }
        }
        ModuleDecl::ExportDefaultExpr(expr) => {
          if let Some(ident) = expr.expr.unwrap_parens().as_ident() {
            self.export_idents.insert(ident.sym.clone());
          }
        }
        _ => {}
      }
    }
    for stmt in node.body.iter().filter_map(|item| item.as_stmt()) {
      if let Stmt::Decl(Decl::TsEnum(enum_decl)) = stmt
        && self.export_idents.contains(&enum_decl.id.sym)
      {
        self.collect(enum_decl);
      }
    }
  }
}
