use rspack_util::atom::AtomKey;
use rustc_hash::FxHashSet;
use swc_next_ecma_ast::{
  Ast, AstNode, BindingIdentifier, IdentifierReference, NodeId, NodeKind, Visit, VisitWith,
  WithStatement,
};
use swc_next_ecma_semantic::name_resolver::{JsNameResolver, SymbolNode, resolver};

use crate::Atom;

/// The subset of JavaScript name-resolution metadata consumed by Rspack.
///
/// SWC Next's full `Semantic` also stores symbols, declarations, references,
/// scopes, and checker state. Dependency scanning only needs resolved scopes,
/// top-level binding names, duplicate top-level names, and the ability to
/// distinguish definite globals from references intercepted by `with`.
pub struct JavascriptNameResolution<'ast> {
  resolver: JsNameResolver<'ast>,
  top_level_bindings: FxHashSet<AtomKey>,
  duplicate_top_level_bindings: FxHashSet<AtomKey>,
  dynamic_unresolved_references: FxHashSet<NodeId>,
}

impl<'ast> JavascriptNameResolution<'ast> {
  pub fn resolve(ast: &'ast Ast<'ast>) -> Self {
    let resolver = resolver(ast);
    let top_level_scope = resolver.top_level_scope_id();
    let mut top_level_bindings = FxHashSet::default();
    let mut duplicate_top_level_bindings = FxHashSet::default();
    let mut has_with_statement = false;

    for index in 0..ast.node_count() {
      let node = NodeId::from_raw_unchecked(index as u32);
      match ast.node_kind(node) {
        NodeKind::BindingIdentifier => {
          // SAFETY: The node kind was checked above.
          let identifier = unsafe { BindingIdentifier::from_node_id_unchecked(node) };
          if resolver.symbol_scope(SymbolNode::BindingIdentifier(identifier)) == top_level_scope {
            let name = AtomKey::from(Atom::from_ast(ast, identifier.name(ast)));
            if !top_level_bindings.insert(name.clone()) {
              duplicate_top_level_bindings.insert(name);
            }
          }
        }
        NodeKind::WithStatement => has_with_statement = true,
        _ => {}
      }
    }

    let dynamic_unresolved_references = if has_with_statement {
      let mut collector = DynamicUnresolvedReferenceCollector {
        ast,
        resolver: &resolver,
        with_depth: 0,
        references: FxHashSet::default(),
      };
      ast.root_program().visit_with(&mut collector);
      collector.references
    } else {
      FxHashSet::default()
    };

    Self {
      resolver,
      top_level_bindings,
      duplicate_top_level_bindings,
      dynamic_unresolved_references,
    }
  }

  pub fn top_level_bindings(&self) -> impl Iterator<Item = &AtomKey> {
    self.top_level_bindings.iter()
  }

  pub fn duplicate_top_level_bindings(&self) -> FxHashSet<AtomKey> {
    self.duplicate_top_level_bindings.clone()
  }

  #[inline]
  pub fn is_definitely_global(&self, identifier: IdentifierReference) -> bool {
    self
      .resolver
      .symbol_scope(SymbolNode::IdentifierReference(identifier))
      == self.resolver.unresolved_scope_id()
      && !self
        .dynamic_unresolved_references
        .contains(&identifier.node_id())
  }
}

struct DynamicUnresolvedReferenceCollector<'resolver, 'ast> {
  ast: &'ast Ast<'ast>,
  resolver: &'resolver JsNameResolver<'ast>,
  with_depth: usize,
  references: FxHashSet<NodeId>,
}

impl<'ast> Visit<'ast> for DynamicUnresolvedReferenceCollector<'_, 'ast> {
  fn ast(&self) -> &Ast<'ast> {
    self.ast
  }

  fn visit_with_statement(&mut self, statement: WithStatement) {
    // A `with` object's expression is evaluated before its object environment
    // exists. Only references in the body can be intercepted dynamically.
    statement.object(self.ast).visit_with(self);
    self.with_depth += 1;
    statement.body(self.ast).visit_with(self);
    self.with_depth -= 1;
  }

  fn visit_identifier_reference(&mut self, identifier: IdentifierReference) {
    if self.with_depth > 0
      && self
        .resolver
        .symbol_scope(SymbolNode::IdentifierReference(identifier))
        == self.resolver.unresolved_scope_id()
    {
      self.references.insert(identifier.node_id());
    }
  }
}
