use rustc_hash::FxHashMap;
use swc_core::{
  atoms::Atom,
  ecma::{
    ast::{
      Decl, ExportDecl, Expr, Lit, ModuleDecl, ModuleItem, Program, Stmt, TsEnumDecl,
      TsEnumMemberId,
    },
    visit::Visit,
  },
};

#[derive(Debug)]
pub struct TopLevelEnumCollector<'a> {
  const_only: bool,
  collected: &'a mut FxHashMap<(Atom, Atom), EnumMemberValue>,
}

#[derive(Debug)]
pub enum EnumMemberValue {
  Number(f64),
  String(Atom),
}

impl<'a> TopLevelEnumCollector<'a> {
  pub fn new(
    const_only: bool,
    collected: &'a mut FxHashMap<(Atom, Atom), EnumMemberValue>,
  ) -> Self {
    Self {
      const_only,
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
    for member in &enum_decl.members {
      if let Some(expr) = &member.init {
        has_numeric_value = false;
        if let Expr::Lit(literal) = &**expr {
          // only string and number is allowed
          match literal {
            Lit::Num(n) => {
              self.collected.insert(
                (enum_decl.id.sym.clone(), enum_member_id_atom(&member.id)),
                EnumMemberValue::Number(n.value),
              );
              has_numeric_value = true;
              next_numeric_value = n.value + 1.0;
            }
            Lit::Str(s) => {
              self.collected.insert(
                (enum_decl.id.sym.clone(), enum_member_id_atom(&member.id)),
                EnumMemberValue::String(s.value.clone()),
              );
            }
            _ => continue,
          }
        } else {
          // TODO: try eval simple expr here
        }
      } else if has_numeric_value {
        self.collected.insert(
          (enum_decl.id.sym.clone(), enum_member_id_atom(&member.id)),
          EnumMemberValue::Number(next_numeric_value),
        );
        next_numeric_value += 1.0;
      } else {
        // value is undefined here, usually TypeScript isolatedModules will emit an error
        // if value is undefined: https://github.com/evanw/esbuild/issues/3868
        // we don't optimize this enum member, so do nothing here
        continue;
      }
    }
  }
}

fn enum_member_id_atom(member_id: &TsEnumMemberId) -> Atom {
  match member_id {
    TsEnumMemberId::Ident(ident) => ident.sym.clone(),
    TsEnumMemberId::Str(str) => str.value.clone(),
  }
}

impl Visit for TopLevelEnumCollector<'_> {
  fn visit_program(&mut self, node: &Program) {
    let Program::Module(node) = node else {
      return;
    };
    for item in &node.body {
      match item {
        ModuleItem::ModuleDecl(decl) => {
          if let ModuleDecl::ExportDecl(ExportDecl {
            decl: Decl::TsEnum(enum_decl),
            ..
          }) = decl
          {
            self.collect(&enum_decl);
          }
        }
        ModuleItem::Stmt(stmt) => {
          if let Stmt::Decl(Decl::TsEnum(enum_decl)) = stmt {
            self.collect(&enum_decl);
          }
        }
      }
    }
  }
}
