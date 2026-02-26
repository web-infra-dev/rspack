use rspack_core::ImportAttributes;
use swc_experimental_ecma_ast::{Ast, Bool, Expr, Lit, ObjectLit, Regex, Str};

pub fn get_value_by_obj_prop(ast: &Ast, obj: ObjectLit, field: &str) -> Option<Expr> {
  obj.props(ast).iter().find_map(|p| {
    let p = ast.get_node_in_sub_range(p);
    p.as_prop()
      .and_then(|p| p.as_key_value())
      .filter(|kv| {
        kv.key(ast)
          .as_ident()
          .filter(|key| ast.get_utf8(key.sym(ast)) == field)
          .is_some()
          || kv
            .key(ast)
            .as_str()
            .filter(|key| ast.get_wtf8(key.value(ast)) == field)
            .is_some()
      })
      .map(|name| name.value(ast))
  })
}

pub fn get_literal_str_by_obj_prop(ast: &Ast, obj: ObjectLit, field: &str) -> Option<Str> {
  let lit = get_value_by_obj_prop(ast, obj, field).and_then(|e| e.as_lit())?;
  match lit {
    Lit::Str(str) => Some(str),
    _ => None,
  }
}

pub fn get_bool_by_obj_prop(ast: &Ast, obj: ObjectLit, field: &str) -> Option<Bool> {
  let lit = get_value_by_obj_prop(ast, obj, field).and_then(|e| e.as_lit())?;
  match lit {
    Lit::Bool(bool) => Some(bool),
    _ => None,
  }
}

pub fn get_regex_by_obj_prop(ast: &Ast, obj: ObjectLit, field: &str) -> Option<Regex> {
  let lit = get_value_by_obj_prop(ast, obj, field).and_then(|e| e.as_lit())?;
  match lit {
    Lit::Regex(regexp) => Some(regexp),
    _ => None,
  }
}

pub fn get_attributes(ast: &Ast, obj: ObjectLit) -> ImportAttributes {
  obj
    .props(ast)
    .iter()
    .filter_map(|p| {
      let p = ast.get_node_in_sub_range(p);
      p.as_prop().and_then(|p| p.as_key_value()).and_then(|kv| {
        kv.key(ast)
          .as_ident()
          .map(|k| ast.get_utf8(k.sym(ast)))
          .or_else(|| {
            kv.key(ast)
              .as_str()
              .and_then(|k| ast.get_wtf8(k.value(ast)).as_str())
          })
          .map(|s| s.to_string())
          .zip(kv.value(ast).as_lit().and_then(|lit| match lit {
            Lit::Str(s) => Some(ast.get_wtf8(s.value(ast)).to_string_lossy().to_string()),
            _ => None,
          }))
      })
    })
    .collect()
}
