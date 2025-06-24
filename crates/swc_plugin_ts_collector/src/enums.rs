use rustc_hash::{FxHashMap, FxHashSet};
use swc_core::{
  atoms::Atom,
  ecma::{
    ast::{
      Decl, ExportDecl, ExportSpecifier, Expr, Lit, ModuleDecl, ModuleItem, Program, Stmt,
      TsEnumDecl, TsEnumMemberId,
    },
    visit::Visit,
  },
};

#[derive(Debug)]
pub struct ExportedEnumCollector<'a> {
  const_only: bool,
  export_idents: FxHashSet<Atom>,
  collected: &'a mut FxHashMap<Atom, FxHashMap<Atom, EnumMemberValue>>,
}

#[derive(Debug)]
pub enum EnumMemberValue {
  Number(f64),
  String(Atom),
}

impl<'a> ExportedEnumCollector<'a> {
  pub fn new(
    const_only: bool,
    collected: &'a mut FxHashMap<Atom, FxHashMap<Atom, EnumMemberValue>>,
  ) -> Self {
    Self {
      const_only,
      export_idents: Default::default(),
      collected,
    }
  }

  fn collect(&mut self, enum_decl: &TsEnumDecl) {
    if self.const_only && !enum_decl.is_const {
      return;
    }
    // ref: https://github.com/evanw/esbuild/blob/f4159a7b823cd5fe2217da2c30e8873d2f319667/internal/js_parser/js_parser.go#L11263-L11320
    let mut next_numeric_value = 0.0;
    let mut has_numeric_value = true;
    let enum_members: FxHashMap<Atom, EnumMemberValue> = enum_decl
      .members
      .iter()
      .filter_map(|member| {
        if let Some(expr) = &member.init {
          has_numeric_value = false;
          if let Expr::Lit(literal) = &**expr {
            // only string and number is allowed
            match literal {
              Lit::Num(n) => {
                has_numeric_value = true;
                next_numeric_value = n.value + 1.0;
                return Some((
                  enum_member_id_atom(&member.id),
                  EnumMemberValue::Number(n.value),
                ));
              }
              Lit::Str(s) => {
                return Some((
                  enum_member_id_atom(&member.id),
                  EnumMemberValue::String(s.value.clone()),
                ));
              }
              _ => return None,
            }
          } else {
            // TODO: try eval simple expr here
            return None;
          }
        } else if has_numeric_value {
          let value = next_numeric_value;
          next_numeric_value += 1.0;
          return Some((
            enum_member_id_atom(&member.id),
            EnumMemberValue::Number(value),
          ));
        } else {
          // value is undefined here, usually TypeScript isolatedModules will emit an error
          // if value is undefined: https://github.com/evanw/esbuild/issues/3868
          // we don't optimize this enum member, so do nothing here
          return None;
        }
      })
      .collect();
    self
      .collected
      .insert(enum_decl.id.sym.clone(), enum_members);
  }
}

fn enum_member_id_atom(member_id: &TsEnumMemberId) -> Atom {
  match member_id {
    TsEnumMemberId::Ident(ident) => ident.sym.clone(),
    TsEnumMemberId::Str(str) => str.value.clone(),
  }
}

impl Visit for ExportedEnumCollector<'_> {
  fn visit_program(&mut self, node: &Program) {
    let Program::Module(node) = node else {
      return;
    };
    for item in &node.body {
      match item {
        ModuleItem::ModuleDecl(decl) => match decl {
          ModuleDecl::ExportDecl(ExportDecl {
            decl: Decl::TsEnum(enum_decl),
            ..
          }) => {
            self.export_idents.insert(enum_decl.id.sym.clone());
            self.collect(&enum_decl);
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
                  self.export_idents.insert(specifier.orig.atom().clone());
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
        },
        ModuleItem::Stmt(stmt) => {
          if let Stmt::Decl(Decl::TsEnum(enum_decl)) = stmt {
            self.collect(&enum_decl);
          }
        }
      }
    }

    self
      .collected
      .retain(|enum_name, _| self.export_idents.contains(&enum_name));
  }
}
