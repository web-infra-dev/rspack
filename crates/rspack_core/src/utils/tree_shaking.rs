use std::sync::Arc;

use dashmap::DashMap;
use swc_common::{
  comments::{self, Comment},
  BytePos, SyntaxContext, DUMMY_SP,
};
use swc_ecma_visit::{
  noop_visit_mut_type,
  swc_ecma_ast::{EsVersion, Module},
  VisitMut, VisitMutWith,
};

struct Noop {
  trailing: Arc<DashMap<BytePos, Vec<Comment>, ahash::RandomState>>,
}
impl Noop {}
impl VisitMut for Noop {
  noop_visit_mut_type!();

  fn visit_mut_ident(&mut self, n: &mut swc_ecma_visit::swc_ecma_ast::Ident) {
    dbg!(&n.sym);
    let ctxt = n.span.ctxt;
    let hi = n.span.hi;
    if SyntaxContext::empty() != ctxt {
      match self.trailing.entry(hi) {
        dashmap::mapref::entry::Entry::Occupied(mut value) => {
          value.get_mut().insert(
            1,
            Comment {
              kind: comments::CommentKind::Block,
              span: DUMMY_SP,
              text: format!("#{}", ctxt.as_u32()).into(),
            },
          );
        }
        dashmap::mapref::entry::Entry::Vacant(entry) => {
          entry.insert(vec![Comment {
            kind: comments::CommentKind::Block,
            span: DUMMY_SP,
            text: format!("#{}", ctxt.as_u32()).into(),
          }]);
        }
      };
    }
  }
}
