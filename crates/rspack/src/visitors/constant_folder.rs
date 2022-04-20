use swc_ecma_ast::{EmptyStmt, Stmt};
use swc_ecma_visit::Fold;

pub struct ConstantFolder {}

impl Fold for ConstantFolder {
    fn fold_stmt(&mut self, n: swc_ecma_ast::Stmt) -> swc_ecma_ast::Stmt {
        if let Stmt::If(_if_stmt) = n {
            // if_stmt.test.as_ref() {

            // };

            Stmt::Empty(EmptyStmt {
                span: Default::default(),
            })
        } else {
            swc_ecma_visit::fold_stmt(self, n)
        }
        // self
    }
}
