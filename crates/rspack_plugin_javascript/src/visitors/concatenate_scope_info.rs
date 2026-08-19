use rspack_core::{
  AnalyzedConcatenationScopeInfo, ConcatenationScopeCanonicalName, ConcatenationScopeIdent,
  ConcatenationScopeIdentKind, DependencyRange, PendingConcatenationScopeInfo,
};
use rspack_util::atom::Atom;
use rustc_hash::FxHashSet;
use smallvec::SmallVec;
use swc_experimental_allocator::atom::Atom as AstAtom;
use swc_experimental_ecma_ast::{ClassExpr, Ident, ObjectPatProp, Prop, Visit, VisitWith};
use swc_experimental_ecma_semantic::resolver::Semantic;

#[derive(Clone, Copy)]
struct PendingConcatenationScopeIdent {
  range: DependencyRange,
  shorthand: bool,
}

enum UsedNames<'ast> {
  Inline(SmallVec<[AstAtom<'ast>; 8]>),
  Heap(FxHashSet<AstAtom<'ast>>),
}

impl Default for UsedNames<'_> {
  fn default() -> Self {
    Self::Inline(SmallVec::new())
  }
}

impl<'ast> UsedNames<'ast> {
  fn insert(&mut self, name: AstAtom<'ast>) -> bool {
    match self {
      Self::Inline(names) => {
        if names.contains(&name) {
          return false;
        }
        if names.len() < names.inline_size() {
          names.push(name);
          return true;
        }

        let mut names_set =
          FxHashSet::with_capacity_and_hasher(names.len() * 2, Default::default());
        names_set.extend(names.drain(..));
        names_set.insert(name);
        *self = Self::Heap(names_set);
        true
      }
      Self::Heap(names) => names.insert(name),
    }
  }
}

pub(crate) struct PendingConcatenationScopeInfoVisitor<'semantic, 'ast> {
  semantic: &'semantic Semantic,
  top_level_idents: SmallVec<[PendingConcatenationScopeIdent; 8]>,
  global_idents: SmallVec<[PendingConcatenationScopeIdent; 4]>,
  used_name_set: UsedNames<'ast>,
  used_names: SmallVec<[(AstAtom<'ast>, DependencyRange); 8]>,
  canonical_names: SmallVec<[ConcatenationScopeCanonicalName; 1]>,
}

impl<'semantic, 'ast> PendingConcatenationScopeInfoVisitor<'semantic, 'ast> {
  pub(crate) fn new(semantic: &'semantic Semantic) -> Self {
    Self {
      semantic,
      top_level_idents: SmallVec::new(),
      global_idents: SmallVec::new(),
      used_name_set: UsedNames::default(),
      used_names: SmallVec::new(),
      canonical_names: SmallVec::new(),
    }
  }

  #[inline]
  fn add_canonical_name(&mut self, ident: &Ident<'ast>, range: DependencyRange) {
    // An escape-spelled identifier is longer than its parser-canonical name.
    // Persist the canonical form only for this uncommon case, keeping the
    // common per-identifier metadata compact.
    if (range.end - range.start) as usize != ident.sym.as_str().len() {
      self.canonical_names.push(ConcatenationScopeCanonicalName {
        range,
        name: Atom::from(ident.sym.as_str()),
      });
    }
  }

  #[inline]
  fn add_ident(&mut self, ident: &Ident<'ast>, shorthand: bool, class_expr_with_ident: bool) {
    if ident.symbol_id.get().is_none() {
      return;
    }
    let scope = self.semantic.node_scope(ident);
    let range = ident.span.into();
    let pending_ident = PendingConcatenationScopeIdent { range, shorthand };
    if scope == self.semantic.unresolved_scope_id() {
      self.add_canonical_name(ident, range);
      self.global_idents.push(pending_ident);
    } else if class_expr_with_ident || scope != self.semantic.top_level_scope_id() {
      if self.used_name_set.insert(ident.sym) {
        self.add_canonical_name(ident, range);
        self.used_names.push((ident.sym, range));
      }
    } else {
      self.add_canonical_name(ident, range);
      self.top_level_idents.push(pending_ident);
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
    idents.extend(
      self
        .used_names
        .into_iter()
        .map(|(_, range)| ConcatenationScopeIdent {
          range,
          shorthand: false,
          kind: ConcatenationScopeIdentKind::UsedName,
        }),
    );
    PendingConcatenationScopeInfo::Analyzed(AnalyzedConcatenationScopeInfo {
      module_ctxt: self.semantic.top_level_scope_id().raw(),
      global_ctxt: self.semantic.unresolved_scope_id().raw(),
      idents,
      canonical_names: self.canonical_names.into_vec(),
    })
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
