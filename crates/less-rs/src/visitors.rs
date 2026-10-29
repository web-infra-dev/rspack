//! The post-evaluation passes: ports of `JoinSelectorVisitor`,
//! `SetTreeVisibilityVisitor(true)`, `ExtendVisitor` and `ToCSSVisitor`.
//!
//! They operate on the evaluated tree in the same order and with the same
//! mutation points as less.js.

use cow_utils::CowUtils;
use rustc_hash::FxHashMap;

use crate::{
  css::{self, CssCtx},
  error::{LResult, LessError},
  eval::{create_derived, merge_rules},
  session::Session,
  tree::*,
};

// ===========================================================================
// JoinSelectorVisitor
// ===========================================================================

/// `budget`: the arena bytes the compilation may use (checked per ruleset).
pub(crate) fn join_selectors<'a>(
  a: Alloc<'a>,
  root: &'a Ruleset<'a>,
  budget: usize,
) -> LResult<()> {
  let mut contexts: Vec<Option<&'a Ruleset<'a>>> = vec![None];
  join_visit(
    a,
    Node::Ruleset(root),
    &mut contexts,
    true,
    &mut Vec::new(),
    budget,
  )
}

fn context_empty(contexts: &[Option<&Ruleset<'_>>]) -> bool {
  match contexts.last() {
    Some(Some(r)) => r.paths.get().map(|p| p.is_empty()).unwrap_or(true),
    _ => true,
  }
}

/// `clean`: no node between the context ruleset (exclusive) and `n` blocks
/// visibility. `mark_visible` then reaches a ruleset exactly when it reaches
/// its context, so the context's selectors may serve in its paths in place of
/// `deriveSelector` copies (see `join_selector`).
/// `buf`: scratch for the paths of one ruleset (complete before recursing).
fn join_visit<'a>(
  a: Alloc<'a>,
  n: Node<'a>,
  contexts: &mut Vec<Option<&'a Ruleset<'a>>>,
  clean: bool,
  buf: &mut Vec<SelectorPath<'a>>,
  budget: usize,
) -> LResult<()> {
  let share = clean && !n.vis().is_some_and(|v| v.blocks_visibility());
  match n {
    Node::Ruleset(r) => {
      if !r.root.get() {
        let paths = &mut *buf;
        paths.clear();
        {
          let parent = contexts.last().copied().flatten();
          let context: &[SelectorPath<'a>] = parent.and_then(|p| p.paths.get()).unwrap_or(&[]);
          let mut selectors = r.selectors.get();
          if let Some(sels) = selectors {
            if !sels.iter().all(|s| s.evald_condition.get()) {
              let kept: Vec<&'a Selector<'a>> = sels
                .iter()
                .copied()
                .filter(|s| s.evald_condition.get())
                .collect();
              selectors = if kept.is_empty() {
                None
              } else {
                Some(a.slice(&kept))
              };
            } else if sels.is_empty() {
              selectors = None;
            }
            r.selectors.set(selectors);
          }
          match selectors {
            Some(sels) => {
              for s in sels {
                join_selector(a, paths, context, s, share)?;
              }
            }
            None => *r.rules.borrow_mut() = None,
          }
        }
        r.paths.set(Some(a.slice(paths)));
        if a.bump.allocated_bytes() > budget {
          return Err(crate::error::out_of_memory(budget));
        }
        contexts.push(Some(r));
      } else {
        contexts.push(None);
      }
      let mut i = 0;
      while let Some(rule) = r.rule_at(i) {
        i += 1;
        join_visit(a, rule, contexts, true, buf, budget)?;
      }
      contexts.pop();
    }
    Node::Media(m) => {
      let context_empty = context_empty(contexts);
      if let Some(body) = m.body() {
        body.root.set(context_empty);
      }
      visit_rules_of(a, m, contexts, share, buf, budget)?;
    }
    Node::Container(m) => visit_rules_of(a, m, contexts, share, buf, budget)?,
    Node::AtRule(at) => {
      let context_empty = context_empty(contexts);
      if at
        .declarations
        .get()
        .map(|d| !d.is_empty())
        .unwrap_or(false)
      {
        // `declarations[0].root = ...` has no effect on declarations.
      } else if let Some(body) = at.body() {
        body.root.set(at.is_rooted || context_empty);
      }
      visit_rules_of(a, at, contexts, share, buf, budget)?;
    }
    Node::Declaration(_) | Node::MixinDefinition(_) => {}
    _ => {}
  }
  Ok(())
}

fn visit_rules_of<'a>(
  a: Alloc<'a>,
  at: &'a AtRule<'a>,
  contexts: &mut Vec<Option<&'a Ruleset<'a>>>,
  share: bool,
  buf: &mut Vec<SelectorPath<'a>>,
  budget: usize,
) -> LResult<()> {
  if let Some(rules) = at.rules.get() {
    for r in rules {
      join_visit(a, *r, contexts, share, buf, budget)?;
    }
  }
  Ok(())
}

type ElVec<'a> = smallvec::SmallVec<[&'a Element<'a>; 8]>;

/// A selector under construction (`elements` is mutated in less.js).
#[derive(Clone)]
struct SelB<'a> {
  elements: ElVec<'a>,
  /// The selector whose properties `createDerived` copies.
  base: Option<&'a Selector<'a>>,
  /// An explicit extend list (`createDerived(els, extendList)`); `None`
  /// keeps the base's.
  ext: Option<&'a [&'a Extend<'a>]>,
}

impl<'a> SelB<'a> {
  fn new_plain(elements: ElVec<'a>) -> SelB<'a> {
    SelB {
      elements,
      base: None,
      ext: None,
    }
  }
  fn derived(
    base: &'a Selector<'a>,
    elements: ElVec<'a>,
    ext: Option<&'a [&'a Extend<'a>]>,
  ) -> SelB<'a> {
    SelB {
      elements,
      base: Some(base),
      ext,
    }
  }
  fn build(&self, a: Alloc<'a>) -> &'a Selector<'a> {
    match self.base {
      Some(b) => create_derived(a, b, &self.elements, self.ext, None),
      None => a.selector(
        &self.elements,
        self.ext,
        None,
        Pos::NONE,
        VisInfo::default(),
      ),
    }
  }
  /// `lastSelector.createDerived(lastSelector.elements, extendList)`
  fn with_extend(mut self, ext: Option<&'a [&'a Extend<'a>]>) -> SelB<'a> {
    if ext.is_some() {
      self.ext = ext;
    }
    self
  }
}

fn find_nested_selector<'a>(el: &'a Element<'a>) -> Option<&'a Selector<'a>> {
  if let ElValue::Node(Node::Paren(p)) = el.value
    && let Node::Selector(s) = p.value
  {
    return Some(s);
  }
  None
}

