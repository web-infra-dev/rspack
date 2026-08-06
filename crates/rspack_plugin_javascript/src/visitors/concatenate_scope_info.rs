use rspack_core::{
  ConcatenationScopeIdent, ConcatenationScopeIdentKind, DependencyRange,
  PendingConcatenationScopeInfo,
};
use smallvec::SmallVec;
use swc_experimental_allocator::atom::Atom as AstAtom;
use swc_experimental_ecma_ast::{ClassExpr, Ident, ObjectPatProp, Prop, Visit, VisitWith};
use swc_experimental_ecma_semantic::resolver::Semantic;

#[derive(Clone, Copy)]
struct PendingConcatenationScopeIdent {
  range: DependencyRange,
  shorthand: bool,
}

pub(crate) struct PendingConcatenationScopeInfoVisitor<'semantic, 'ast> {
  semantic: &'semantic Semantic,
  top_level_idents: SmallVec<[PendingConcatenationScopeIdent; 8]>,
  global_idents: SmallVec<[PendingConcatenationScopeIdent; 4]>,
  used_names: SmallVec<[(AstAtom<'ast>, DependencyRange); 8]>,
}

impl<'semantic, 'ast> PendingConcatenationScopeInfoVisitor<'semantic, 'ast> {
  pub(crate) fn new(semantic: &'semantic Semantic) -> Self {
    Self {
      semantic,
      top_level_idents: SmallVec::new(),
      global_idents: SmallVec::new(),
      used_names: SmallVec::new(),
    }
  }

  #[inline]
  fn add_ident(&mut self, ident: &Ident<'ast>, shorthand: bool, class_expr_with_ident: bool) {
    if ident.symbol_id.get().is_none() {
      return;
    }
    let scope = self.semantic.node_scope(ident);
    let range = ident.span.into();
    if scope == self.semantic.unresolved_scope_id() {
      self
        .global_idents
        .push(PendingConcatenationScopeIdent { range, shorthand });
    } else if class_expr_with_ident || scope != self.semantic.top_level_scope_id() {
      self.used_names.push((ident.sym, range));
    } else {
      self
        .top_level_idents
        .push(PendingConcatenationScopeIdent { range, shorthand });
    }
  }

  pub(crate) fn into_info(self) -> PendingConcatenationScopeInfo {
    let mut idents = Vec::with_capacity(
      self.top_level_idents.len() + self.global_idents.len() + self.used_names.len(),
    );
    idents.extend(
      self
        .top_level_idents
        .into_iter()
        .map(|ident| ConcatenationScopeIdent {
          range: ident.range,
          shorthand: ident.shorthand,
          kind: ConcatenationScopeIdentKind::TopLevel,
        }),
    );
    idents.extend(
      self
        .global_idents
        .into_iter()
        .map(|ident| ConcatenationScopeIdent {
          range: ident.range,
          shorthand: ident.shorthand,
          kind: ConcatenationScopeIdentKind::Global,
        }),
    );
    let mut used_names = SmallVec::<[AstAtom<'ast>; 8]>::new();
    idents.extend(self.used_names.into_iter().filter_map(|(symbol, range)| {
      if used_names.contains(&symbol) {
        return None;
      }
      used_names.push(symbol);
      Some(ConcatenationScopeIdent {
        range,
        shorthand: false,
        kind: ConcatenationScopeIdentKind::UsedName,
      })
    }));
    PendingConcatenationScopeInfo {
      module_ctxt: self.semantic.top_level_scope_id().raw(),
      global_ctxt: self.semantic.unresolved_scope_id().raw(),
      idents,
    }
  }
}

impl<'ast> Visit<'ast> for PendingConcatenationScopeInfoVisitor<'_, 'ast> {
  fn visit_ident(&mut self, ident: &Ident<'ast>) {
    self.add_ident(ident, false, false);
  }

  fn visit_object_pat_prop(&mut self, prop: &ObjectPatProp<'ast>) {
    match prop {
      ObjectPatProp::Assign(assign) => {
        self.add_ident(&assign.key.id, true, false);
        assign.value.visit_with(self);
      }
      ObjectPatProp::KeyValue(_) | ObjectPatProp::Rest(_) => prop.visit_children_with(self),
    }
  }

  fn visit_prop(&mut self, prop: &Prop<'ast>) {
    if let Prop::Shorthand(ident) = prop {
      self.add_ident(ident, true, false);
    } else {
      prop.visit_children_with(self);
    }
  }

  fn visit_class_expr(&mut self, class_expr: &ClassExpr<'ast>) {
    if let Some(ident) = &class_expr.ident
      && class_expr.class.super_class.is_some()
    {
      self.add_ident(ident, false, true);
      class_expr.class.visit_with(self);
    } else {
      class_expr.visit_children_with(self);
    }
  }
}
