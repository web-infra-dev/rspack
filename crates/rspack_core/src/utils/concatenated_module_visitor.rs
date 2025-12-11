use swc_core::{atoms::Atom, ecma::ast::Ident as LegacyIdent};
use swc_experimental_ecma_ast::{Ast, ClassExpr, Ident, NodeIdTrait, NodeKind};
use swc_experimental_ecma_semantic::resolver::Semantic;

#[derive(Clone, Debug)]
pub struct ConcatenatedModuleIdent {
  pub id: LegacyIdent,
  pub shorthand: bool,
  pub is_class_expr_with_ident: bool,
}

pub fn collect_ident(ast: &Ast, semantic: &Semantic) -> Vec<ConcatenatedModuleIdent> {
  let mut ids = Vec::new();
  for (node_id, node) in ast.nodes() {
    if node.kind() == NodeKind::Ident {
      let ident = Ident::from_node_id(node_id, ast);
      let parent_id = semantic.parent_node(node_id);
      let (shorthand, is_class_expr_with_ident) = match ast.get_node(parent_id).kind() {
        NodeKind::BindingIdent => {
          let parent_id = semantic.parent_node(parent_id);
          (
            ast.get_node(parent_id).kind() == NodeKind::AssignPatProp,
            false,
          )
        }
        // https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/optimize/ConcatenatedModule.js#L1173-L1197
        NodeKind::ClassExpr => {
          let class_expr = ClassExpr::from_node_id(parent_id, ast);
          (false, class_expr.class(ast).super_class(ast).is_some())
        }
        NodeKind::ObjectLit => (true, false),
        _ => (false, false),
      };

      let span = ident.span(ast);
      let sym = Atom::new(ast.get_utf8(ident.sym(ast)));
      let ctxt = semantic.node_scope(ident).to_ctxt();
      ids.push(ConcatenatedModuleIdent {
        id: LegacyIdent::new(sym, span, ctxt),
        shorthand,
        is_class_expr_with_ident,
      });
    }
  }
  ids
}