/// `Ruleset.joinSelector(paths, context, selector)`
pub(crate) fn join_selector<'a>(
  a: Alloc<'a>,
  paths: &mut Vec<SelectorPath<'a>>,
  context: &[SelectorPath<'a>],
  selector: &'a Selector<'a>,
  share: bool,
) -> LResult<()> {
  let mut new_paths: Vec<Vec<SelB<'a>>> = Vec::new();
  // Fast path: without `&` or nested selectors nothing is replaced.
  let needs_replace = selector
    .elements
    .iter()
    .any(|e| matches!(e.value, ElValue::Str("&")) || find_nested_selector(e).is_some());
  let had_parent = needs_replace && replace_parent_selector(a, &mut new_paths, context, selector)?;
  if !had_parent {
    if !context.is_empty() {
      for ctx in context {
        let mut concatenated: BVec<'a, &'a Selector<'a>> =
          BVec::with_capacity_in(ctx.len() + 1, a.bump);
        for s in ctx.iter() {
          // `deriveSelector` copies the parent with the child's
          // visibility; when that changes nothing the parent itself
          // serves (visibility marking depends only on that state)
          if share && !s.has_aux() && s.vis.info() == selector.vis.info() {
            concatenated.push(*s);
            continue;
          }
          let d = crate::eval::create_derived_owned(
            a,
            s,
            s.elements,
            s.extend_list(),
            Some(s.evald_condition.get()),
          );
          d.vis.copy_from(selector.vis.info());
          concatenated.push(d);
        }
        concatenated.push(selector);
        paths.push(concatenated.into_bump_slice());
      }
    } else {
      paths.push(a.slice(&[selector]));
    }
    return Ok(());
  }
  let new_paths: Vec<Vec<&'a Selector<'a>>> = {
    new_paths
      .iter()
      .map(|p| p.iter().map(|s| s.build(a)).collect())
      .collect()
  };
  for p in new_paths {
    paths.push(a.slice(&p));
  }
  Ok(())
}

fn merge_elements_onto_selectors<'a>(
  elements: &[&'a Element<'a>],
  selectors: &mut Vec<Vec<SelB<'a>>>,
) {
  if elements.is_empty() {
    return;
  }
  if selectors.is_empty() {
    selectors.push(vec![SelB::new_plain(elements.iter().copied().collect())]);
    return;
  }
  for sel in selectors.iter_mut() {
    if !sel.is_empty() {
      let last = sel.last_mut().expect("selector is nonempty");
      // createDerived(elements.concat(...)) keeps the original's props
      last.elements.extend_from_slice(elements);
    } else {
      sel.push(SelB::new_plain(elements.iter().copied().collect()));
    }
  }
}

fn add_replacement_into_path<'a>(
  a: Alloc<'a>,
  beginning_path: &[SelB<'a>],
  add_path: &[&'a Selector<'a>],
  replaced: &'a Element<'a>,
  original: &'a Selector<'a>,
) -> Vec<SelB<'a>> {
  let mut new_path: Vec<SelB<'a>>;
  let mut joined: SelB<'a>;
  if !beginning_path.is_empty() {
    new_path = beginning_path.to_vec();
    let last = new_path.pop().expect("beginning path is nonempty");
    joined = SelB::derived(original, last.elements.clone(), None);
  } else {
    new_path = Vec::new();
    joined = SelB::derived(original, ElVec::new(), None);
  }
  if !add_path.is_empty() {
    let mut combinator = replaced.combinator.get();
    let parent_el = add_path[0].elements[0];
    if combinator.empty_or_whitespace() && !parent_el.combinator.get().empty_or_whitespace() {
      combinator = parent_el.combinator.get();
    }
    joined.elements.push(a.element(
      combinator,
      parent_el.value,
      replaced.is_variable,
      replaced.pos,
      VisInfo::default(),
    ));
    joined
      .elements
      .extend_from_slice(&add_path[0].elements[1..]);
  }
  if !joined.elements.is_empty() {
    new_path.push(joined);
  }
  if add_path.len() > 1 {
    for s in &add_path[1..] {
      // selector.createDerived(selector.elements, [])
      let d = SelB::derived(s, s.elements.iter().copied().collect(), Some(&[]));
      new_path.push(d);
    }
  }
  new_path
}

fn add_all_replacements_into_path<'a>(
  a: Alloc<'a>,
  beginning_paths: &[Vec<SelB<'a>>],
  add_paths: &[&'a Selector<'a>],
  replaced: &'a Element<'a>,
  original: &'a Selector<'a>,
  result: &mut Vec<Vec<SelB<'a>>>,
) {
  for bp in beginning_paths {
    result.push(add_replacement_into_path(
      a, bp, add_paths, replaced, original,
    ));
  }
}

fn create_parenthesis<'a>(
  a: Alloc<'a>,
  elements_to_pak: &[&'a Selector<'a>],
  original: &'a Element<'a>,
) -> Node<'a> {
  if elements_to_pak.is_empty() {
    // `new Paren(elementsToPak[0])` → Paren(undefined)
    return a.paren(a.null_anonymous());
  }
  let inside: Vec<&'a Element<'a>> = elements_to_pak
    .iter()
    .map(|s| {
      a.element(
        Combinator::EMPTY,
        ElValue::Node(Node::Selector(s)),
        original.is_variable,
        original.pos,
        VisInfo::default(),
      )
    })
    .collect();
  let sel = a.selector(&inside, None, None, Pos::NONE, VisInfo::default());
  a.paren(Node::Selector(sel))
}

fn create_selector<'a>(
  a: Alloc<'a>,
  contained: Node<'a>,
  original: &'a Element<'a>,
) -> &'a Selector<'a> {
  let el = a.element(
    Combinator::EMPTY,
    ElValue::Node(contained),
    original.is_variable,
    original.pos,
    VisInfo::default(),
  );
  a.selector(&[el], None, None, Pos::NONE, VisInfo::default())
}

/// Each `&` multiplies a selector's paths by its parent's: less.js runs out
/// of memory on long `&` chains over selector lists; less-rs stops here,
/// and when the paths would hold more elements than `MAX_JOINED_ELEMENTS`
/// (few long paths multiply too: each holds its parent's elements).
const MAX_JOINED_PATHS: usize = 100_000;
const MAX_JOINED_ELEMENTS: usize = 4_000_000;

fn too_many_selectors() -> Box<LessError> {
  LessError::new(
    "Syntax",
    format!(
      "Too many selectors: the parent selectors `&` combine into more than {} paths or {} elements",
      MAX_JOINED_PATHS, MAX_JOINED_ELEMENTS
    ),
  )
}

/// Elements in a path under construction (each selector counts one more).
fn path_size(p: &[SelB<'_>]) -> usize {
  p.iter().map(|s| s.elements.len() + 1).sum()
}

/// The elements `count` copies of `paths`, each extended by `extra`
/// elements, would hold: `count * Σ size(path) + paths * extra`.
fn joined_size(paths: &[Vec<SelB<'_>>], count: usize, extra: usize) -> usize {
  let own: usize = paths.iter().map(|p| path_size(p)).sum();
  own
    .saturating_mul(count)
    .saturating_add(paths.len().saturating_mul(extra))
}

fn replace_parent_selector<'a>(
  a: Alloc<'a>,
  paths: &mut Vec<Vec<SelB<'a>>>,
  context: &[SelectorPath<'a>],
  in_selector: &'a Selector<'a>,
) -> LResult<bool> {
  let mut had_parent = false;
  let mut current_elements: ElVec<'a> = ElVec::new();
  let mut new_selectors: Vec<Vec<SelB<'a>>> = vec![Vec::new()];
  for el in in_selector.elements {
    if !matches!(el.value, ElValue::Str("&")) {
      if let Some(nested) = find_nested_selector(el) {
        merge_elements_onto_selectors(&current_elements, &mut new_selectors);
        let mut replaced_new: Vec<Vec<SelB<'a>>> = Vec::new();
        if let Some(list) = nested.list() {
          // the `(a, b)` form: resolve each sub-selector
          let mut resolved: Vec<Node<'a>> = Vec::new();
          let mut replaced = false;
          for sub in list {
            if let Node::Selector(ss) = sub {
              let mut sub_paths: Vec<Vec<SelB<'a>>> = Vec::new();
              let r = replace_parent_selector(a, &mut sub_paths, context, ss)?;
              replaced = replaced || r;
              if !sub_paths.is_empty() && !sub_paths[0].is_empty() {
                resolved.push(Node::Selector(sub_paths[0][0].build(a)));
              } else {
                resolved.push(*sub);
              }
            } else {
              resolved.push(*sub);
            }
          }
          had_parent = had_parent || replaced;
          let resolved_sel = a.alloc(Selector::new(
            a,
            &[],
            Some(crate::tree::SelAux::List(a.slice(&resolved))),
            None,
            Pos::NONE,
            VisInfo::default(),
          ));
          let paren = a.paren(Node::Selector(resolved_sel));
          let replacement = create_selector(a, paren, el);
          add_all_replacements_into_path(
            a,
            &new_selectors,
            &[replacement],
            el,
            in_selector,
            &mut replaced_new,
          );
        } else {
          let mut nested_paths: Vec<Vec<SelB<'a>>> = Vec::new();
          let replaced = replace_parent_selector(a, &mut nested_paths, context, nested)?;
          had_parent = had_parent || replaced;
          let nested_size: usize = nested_paths.iter().map(|p| path_size(p)).sum();
          if new_selectors.len().saturating_mul(nested_paths.len()) > MAX_JOINED_PATHS
            || joined_size(&new_selectors, nested_paths.len(), nested_size) > MAX_JOINED_ELEMENTS
          {
            return Err(too_many_selectors());
          }
          for np in &nested_paths {
            let built: Vec<&'a Selector<'a>> = np.iter().map(|s| s.build(a)).collect();
            let replacement = create_selector(a, create_parenthesis(a, &built, el), el);
            add_all_replacements_into_path(
              a,
              &new_selectors,
              &[replacement],
              el,
              in_selector,
              &mut replaced_new,
            );
          }
        }
        new_selectors = replaced_new;
        current_elements.clear();
      } else {
        current_elements.push(el);
      }
    } else {
      had_parent = true;
      let context_size: usize = context
        .iter()
        .map(|p| p.iter().map(|s| s.elements.len() + 1).sum::<usize>())
        .sum();
      if new_selectors.len().saturating_mul(context.len().max(1)) > MAX_JOINED_PATHS
        || joined_size(
          &new_selectors,
          context.len().max(1),
          context_size + current_elements.len() + 1,
        ) > MAX_JOINED_ELEMENTS
      {
        return Err(too_many_selectors());
      }
      let mut multiplied: Vec<Vec<SelB<'a>>> = Vec::new();
      merge_elements_onto_selectors(&current_elements, &mut new_selectors);
      for sel in new_selectors.iter_mut() {
        if context.is_empty() {
          if !sel.is_empty() {
            let e = a.element(
              el.combinator.get(),
              ElValue::Str(""),
              el.is_variable,
              el.pos,
              VisInfo::default(),
            );
            sel[0].elements.push(e);
          }
          multiplied.push(sel.clone());
        } else {
          for ctx in context {
            multiplied.push(add_replacement_into_path(a, sel, ctx, el, in_selector));
          }
        }
      }
      new_selectors = multiplied;
      current_elements.clear();
    }
  }
  merge_elements_onto_selectors(&current_elements, &mut new_selectors);
  for ns in new_selectors.iter_mut() {
    let len = ns.len();
    if len > 0 {
      let last = ns[len - 1].clone();
      ns[len - 1] = last.with_extend(in_selector.extend_list());
      paths.push(ns.clone());
    }
  }
  Ok(had_parent)
}

// ===========================================================================
// SetTreeVisibilityVisitor(true)
// ===========================================================================

pub(crate) fn mark_visible<'a>(root: &'a Ruleset<'a>) {
  vis_visit(Node::Ruleset(root));
}

fn vis_visit(n: Node<'_>) {
  let v = match n.vis() {
    Some(v) => v,
    None => {
      vis_children(n);
      return;
    }
  };
  if v.blocks_visibility() {
    return;
  }
  v.ensure_visibility();
  vis_children(n);
}

fn vis_selector(s: &Selector<'_>) {
  if s.vis.blocks_visibility() {
    return;
  }
  s.vis.ensure_visibility();
  for e in s.elements {
    if !e.vis.blocks_visibility() {
      e.vis.ensure_visibility();
      if let ElValue::Node(n) = e.value {
        vis_visit(n);
      }
    }
  }
  if let Some(ext) = s.extend_list() {
    for e in ext {
      vis_visit(Node::Extend(e));
    }
  }
}

/// Gives the shared declarations (see [`Declaration::SHARED`]) inside
/// visibility-blocked subtrees copies of their own: `mark_visible` does not
/// reach those occurrences, which must not see the visibility it gives to
/// occurrences it does reach (it decides line breaks in the output).
pub(crate) fn unshare_blocked<'a>(a: Alloc<'a>, n: Node<'a>, inside: bool) {
  let inside = inside || n.vis().is_some_and(|v| v.blocks_visibility());
  let copy = |n: Node<'a>| match n {
    Node::Declaration(d) if inside && d.shared() => Some(Node::Declaration(a.declaration_copy(d))),
    _ => None,
  };
  match n {
    Node::Ruleset(r) => {
      let mut i = 0;
      while let Some(rule) = r.rule_at(i) {
        match copy(rule) {
          Some(c) => r.set_rule(i, c),
          None => unshare_blocked(a, rule, inside),
        }
        i += 1;
      }
    }
    Node::AtRule(at) | Node::Media(at) | Node::Container(at) => {
      for cell in [&at.rules, &at.declarations] {
        let Some(list) = cell.get() else { continue };
        if list.iter().any(|n| copy(*n).is_some()) {
          let copied: Vec<Node<'a>> = list.iter().map(|n| copy(*n).unwrap_or(*n)).collect();
          cell.set(Some(a.slice(&copied)));
        }
        for x in cell.get().unwrap_or(&[]) {
          unshare_blocked(a, *x, inside);
        }
      }
    }
    _ => {}
  }
}

fn vis_children(n: Node<'_>) {
  match n {
    Node::Ruleset(r) | Node::MixinDefinition(r) => {
      if let Some(paths) = r.paths.get() {
        for p in paths.iter() {
          for s in p.iter() {
            vis_selector(s);
          }
        }
      } else if let Some(sels) = r.selectors.get() {
        for s in sels {
          vis_selector(s);
        }
      }
      let mut i = 0;
      while let Some(rule) = r.rule_at(i) {
        i += 1;
        vis_visit(rule);
      }
    }
    Node::AtRule(a) | Node::Media(a) | Node::Container(a) => {
      if let Some(f) = a.features.get() {
        vis_visit(f);
      }
      if let Some(r) = a.rules.get() {
        for x in r {
          vis_visit(*x);
        }
      } else if let Some(d) = a.declarations.get() {
        for x in d {
          vis_visit(*x);
        }
      }
      if let Some(v) = a.value {
        vis_visit(v);
      }
    }
    Node::Extend(e) => vis_selector(e.selector),
    Node::Selector(s) => vis_selector(s),
    Node::Declaration(d) => vis_visit(d.value.get()),
    Node::Value(v) => v.value.iter().for_each(|x| vis_visit(*x)),
    Node::Expression(e) => e.value.iter().for_each(|x| vis_visit(*x)),
    Node::Paren(p) => vis_visit(p.value),
    Node::DetachedRuleset(d) => vis_visit(d.ruleset),
    _ => {}
  }
}

// ===========================================================================
// ExtendVisitor
// ===========================================================================

pub(crate) struct ExtendVisitor<'a, 's> {
  s: &'s Session<'a>,
  a: Alloc<'a>,
  /// allExtends of each at-rule scope (by address).
  scope_extends: FxHashMap<usize, Vec<&'a Extend<'a>>>,
  root_extends: Vec<&'a Extend<'a>>,
  found_extends: bool,
  stack: Vec<Vec<&'a Extend<'a>>>,
  /// Per scope level: (extend count, first-element index, non-string firsts).
  index_cache: Vec<(usize, FxHashMap<&'a str, Vec<usize>>, Vec<usize>)>,
}

impl<'a, 's> ExtendVisitor<'a, 's> {
  pub(crate) fn new(s: &'s Session<'a>) -> Self {
    ExtendVisitor {
      s,
      a: s.a,
      scope_extends: FxHashMap::default(),
      root_extends: Vec::new(),
      found_extends: false,
      stack: Vec::new(),
      index_cache: Vec::new(),
    }
  }

  pub(crate) fn run(&mut self, root: &'a Ruleset<'a>) -> LResult<()> {
    // ExtendFinderVisitor
    let mut stack: Vec<Vec<&'a Extend<'a>>> = vec![Vec::new()];
    self.find_visit(Node::Ruleset(root), &mut stack);
    self.root_extends = stack.pop().unwrap_or_default();
    if !self.found_extends {
      return Ok(());
    }
    let all = self.root_extends.clone();
    let chained = self.do_extend_chaining(&all, &all, 0)?;
    self.root_extends.extend(chained);
    self.stack = vec![self.root_extends.clone()];
    self.process_visit(Node::Ruleset(root))?;
    self.check_extends_for_non_matched();
    Ok(())
  }

  fn find_visit(&mut self, n: Node<'a>, stack: &mut Vec<Vec<&'a Extend<'a>>>) {
    match n {
      Node::Ruleset(r) => {
        if !r.root.get() {
          let mut all_selectors_extend: Vec<&'a Extend<'a>> = Vec::new();
          let mut ri = 0;
          while let Some(rule) = r.rule_at(ri) {
            ri += 1;
            if let Node::Extend(e) = rule {
              all_selectors_extend.push(e);
              r.extend_on_every_path.set(true);
            }
          }
          let paths: &[SelectorPath<'a>] = r.paths.get().unwrap_or(&[]);
          if !all_selectors_extend.is_empty()
            || paths.iter().any(|p| {
              p.last()
                .map(|s| s.extend_list().map(|l| !l.is_empty()).unwrap_or(false))
                .unwrap_or(false)
            })
          {
            for path in paths {
              let selector = match path.last() {
                Some(s) => *s,
                None => continue,
              };
              let mut list: Vec<&'a Extend<'a>> = match selector.extend_list() {
                Some(l) => {
                  let mut v = l.to_vec();
                  v.extend_from_slice(&all_selectors_extend);
                  v
                }
                None => all_selectors_extend.clone(),
              };
              list = list
                .iter()
                .map(|e| self.a.extend(e.selector, e.option, e.pos, e.vis.info()))
                .collect();
              for (j, ext) in list.iter().enumerate() {
                self.found_extends = true;
                self.find_self_selectors(ext, path);
                ext.ruleset.set(Some(r));
                if j == 0 {
                  ext.first_extend_on_this_selector_path.set(true);
                }
                stack.last_mut().expect("extend scope is active").push(ext);
              }
            }
          }
        }
        let mut i = 0;
        while let Some(rule) = r.rule_at(i) {
          i += 1;
          self.find_visit(rule, stack);
        }
      }
      Node::Media(m) | Node::AtRule(m) => {
        stack.push(Vec::new());
        if let Some(rules) = m.rules.get() {
          for r in rules {
            self.find_visit(*r, stack);
          }
        }
        let list = stack.pop().unwrap_or_default();
        self.scope_extends.insert(m as *const _ as usize, list);
      }
      Node::Container(m) => {
        if let Some(rules) = m.rules.get() {
          for r in rules {
            self.find_visit(*r, stack);
          }
        }
      }
      _ => {}
    }
  }

  /// `Extend.findSelfSelectors(selectors)`
  fn find_self_selectors(&self, ext: &'a Extend<'a>, selectors: &[&'a Selector<'a>]) {
    let mut self_elements: Vec<&'a Element<'a>> = Vec::new();
    for (i, s) in selectors.iter().enumerate() {
      if i > 0 && !s.elements.is_empty() && s.elements[0].combinator.get().value().is_empty() {
        s.elements[0].combinator.set(Combinator::SPACE);
      }
      self_elements.extend_from_slice(s.elements);
    }
    let sel = self
      .a
      .selector(&self_elements, None, None, Pos::NONE, VisInfo::default());
    sel.vis.copy_from(ext.vis.info());
    ext.self_selectors.set(self.a.slice(&[sel]));
  }

  fn do_extend_chaining(
    &mut self,
    extends_list: &[&'a Extend<'a>],
    target_list: &[&'a Extend<'a>],
    iteration: u32,
  ) -> LResult<Vec<&'a Extend<'a>>> {
    let mut to_add: Vec<&'a Extend<'a>> = Vec::new();
    // `find_match` reads only the extend's selector and `all` flags, and
    // the copies of one extend (one per selector path of its ruleset)
    // share them: the targets each one matches are found once, so many
    // copies cost one pass over the targets, not one each
    type Key<'a> = (*const Selector<'a>, bool, bool);
    let mut matching: rustc_hash::FxHashMap<Key<'a>, Vec<usize>> = Default::default();
    for ext in extends_list {
      let key: Key<'a> = (ext.selector as *const _, ext.allow_before, ext.allow_after);
      let targets = matching.entry(key).or_insert_with(|| {
        (0..target_list.len())
          .filter(|&ti| {
            let Some(&s) = target_list[ti].self_selectors.get().first() else {
              return false;
            };
            !self.find_match(ext, &[s]).is_empty()
          })
          .collect()
      });
      for &ti in targets.iter() {
        let target = target_list[ti];
        if ext.parent_ids.borrow().contains(&target.object_id) {
          continue;
        }
        let target_self = target.self_selectors.get()[0];
        let selector_path = [target_self];
        let matches = self.find_match(ext, &selector_path);
        if !matches.is_empty() {
          ext.has_found_matches.set(true);
          for self_selector in ext.self_selectors.get() {
            let info = target.vis.info();
            let new_selector = self.extend_selector(
              &matches,
              &selector_path,
              self_selector,
              ext.vis.is_visible(),
            );
            let new_ext = self
              .a
              .extend(target.selector, target.option, Pos::NONE, info);
            new_ext.self_selectors.set(self.a.slice(&new_selector));
            if let Some(last) = new_selector.last() {
              last.set_extend_list(self.a, Some(self.a.slice(&[new_ext])));
            }
            to_add.push(new_ext);
            new_ext.ruleset.set(target.ruleset.get());
            {
              let mut pids = new_ext.parent_ids.borrow_mut();
              pids.extend_from_slice(&target.parent_ids.borrow());
              pids.extend_from_slice(&ext.parent_ids.borrow());
            }
            if target.first_extend_on_this_selector_path.get() {
              new_ext.first_extend_on_this_selector_path.set(true);
              if let Some(rs) = target.ruleset.get() {
                rs.push_paths(self.a, &[self.a.slice(&new_selector)]);
              }
            }
          }
        }
      }
    }
    if !to_add.is_empty() {
      if iteration > 100 {
        let mut one = "{unable to calculate}".to_string();
        let mut two = "{unable to calculate}".to_string();
        if let Some(first) = to_add.first() {
          if let Some(s) = first.self_selectors.get().first()
            && let Ok(t) = css::to_css_plain(Node::Selector(s))
          {
            one = t;
          }
          if let Ok(t) = css::to_css_plain(Node::Selector(first.selector)) {
            two = t;
          }
        }
        return Err(LessError::new(
          "",
          format!(
            "extend circular reference detected. One of the circular extends is currently:{}:extend({})",
            one, two
          ),
        ));
      }
      let more = self.do_extend_chaining(&to_add.clone(), target_list, iteration + 1)?;
      to_add.extend(more);
    }
    Ok(to_add)
  }

  fn process_visit(&mut self, n: Node<'a>) -> LResult<()> {
    match n {
      Node::Ruleset(r) => {
        if !r.root.get() {
          self.process_ruleset(r);
        }
        let mut i = 0;
        while let Some(rule) = r.rule_at(i) {
          i += 1;
          self.process_visit(rule)?;
        }
      }
      Node::Media(m) | Node::AtRule(m) => {
        let own = self
          .scope_extends
          .get(&(m as *const _ as usize))
          .cloned()
          .unwrap_or_default();
        let mut new_all: Vec<&'a Extend<'a>> = own.clone();
        new_all.extend(self.stack.last().cloned().unwrap_or_default());
        let chained = self.do_extend_chaining(&new_all.clone(), &own, 0)?;
        new_all.extend(chained);
        self.stack.push(new_all);
        self.index_cache.truncate(self.stack.len() - 1);
        if let Some(rules) = m.rules.get() {
          for r in rules {
            self.process_visit(*r)?;
          }
        }
        self.stack.pop();
        self.index_cache.truncate(self.stack.len());
      }
      Node::Container(m) => {
        if let Some(rules) = m.rules.get() {
          for r in rules {
            self.process_visit(*r)?;
          }
        }
      }
      _ => {}
    }
    Ok(())
  }

  fn process_ruleset(&mut self, r: &'a Ruleset<'a>) {
    if r.extend_on_every_path.get() {
      return;
    }
    let all = std::mem::take(self.stack.last_mut().expect("extend scope is active"));
    if all.is_empty() {
      *self.stack.last_mut().expect("extend scope is active") = all;
      return;
    }
    let paths: &[SelectorPath<'a>] = r.paths.get().unwrap_or(&[]);
    // Candidate (extend, path) pairs via an index on the needle's first
    // element value (built once per scope); the match order
    // (extend-major) is preserved.
    let level = self.stack.len() - 1;
    if self.index_cache.len() <= level || self.index_cache[level].0 != all.len() {
      let mut index: FxHashMap<&'a str, Vec<usize>> = FxHashMap::default();
      let mut always: Vec<usize> = Vec::new();
      for (ei, ext) in all.iter().enumerate() {
        match ext.selector.elements.first().and_then(|e| e.value.as_str()) {
          Some(s) => index.entry(s).or_default().push(ei),
          None => always.push(ei),
        }
      }
      self.index_cache.truncate(level);
      while self.index_cache.len() < level {
        self
          .index_cache
          .push((usize::MAX, FxHashMap::default(), Vec::new()));
      }
      self.index_cache.push((all.len(), index, always));
    }
    let (_, index, always) = std::mem::take(&mut self.index_cache[level]);
    let mut pairs: smallvec::SmallVec<[(usize, usize); 32]> = smallvec::SmallVec::new();
    for (pi, path) in paths.iter().enumerate() {
      if let Some(last) = path.last()
        && last.extend_list().map(|l| !l.is_empty()).unwrap_or(false)
      {
        continue;
      }
      let start = pairs.len();
      for s in path.iter() {
        for e in s.elements {
          if let Some(v) = e.value.as_str()
            && let Some(list) = index.get(v)
          {
            for &ei in list {
              pairs.push((ei, pi));
            }
          }
        }
      }
      for &ei in &always {
        pairs.push((ei, pi));
      }
      let seg = &mut pairs[start..];
      seg.sort_unstable();
      let mut w = start;
      for i in start..pairs.len() {
        if i == start || pairs[i] != pairs[w - 1] {
          pairs[w] = pairs[i];
          w += 1;
        }
      }
      pairs.truncate(w);
    }
    pairs.sort_unstable();
    let mut to_add: smallvec::SmallVec<[SelectorPath<'a>; 4]> = smallvec::SmallVec::new();
    for (ei, pi) in pairs {
      let ext = all[ei];
      let path = paths[pi];
      if !could_match(ext, path) {
        continue;
      }
      let matches = self.find_match(ext, path);
      if !matches.is_empty() {
        ext.has_found_matches.set(true);
        let visible = ext.vis.is_visible();
        for ss in ext.self_selectors.get() {
          let p = self.extend_selector(&matches, path, ss, visible);
          to_add.push(self.a.slice(&p));
        }
      }
    }
    self.index_cache[level] = (all.len(), index, always);
    *self.stack.last_mut().expect("extend scope is active") = all;
    if !to_add.is_empty() {
      r.push_paths(self.a, &to_add);
    }
  }

  fn find_match(&self, ext: &'a Extend<'a>, haystack: &[&'a Selector<'a>]) -> Matches<'a> {
    let needle = ext.selector.elements;
    let mut potential: smallvec::SmallVec<[Match<'a>; 4]> = smallvec::SmallVec::new();
    let mut matches: Matches<'a> = smallvec::SmallVec::new();
    for (hsi, hs) in haystack.iter().enumerate() {
      for (hei, he) in hs.elements.iter().enumerate() {
        if ext.allow_before || (hsi == 0 && hei == 0) {
          potential.push(Match {
            path_index: hsi,
            index: hei,
            matched: 0,
            initial_combinator: he.combinator.get(),
            finished: false,
            end_path_index: 0,
            end_path_element_index: 0,
          });
        }
        let mut i = 0;
        while i < potential.len() {
          let mut keep = true;
          {
            let pm = &mut potential[i];
            let mut target_comb = he.combinator.get().value();
            if target_comb.is_empty() && hei == 0 {
              target_comb = " ";
            }
            if pm.matched >= needle.len()
              || !is_element_values_equal(needle[pm.matched].value, he.value)
              || (pm.matched > 0 && needle[pm.matched].combinator.get().value() != target_comb)
            {
              keep = false;
            } else {
              pm.matched += 1;
            }
            if keep {
              pm.finished = pm.matched == needle.len();
              if pm.finished
                && !ext.allow_after
                && (hei + 1 < hs.elements.len() || hsi + 1 < haystack.len())
              {
                keep = false;
              }
            }
            if keep && pm.finished {
              pm.end_path_index = hsi;
              pm.end_path_element_index = hei + 1;
            }
          }
          if keep {
            if potential[i].finished {
              let m = potential[i].clone();
              potential.clear();
              matches.push(m);
              break;
            }
            i += 1;
          } else {
            potential.remove(i);
          }
        }
      }
    }
    matches
  }

  fn extend_selector(
    &self,
    matches: &[Match<'a>],
    selector_path: &[&'a Selector<'a>],
    replacement: &'a Selector<'a>,
    visible: bool,
  ) -> Vec<&'a Selector<'a>> {
    // path entries: either an existing selector or a new element list
    enum P<'a> {
      Existing(&'a Selector<'a>),
      New(Vec<&'a Element<'a>>),
    }
    let mut cur_path_index = 0usize;
    let mut cur_el_index = 0usize;
    let mut path: Vec<P<'a>> = Vec::new();
    for (mi, m) in matches.iter().enumerate() {
      let selector = selector_path[m.path_index];
      let first = replacement.elements[0];
      let first_element = self.a.element(
        m.initial_combinator,
        first.value,
        first.is_variable,
        first.pos,
        VisInfo::default(),
      );
      if m.path_index > cur_path_index && cur_el_index > 0 {
        let extra = &selector_path[cur_path_index].elements[cur_el_index..];
        append_elements(&mut path, extra);
        cur_el_index = 0;
        cur_path_index += 1;
      }
      let mut new_elements: Vec<&'a Element<'a>> =
        selector.elements[cur_el_index.min(m.index)..m.index].to_vec();
      if cur_el_index > m.index {
        new_elements.clear();
      }
      new_elements.push(first_element);
      new_elements.extend_from_slice(&replacement.elements[1..]);
      if cur_path_index == m.path_index && mi > 0 {
        append_elements(&mut path, &new_elements);
      } else {
        for s in &selector_path[cur_path_index..m.path_index.max(cur_path_index)] {
          path.push(P::Existing(s));
        }
        path.push(P::New(new_elements));
      }
      cur_path_index = m.end_path_index;
      cur_el_index = m.end_path_element_index;
      if cur_el_index >= selector_path[cur_path_index].elements.len() {
        cur_el_index = 0;
        cur_path_index += 1;
      }
    }
    if cur_path_index < selector_path.len() && cur_el_index > 0 {
      let extra = &selector_path[cur_path_index].elements[cur_el_index..];
      append_elements(&mut path, extra);
      cur_path_index += 1;
    }
    if cur_path_index < selector_path.len() {
      for s in &selector_path[cur_path_index..] {
        path.push(P::Existing(s));
      }
    }
    fn append_elements<'a>(path: &mut Vec<P<'a>>, extra: &[&'a Element<'a>]) {
      if let Some(last) = path.last_mut() {
        match last {
          P::New(v) => v.extend_from_slice(extra),
          P::Existing(s) => {
            let mut v = s.elements.to_vec();
            v.extend_from_slice(extra);
            *last = P::New(v);
          }
        }
      }
    }
    path
      .into_iter()
      .map(|p| {
        let d = match p {
          P::Existing(s) => create_derived(self.a, s, s.elements, None, None),
          P::New(els) => {
            let s = self
              .a
              .selector(&els, None, None, Pos::NONE, VisInfo::default());
            create_derived(self.a, s, &els, None, None)
          }
        };
        if visible {
          d.vis.ensure_visibility();
        } else {
          d.vis.ensure_invisibility();
        }
        d
      })
      .collect()
  }

  fn check_extends_for_non_matched(&mut self) {
    for ext in self.root_extends.clone() {
      if !ext.has_found_matches.get() && ext.parent_ids.borrow().len() == 1 {
        let selector =
          css::to_css_plain(Node::Selector(ext.selector)).unwrap_or_else(|_| "_unknown_".into());
        let idx = self.s.locate(ext.pos).map(|(_, i)| i as i64).unwrap_or(0);
        let key = format!("{} {}", idx, selector);
        if self.s.extend_warn_indices.borrow_mut().insert(key) && !self.s.opts.quiet {
          self.s.warn(
            format!("WARNING: extend '{}' has no matches", selector),
            None,
          );
        }
      }
    }
  }
}

/// Cheap pre-filter for `findMatch`: the needle's first element value must
/// occur in the haystack (at its very start unless `allowBefore`).
fn could_match<'a>(ext: &'a Extend<'a>, path: &[&'a Selector<'a>]) -> bool {
  let first = match ext.selector.elements.first() {
    Some(f) => f.value,
    None => return false,
  };
  if !ext.allow_before {
    return path
      .first()
      .and_then(|s| s.elements.first())
      .map(|e| is_element_values_equal(first, e.value))
      .unwrap_or(false);
  }
  path.iter().any(|s| {
    s.elements
      .iter()
      .any(|e| is_element_values_equal(first, e.value))
  })
}

type Matches<'a> = smallvec::SmallVec<[Match<'a>; 2]>;

#[derive(Clone)]
struct Match<'a> {
  path_index: usize,
  index: usize,
  matched: usize,
  initial_combinator: Combinator<'a>,
  finished: bool,
  end_path_index: usize,
  end_path_element_index: usize,
}

fn is_element_values_equal<'a>(v1: ElValue<'a>, v2: ElValue<'a>) -> bool {
  match (v1, v2) {
    (ElValue::Str(a), ElValue::Str(b)) => a == b,
    (ElValue::Str(_), _) | (_, ElValue::Str(_)) => false,
    (ElValue::Node(a), ElValue::Node(b)) => node_values_equal(a, b),
  }
}

fn node_values_equal<'a>(a: Node<'a>, b: Node<'a>) -> bool {
  if let Node::Attribute(x) = a {
    let y = match b {
      Node::Attribute(y) => y,
      _ => return false,
    };
    if x.op != y.op {
      return false;
    }
    let key_eq = match (x.key, y.key) {
      (StrOrNode::Str(p), StrOrNode::Str(q)) => p == q,
      (StrOrNode::Node(p), StrOrNode::Node(q)) => p.ptr_eq(&q),
      _ => false,
    };
    if !key_eq {
      return false;
    }
    match (x.value, y.value) {
      (None, None) => return true,
      (None, _) | (_, None) => return false,
      (Some(p), Some(q)) => {
        let s = |v: StrOrNode<'_>| -> Option<String> {
          match v {
            StrOrNode::Str(s) => Some(s.to_string()),
            StrOrNode::Node(n) => n.str_value().map(|s| s.to_string()),
          }
        };
        return s(p).is_some() && s(p) == s(q);
      }
    }
  }
  // `elementValue1.value` / `elementValue2.value`
  let inner = |n: Node<'a>| -> Option<Node<'a>> {
    match n {
      Node::Paren(p) => Some(p.value),
      _ => None,
    }
  };
  let (sa, sb) = match (inner(a), inner(b)) {
    (Some(Node::Selector(x)), Some(Node::Selector(y))) => (x, y),
    _ => return false,
  };
  if sa.list().is_some() || sb.list().is_some() {
    return false;
  }
  if sa.elements.len() != sb.elements.len() {
    return false;
  }
  for i in 0..sa.elements.len() {
    let ca = sa.elements[i].combinator.get().value();
    let cb = sb.elements[i].combinator.get().value();
    if ca != cb {
      let na = if ca.is_empty() { " " } else { ca };
      let nb = if cb.is_empty() { " " } else { cb };
      if i != 0 || na != nb {
        return false;
      }
    }
    if !is_element_values_equal(sa.elements[i].value, sb.elements[i].value) {
      return false;
    }
  }
  true
}

// ===========================================================================
// ToCSSVisitor
// ===========================================================================

pub(crate) struct ToCssVisitor<'a, 's> {
  s: &'s Session<'a>,
  a: Alloc<'a>,
  charset: bool,
  compress: bool,
  /// Reused buffers for `visit_ruleset` (it recurses).
  bufs: Vec<Vec<Node<'a>>>,
}

impl<'a, 's> ToCssVisitor<'a, 's> {
  pub(crate) fn new(s: &'s Session<'a>) -> Self {
    ToCssVisitor {
      s,
      a: s.a,
      charset: false,
      compress: s.opts.compress,
      bufs: Vec::new(),
    }
  }

  pub(crate) fn run(&mut self, root: &'a Ruleset<'a>) -> LResult<()> {
    self.visit(Node::Ruleset(root))?;
    Ok(())
  }

  /// Visits a node; returns its replacement list (empty = removed).
  fn visit(&mut self, n: Node<'a>) -> LResult<Out<'a>> {
    match n {
      Node::Declaration(d) => {
        if d.vis.blocks_visibility() || d.variable {
          return Ok(Out::new());
        }
        if self.compress {
          // visitDeeper: silent comments inside values are dropped
          let v = d.value.get();
          if let Some(nv) = self.strip_comments(v) {
            d.value.set(nv);
          }
        }
        Ok(smallvec::smallvec![n])
      }
      Node::MixinDefinition(_) | Node::Extend(_) => Ok(Out::new()),
      Node::Comment(c) => {
        if c.vis.blocks_visibility() || c.is_silent(self.compress) {
          return Ok(Out::new());
        }
        Ok(smallvec::smallvec![n])
      }
      Node::Media(m) => {
        self.visit_at_children(m)?;
        Ok(
          self
            .resolve_visibility(m)
            .map(|x| smallvec::smallvec![x])
            .unwrap_or_default(),
        )
      }
      Node::Import(i) => {
        if i.vis.blocks_visibility() {
          return Ok(Out::new());
        }
        Ok(smallvec::smallvec![n])
      }
      Node::AtRule(at) => {
        if at.rules.get().map(|r| !r.is_empty()).unwrap_or(false) {
          self.visit_at_rule_with_body(at)
        } else {
          self.visit_at_rule_without_body(at)
        }
      }
      Node::Anonymous(an) => {
        if !an.vis.blocks_visibility() {
          return Ok(smallvec::smallvec![n]);
        }
        Ok(Out::new())
      }
      Node::Ruleset(r) => self.visit_ruleset(r),
      Node::Container(m) => {
        self.visit_at_children(m)?;
        Ok(smallvec::smallvec![n])
      }
      // a JS array in a rules list is flattened (without being visited)
      Node::Rules(list) => Ok(list.iter().copied().collect()),
      other => Ok(smallvec::smallvec![other]),
    }
  }

  /// Removes silent comments from value arrays; `None` when unchanged.
  fn strip_comments(&self, n: Node<'a>) -> Option<Node<'a>> {
    let compress = self.compress;
    let filter = |items: &'a [Node<'a>]| -> Option<Vec<Node<'a>>> {
      let mut changed = false;
      let mut out = Vec::with_capacity(items.len());
      for it in items {
        if let Node::Comment(c) = it
          && (c.vis.blocks_visibility() || c.is_silent(compress))
        {
          changed = true;
          continue;
        }
        match self.strip_comments(*it) {
          Some(x) => {
            changed = true;
            out.push(x);
          }
          None => out.push(*it),
        }
      }
      if changed { Some(out) } else { None }
    };
    match n {
      Node::Expression(e) => {
        filter(e.value).map(|v| Node::Expression(self.a.expression(&v, e.no_spacing.get())))
      }
      Node::Value(v) => filter(v.value).map(|x| self.a.value(&x)),
      _ => None,
    }
  }

  fn visit_array(&mut self, nodes: &[Node<'a>]) -> LResult<Vec<Node<'a>>> {
    let mut out = Vec::with_capacity(nodes.len());
    for n in nodes {
      out.extend(self.visit(*n)?);
    }
    Ok(out)
  }

  fn visit_at_children(&mut self, at: &'a AtRule<'a>) -> LResult<()> {
    if let Some(rules) = at.rules.get() {
      let nr = self.visit_array(rules)?;
      at.rules.set(Some(self.a.slice(&nr)));
    } else if let Some(d) = at.declarations.get() {
      let nd = self.visit_array(d)?;
      at.declarations.set(Some(self.a.slice(&nd)));
    }
    Ok(())
  }

  fn resolve_visibility(&mut self, at: &'a AtRule<'a>) -> Option<Node<'a>> {
    let is_empty = |at: &AtRule<'a>| at.rules.get().map(|r| r.is_empty()).unwrap_or(true);
    if !at.vis.blocks_visibility() {
      if is_empty(at) {
        return None;
      }
      return Some(at.node());
    }
    let body = at.rules.get().and_then(|r| r.first().copied());
    match body.and_then(|b| b.as_ruleset()) {
      Some(rs) => {
        if let Some(rules) = rs.rules.borrow_mut().as_mut() {
          rules.retain(|r| r.is_visible());
        }
        if rs.rules_len() == 0 {
          return None;
        }
      }
      None => return None,
    }
    at.vis.ensure_visibility();
    at.vis.remove_block();
    Some(at.node())
  }

  fn visit_at_rule_with_body(&mut self, at: &'a AtRule<'a>) -> LResult<Out<'a>> {
    self.visit_at_children(at)?;
    if let Some(rules) = at.rules.get()
      && !rules.is_empty()
      && let Some(rs) = rules[0].as_ruleset()
    {
      let mut v = rs.rules_vec();
      merge_rules(self.a, &mut v);
      if rs.has_rules() {
        *rs.rules.borrow_mut() = Some(self.a.vec_from(&v));
      }
    }
    Ok(
      self
        .resolve_visibility(at)
        .map(|x| smallvec::smallvec![x])
        .unwrap_or_default(),
    )
  }

  fn visit_at_rule_without_body(&mut self, at: &'a AtRule<'a>) -> LResult<Out<'a>> {
    if at.vis.blocks_visibility() {
      return Ok(Out::new());
    }
    if at.name == "@charset" {
      if self.charset {
        if let Some(d) = at.debug_info {
          let c = self.a.alloc(Comment {
            vis: Vis::default(),
            value: self.a.str(&format!("/* {} */\n", "")),
            is_line_comment: false,
            pos: Pos::NONE,
            debug_info: std::cell::Cell::new(Some(d)),
          });
          let _ = c;
          // `new tree.Comment('/* ' + toCSS + ' */\n')` with debugInfo
          let text = css::to_css(Node::AtRule(at), &mut CssCtx::default()).unwrap_or_default();
          let c2 = self.a.alloc(Comment {
            vis: Vis::default(),
            value: self
              .a
              .str(&format!("/* {} */\n", text.cow_replace('\n', ""))),
            is_line_comment: false,
            pos: Pos::NONE,
            debug_info: std::cell::Cell::new(Some(d)),
          });
          return self.visit(Node::Comment(c2));
        }
        return Ok(Out::new());
      }
      self.charset = true;
    }
    // `visitDeeper` stays true: the declarations are visited too.
    if let Some(d) = at.declarations.get() {
      let nd = self.visit_array(d)?;
      at.declarations.set(Some(self.a.slice(&nd)));
    }
    Ok(smallvec::smallvec![Node::AtRule(at)])
  }

  fn check_valid_nodes(&self, rules: &[Node<'a>], is_root: bool) -> LResult<()> {
    for r in rules {
      if is_root
        && let Node::Declaration(d) = r
        && !d.variable
      {
        return Err(LessError::at(
          "",
          "Properties must be inside selector blocks. They cannot be in the root",
          d.pos,
        ));
      }
      if let Node::Call(c) = r {
        return Err(LessError::at(
          "",
          format!("Function '{}' did not return a root node", c.name),
          c.pos,
        ));
      }
      if !r.allow_root() && !matches!(r, Node::Rules(_)) {
        return Err(LessError::at(
          "",
          format!(
            "{} node returned by a function is not valid here",
            r.type_name()
          ),
          r.pos(),
        ));
      }
    }
    Ok(())
  }

  fn visit_ruleset(&mut self, r: &'a Ruleset<'a>) -> LResult<Out<'a>> {
    {
      let b = r.rules.borrow();
      self.check_valid_nodes(b.as_deref().unwrap_or(&[]), r.first_root.get())?;
    }
    let taken = r.rules.borrow_mut().take();
    // visited rules are collected here and written back into the taken
    // list's buffer (no new list per ruleset)
    let mut out: Vec<Node<'a>> = self.bufs.pop().unwrap_or_default();
    out.clear();
    let r = self.visit_ruleset_into(r, taken, &mut out);
    self.bufs.push(out);
    r
  }

  fn visit_ruleset_into(
    &mut self,
    r: &'a Ruleset<'a>,
    taken: Option<BVec<'a, Node<'a>>>,
    out: &mut Vec<Node<'a>>,
  ) -> LResult<Out<'a>> {
    let mut rulesets: Out<'a> = Out::new();
    if !r.root.get() {
      self.compile_ruleset_paths(r);
      let mut rest: smallvec::SmallVec<[Node<'a>; 16]> = smallvec::SmallVec::new();
      if let Some(list) = &taken {
        for rule in list.iter() {
          if has_rules(*rule) {
            rulesets.extend(self.visit(*rule)?);
          } else {
            rest.push(*rule);
          }
        }
      }
      if rest.is_empty() {
        // rules = null
        if self.is_visible_ruleset_with(r, false) {
          r.vis.ensure_visibility();
          rulesets.insert(0, Node::Ruleset(r));
        }
        return Ok(rulesets);
      }
      out.reserve(rest.len());
      for n in rest {
        out.extend(self.visit(n)?);
      }
    } else {
      match &taken {
        Some(list) => {
          out.reserve(list.len());
          for n in list.iter() {
            out.extend(self.visit(*n)?);
          }
        }
        None => {
          if self.is_visible_ruleset_with(r, false) {
            r.vis.ensure_visibility();
            rulesets.insert(0, Node::Ruleset(r));
          }
          return Ok(rulesets);
        }
      }
    }
    if out
      .iter()
      .any(|n| matches!(n, Node::Declaration(d) if d.merge.get() != Merge::None))
    {
      merge_rules(self.a, out);
    }
    self.remove_duplicate_rules(out)?;
    let non_empty = !out.is_empty();
    let mut list = taken.unwrap_or_else(|| self.a.vec());
    list.clear();
    list.extend_from_slice(out);
    *r.rules.borrow_mut() = Some(list);
    if self.is_visible_ruleset_with(r, non_empty) {
      r.vis.ensure_visibility();
      rulesets.insert(0, Node::Ruleset(r));
    }
    Ok(rulesets)
  }

  fn is_visible_ruleset_with(&self, r: &'a Ruleset<'a>, non_empty: bool) -> bool {
    if r.first_root.get() {
      return true;
    }
    if !non_empty {
      return false;
    }
    if !r.root.get() {
      let has_visible = r.paths.get().map(|p| !p.is_empty()).unwrap_or(false);
      if !has_visible {
        return false;
      }
    }
    true
  }

  fn compile_ruleset_paths(&self, r: &'a Ruleset<'a>) {
    // `p[0].elements[0].combinator = new Combinator('')` is applied when
    // the CSS is generated (`CssCtx::blank_first_combinator`), since
    // elements may be shared.
    if let Some(old) = r.paths.get() {
      let keep = |path: &SelectorPath<'a>| {
        path
          .iter()
          .any(|s| s.vis.is_visible() && s.evald_condition.get())
      };
      if !old.iter().all(keep) {
        let mut v = self.a.vec_from(old);
        v.retain(|path| keep(path));
        r.paths.set(Some(v.into_bump_slice()));
      }
    }
  }

  fn remove_duplicate_rules_b(&self, rules: &mut BVec<'a, Node<'a>>) -> LResult<()> {
    let mut has_dup = false;
    {
      let mut names: smallvec::SmallVec<[&str; 16]> = smallvec::SmallVec::new();
      for r in rules.iter() {
        if let Node::Declaration(d) = r {
          names.push(d.name_str().unwrap_or(""));
        }
      }
      names.sort_unstable();
      if names.windows(2).any(|w| w[0] == w[1]) {
        has_dup = true;
      }
    }
    if !has_dup {
      return Ok(());
    }
    let mut v: Vec<Node<'a>> = rules.to_vec();
    self.remove_duplicate_rules(&mut v)?;
    rules.clear();
    rules.extend_from_slice(&v);
    Ok(())
  }

  fn remove_duplicate_rules(&self, rules: &mut Vec<Node<'a>>) -> LResult<()> {
    // fast path: no two declarations share a name
    {
      let mut names: smallvec::SmallVec<[&str; 16]> = smallvec::SmallVec::new();
      for r in rules.iter() {
        if let Node::Declaration(d) = r {
          names.push(d.name_str().unwrap_or(""));
        }
      }
      names.sort_unstable();
      if !names.windows(2).any(|w| w[0] == w[1]) {
        return Ok(());
      }
    }
    enum Entry<'a> {
      One(&'a Declaration<'a>),
      Many(Vec<String>),
    }
    let mut cache: FxHashMap<&'a str, Entry<'a>> = FxHashMap::default();
    let mut ctx = CssCtx {
      compress: self.compress,
      ..CssCtx::default()
    };
    let mut i = rules.len();
    while i > 0 {
      i -= 1;
      if let Node::Declaration(d) = rules[i] {
        let name = d.name_str().unwrap_or("");
        match cache.get_mut(name) {
          None => {
            cache.insert(name, Entry::One(d));
          }
          Some(entry) => {
            if let Entry::One(prev) = entry {
              let prev_css = css::to_css(Node::Declaration(prev), &mut ctx)?;
              *entry = Entry::Many(vec![prev_css]);
            }
            let rule_css = css::to_css(Node::Declaration(d), &mut ctx)?;
            if let Entry::Many(list) = entry {
              if list.contains(&rule_css) {
                rules.remove(i);
              } else {
                list.push(rule_css);
              }
            }
          }
        }
      }
    }
    Ok(())
  }
}

type Out<'a> = smallvec::SmallVec<[Node<'a>; 2]>;

/// `rule && rule.rules` (truthy rules array).
fn has_rules(n: Node<'_>) -> bool {
  match n {
    Node::Ruleset(r) | Node::MixinDefinition(r) => r.has_rules(),
    Node::Media(a) | Node::Container(a) | Node::AtRule(a) => a.rules.get().is_some(),
    _ => false,
  }
}
