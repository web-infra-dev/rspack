//! `Ruleset.eval` and the scope helpers of `tree/ruleset.js`.

use cow_utils::CowUtils;

use super::EvalCtx;
use crate::{
  error::LResult,
  session::DefaultState,
  syntax::parser::{ParseFn, Parsed, Parser},
  tree::*,
};

impl<'a, 's> EvalCtx<'a, 's> {
  pub(crate) fn eval_ruleset(&mut self, rs: &'a Ruleset<'a>) -> LResult<&'a Ruleset<'a>> {
    self.eval_ruleset_as(rs, false)
  }

  /// `new Ruleset(null, rs.rules).eval(context)` without allocating the
  /// intermediate ruleset.
  pub(crate) fn eval_ruleset_plain(&mut self, rs: &'a Ruleset<'a>) -> LResult<&'a Ruleset<'a>> {
    self.eval_ruleset_as(rs, true)
  }

  fn eval_ruleset_as(&mut self, rs: &'a Ruleset<'a>, plain: bool) -> LResult<&'a Ruleset<'a>> {
    self.enter_pub()?;
    let r = if self.depth.is_multiple_of(4) {
      super::grow(|| self.eval_ruleset_as_inner(rs, plain))
    } else {
      self.eval_ruleset_as_inner(rs, plain)
    };
    self.leave_pub();
    r
  }

  fn eval_ruleset_as_inner(
    &mut self,
    rs: &'a Ruleset<'a>,
    plain: bool,
  ) -> LResult<&'a Ruleset<'a>> {
    let mut selectors: Option<&'a [&'a Selector<'a>]> = None;
    let mut has_one_passing = false;
    match if plain { None } else { rs.selectors.get() } {
      Some(sels) if !sels.is_empty() => {
        self.s.default_state.set(DefaultState::Error);
        let mut evald: BVec<'a, &'a Selector<'a>> = BVec::with_capacity_in(sels.len(), self.a.bump);
        let mut has_variable = false;
        for sel in sels {
          let s = self.eval_selector(sel)?;
          if s.elements.iter().any(|e| e.is_variable) {
            has_variable = true;
          }
          if s.evald_condition.get() {
            has_one_passing = true;
          }
          evald.push(s);
        }
        if has_variable {
          let mut parts = Vec::with_capacity(evald.len());
          for s in &evald {
            parts.push(self.to_css(Node::Selector(s))?);
          }
          let text = self.str(&parts.join(","));
          let start = evald[0].pos;
          if let Some(sels) = self.parse_selectors(text, start)? {
            evald = self.a.vec_from(&sels);
          }
        }
        self.s.default_state.set(DefaultState::Unset);
        selectors = Some(evald.into_bump_slice());
      }
      _ => has_one_passing = true,
    }
    let rules = if plain {
      None
    } else {
      rs.rules.borrow().as_ref().map(|r| self.a.vec_from(r))
    };
    let ruleset = if plain {
      let a = self.a;
      let original = *self.s.plain_original.get_or_init(|| a.ruleset(None, None));
      drop(rules);
      let ruleset = {
        let src = rs.rules.borrow();
        self.pooled_ruleset(src.as_deref())
      };
      ruleset.original_ruleset.set(Some(original));
      ruleset
    } else {
      let ruleset = self.a.ruleset_full(
        selectors,
        rules,
        rs.strict_imports,
        rs.vis.info(),
        None,
        rs.pos,
      );
      ruleset.original_ruleset.set(Some(rs));
      ruleset.root.set(rs.root.get());
      ruleset.first_root.set(rs.first_root.get());
      ruleset.allow_imports.set(rs.allow_imports.get());
      if let Some(d) = rs.debug_info() {
        ruleset.set_debug_info(self.a, Some(d));
      }
      ruleset
    };
    if !has_one_passing {
      ruleset.bump_generation();
      if let Some(r) = ruleset.rules.borrow_mut().as_mut() {
        r.clear();
      }
    }
    self.frames.insert(0, ruleset);
    let r = self.eval_ruleset_body(ruleset, selectors);
    self.frames.remove(0);
    r?;
    Ok(ruleset)
  }

  fn eval_ruleset_body(
    &mut self,
    ruleset: &'a Ruleset<'a>,
    selectors: Option<&'a [&'a Selector<'a>]>,
  ) -> LResult<()> {
    if ruleset.root.get() || ruleset.allow_imports.get() || !ruleset.strict_imports {
      self.eval_imports(ruleset)?;
    }
    // evalFirst rules (mixin definitions, detached rulesets)
    let mut i = 0;
    while let Some(rule) = ruleset.rule_at(i) {
      if matches!(rule, Node::MixinDefinition(_) | Node::DetachedRuleset(_)) {
        let v = self.eval(rule)?;
        ruleset.set_rule(i, v);
        self.find_index_replaced(ruleset, i, v);
      }
      i += 1;
    }
    let media_block_count = self.media_blocks.as_ref().map(|m| m.len()).unwrap_or(0);
    // mixin calls and variable calls; the rulesets they return are held
    // only by their (discarded) result containers
    let mut fresh: smallvec::SmallVec<[usize; 16]> = smallvec::SmallVec::new();
    let mut i = 0;
    while let Some(rule) = ruleset.rule_at(i) {
      match rule {
        Node::MixinCall(mc) => {
          let rules = self.eval_mixin_call(mc)?;
          let mut filtered = Vec::with_capacity(rules.len());
          for r in rules {
            if let Node::Declaration(d) = r
              && d.variable
              && let Some(name) = d.name_str()
              && self.frame_variable(ruleset, name)?.is_some()
            {
              continue;
            }
            filtered.push(r);
          }
          let n = filtered.len();
          fresh.extend(filtered.iter().filter_map(|r| match r {
            Node::Ruleset(x) => Some(*x as *const Ruleset<'a> as usize),
            _ => None,
          }));
          splice(ruleset, i, &filtered);
          i = (i + n).wrapping_sub(1);
          self.spliced_calls(ruleset, &filtered);
        }
        Node::VariableCall(vc) => {
          let rs = self.eval_variable_call_rules(vc)?;
          let filtered: Vec<Node<'a>> = rs
            .into_iter()
            .filter(|r| !matches!(r, Node::Declaration(d) if d.variable))
            .collect();
          let n = filtered.len();
          fresh.extend(filtered.iter().filter_map(|r| match r {
            Node::Ruleset(x) => Some(*x as *const Ruleset<'a> as usize),
            _ => None,
          }));
          splice(ruleset, i, &filtered);
          i = (i + n).wrapping_sub(1);
          self.spliced_calls(ruleset, &filtered);
        }
        _ => {}
      }
      i = i.wrapping_add(1);
    }
    // everything else
    let mut i = 0;
    while let Some(rule) = ruleset.rule_at(i) {
      if let Node::Ruleset(x) = rule {
        // `rule.eval(context)` of a ruleset just returned by a mixin
        // call whose evaluation is a fixed point: the copy less.js
        // makes is structurally identical and the original becomes
        // garbage, so keep the original (recursive mixins would
        // otherwise copy their whole output at every level)
        if !fresh.is_empty()
          && fresh.contains(&(x as *const Ruleset<'a> as usize))
          && self.is_fixed_point(x)
        {
          if x.selectors.get().is_some_and(|s| !s.is_empty()) {
            // what evaluating its selectors leaves behind
            self.s.default_state.set(DefaultState::Unset);
          }
          i += 1;
          continue;
        }
      }
      if !matches!(rule, Node::MixinDefinition(_) | Node::DetachedRuleset(_)) {
        let v = self.eval(rule)?;
        ruleset.set_rule(i, v);
        self.find_index_replaced(ruleset, i, v);
      }
      i += 1;
    }
    // fold `& { }` rulesets into the parent
    let mut i: isize = 0;
    loop {
      let rule = match ruleset.rule_at(i as usize) {
        Some(r) => r,
        None => break,
      };
      if let Node::Ruleset(r) = rule
        && let Some(sels) = r.selectors.get()
        && sels.len() == 1
        && sels[0].is_just_parent_selector()
      {
        remove_at(ruleset, i as usize);
        i -= 1;
        let info = r.vis.info();
        for sub in r.rules_vec() {
          // changed in place: shared declarations get copies
          let sub = if info == VisInfo::default() {
            sub
          } else {
            self.unshared(sub)
          };
          sub.copy_visibility_info(info);
          let is_var = matches!(sub, Node::Declaration(d) if d.variable);
          if !is_var {
            i += 1;
            insert_at(ruleset, i as usize, sub);
          }
        }
      }
      i += 1;
    }
    // bubble media blocks created inside
    if let Some(mb) = self.media_blocks.clone() {
      for m in mb.iter().skip(media_block_count) {
        self.bubble_selectors(m, selectors);
      }
    }
    Ok(())
  }

  fn parse_selectors(
    &mut self,
    text: &'a str,
    start: Pos,
  ) -> LResult<Option<Vec<&'a Selector<'a>>>> {
    let (file, idx) = match self.s.locate(start) {
      Some(x) => x,
      None => match self
        .s
        .file_id(self.s.opts.filename.as_deref().unwrap_or("input"))
      {
        Some(f) => (f, 0),
        None => return Ok(None),
      },
    };
    let mut p = Parser::new(self.s, file, text, idx);
    let r = p.parse_node(&[ParseFn::Selectors]).map_err(|mut e| {
      e.typ = "Syntax".to_string();
      e
    })?;
    match r {
      Some(mut v) => match v.pop() {
        Some(Parsed::Selectors(Some(s))) => Ok(Some(s.into_vec())),
        _ => Ok(Some(Vec::new())),
      },
      None => Ok(None),
    }
  }

  pub(crate) fn eval_imports(&mut self, ruleset: &'a Ruleset<'a>) -> LResult<()> {
    let mut i = 0;
    while let Some(rule) = ruleset.rule_at(i) {
      if let Node::Import(imp) = rule {
        let r = self.eval_import(imp)?;
        if r.is_list {
          let n = r.nodes.len();
          splice(ruleset, i, &r.nodes);
          i = (i + n).wrapping_sub(1);
        } else {
          splice(ruleset, i, &r.nodes);
        }
        ruleset.reset_cache();
      }
      i = i.wrapping_add(1);
    }
    Ok(())
  }

  // ------------------------------------------------------------------
  // scope lookups
  // ------------------------------------------------------------------

  fn build_variables(&mut self, rs: &'a Ruleset<'a>) -> LResult<()> {
    if rs.no_variables.get()
      || rs
        .cache
        .get()
        .is_some_and(|c| c.variables.borrow().is_some())
    {
      return Ok(());
    }
    // sized for the variable declarations (imports' variables grow it)
    let n = rs.rules.borrow().as_deref().map_or(0, |rules| {
      rules
        .iter()
        .filter(|r| matches!(r, Node::Declaration(d) if d.variable) || matches!(r, Node::Import(_)))
        .count()
    });
    let mut map = VarsBuilder {
      small: smallvec::SmallVec::new(),
      big: None,
      capacity: n,
    };
    let mut ri = 0;
    while let Some(r) = rs.rule_at(ri) {
      ri += 1;
      match r {
        Node::Declaration(d) if d.variable => {
          if let Some(n) = d.name_str() {
            map.insert(self.a, n, d);
          }
        }
        Node::Import(imp) => {
          if let Some(ImportRoot::Tree(root)) = imp.root.get() {
            self.build_variables(root)?;
            let names: Vec<&'a str> = root
              .cache
              .get()
              .and_then(|c| c.variables.borrow().as_ref().map(|m| m.names()))
              .unwrap_or_default();
            for name in names {
              if let Some(d) = self.frame_variable(root, name)? {
                map.insert(self.a, name, d);
              }
            }
          }
        }
        _ => {}
      }
    }
    if map.is_empty() && rs.cache.get().is_none() {
      rs.no_variables.set(true);
    } else {
      let vars = map.finish(self.a);
      *rs.cache(self.a).variables.borrow_mut() = Some(vars);
    }
    Ok(())
  }

  /// `ruleset.variable(name)`
  pub(crate) fn frame_variable(
    &mut self,
    rs: &'a Ruleset<'a>,
    name: &str,
  ) -> LResult<Option<&'a Declaration<'a>>> {
    self.build_variables(rs)?;
    let d = rs
      .cache
      .get()
      .and_then(|c| c.variables.borrow().as_ref().and_then(|m| m.get(name)));
    match d {
      Some(d) => Ok(Some(self.parse_value(rs, d)?)),
      None => Ok(None),
    }
  }

  fn build_properties(&mut self, rs: &'a Ruleset<'a>) {
    if rs.cache(self.a).properties.get().is_some() {
      return;
    }
    let mut map: PropMap<'a> = PropMap::with_hasher_in(Default::default(), self.a.bump);
    let mut ri = 0;
    while let Some(r) = rs.rule_at(ri) {
      ri += 1;
      if let Node::Declaration(d) = r
        && !d.variable
      {
        let name = match d.name {
          DeclName::Str(s) => s,
          DeclName::Parts(p) => {
            if p.len() == 1 {
              if let Node::Keyword(k) = p[0] {
                k.value
              } else {
                "[object Object]"
              }
            } else {
              "[object Object]"
            }
          }
        };
        let key = self.str(&format!("${}", name));
        map
          .entry(key)
          .or_insert_with(|| BVec::new_in(self.a.bump))
          .push(d);
      }
    }
    rs.cache(self.a)
      .properties
      .set(Some(self.a.alloc(std::cell::RefCell::new(map))));
  }

  /// `ruleset.property(name)`
  pub(crate) fn frame_property(
    &mut self,
    rs: &'a Ruleset<'a>,
    name: &str,
  ) -> LResult<Option<Vec<&'a Declaration<'a>>>> {
    self.build_properties(rs);
    let decls: Option<Vec<&'a Declaration<'a>>> = rs
      .cache(self.a)
      .properties
      .get()
      .and_then(|m| m.borrow().get(name).map(|v| v.to_vec()));
    match decls {
      Some(list) => {
        let mut out = Vec::with_capacity(list.len());
        for d in list {
          out.push(self.parse_value(rs, d)?);
        }
        Ok(Some(out))
      }
      None => Ok(None),
    }
  }

  pub(crate) fn last_declaration(
    &mut self,
    rs: &'a Ruleset<'a>,
  ) -> LResult<Option<&'a Declaration<'a>>> {
    let rules = rs.rules_vec();
    for r in rules.iter().rev() {
      if let Node::Declaration(d) = r {
        return Ok(Some(self.parse_value(rs, d)?));
      }
    }
    Ok(None)
  }

  /// `Ruleset.parseValue`: lazily parses anonymous declaration values.
  /// `parseValue` of a declaration found in `rs`.
  pub(crate) fn parse_value(
    &mut self,
    rs: &'a Ruleset<'a>,
    d: &'a Declaration<'a>,
  ) -> LResult<&'a Declaration<'a>> {
    if let Node::Anonymous(an) = d.value.get()
      && !an.prevent_reparse
      && !d.parsed()
    {
      if an.is_null {
        d.set_parsed();
        return Ok(d);
      }
      // less.js reparses its own copy at this place only. Variables
      // are not output, and their lazily parsed values (plain tokens,
      // see `anonymousValue`) evaluate alike parsed or not, so a
      // shared variable is reparsed in place.
      let d = if d.shared() && !d.variable {
        self.unshare_in(rs, d)
      } else {
        d
      };
      let (file, idx) = match self.s.locate(an.pos).or_else(|| self.s.locate(d.pos)) {
        Some(x) => x,
        None => {
          let f = self
            .s
            .file_id(self.s.opts.filename.as_deref().unwrap_or("input"))
            .unwrap_or(0);
          (f, 0)
        }
      };
      let idx = if an.pos.is_none() { 0 } else { idx };
      let mut p = Parser::new(self.s, file, an.value, idx);
      let r = p
        .parse_node(&[ParseFn::Value, ParseFn::Important])
        .map_err(|mut e| {
          e.typ = "Syntax".to_string();
          e
        })?;
      match r {
        None => d.set_parsed(),
        Some(results) => {
          let mut it = results.into_iter();
          let value = match it.next() {
            Some(Parsed::Node(v)) => v,
            _ => None,
          };
          let important = match it.next() {
            Some(Parsed::Important(i)) => i.unwrap_or(""),
            _ => "",
          };
          match value {
            Some(v) => d.value.set(v),
            None => d.value.set(self.a.null_anonymous()),
          }
          d.set_important(important);
          d.set_parsed();
        }
      }
      return Ok(d);
    }
    Ok(d)
  }

  /// A node that may be changed in place: shared declarations are copied.
  pub(crate) fn unshared(&self, n: Node<'a>) -> Node<'a> {
    match n {
      Node::Declaration(d) if d.shared() => Node::Declaration(self.a.declaration_copy(d)),
      n => n,
    }
  }

  /// `ruleset.resetCache()` after a call's results replaced the call: the
  /// caches are updated instead where that is equivalent. The results'
  /// variables have names the scope does not define (they are filtered
  /// out otherwise), so they extend the variables map; mixin lookups only
  /// change when rulesets came in.
  fn spliced_calls(&self, rs: &'a Ruleset<'a>, items: &[Node<'a>]) {
    let Some(c) = rs.cache.get() else {
      rs.no_variables.set(false);
      return;
    };
    if items.iter().any(|n| matches!(n, Node::Import(_))) {
      rs.reset_cache();
      return;
    }
    c.properties.set(None);
    if items.iter().any(|n| n.as_ruleset().is_some())
      && let Some(l) = c.lookups.get()
    {
      l.borrow_mut().clear();
    }
    let mut vars = items.iter().filter_map(|n| match n {
      Node::Declaration(d) if d.variable => d.name_str().map(|name| (name, *d)),
      _ => None,
    });
    let Some(first) = vars.next() else { return };
    let mut variables = c.variables.borrow_mut();
    match &mut *variables {
      Some(Vars::Map(m)) => {
        for (name, d) in std::iter::once(first).chain(vars) {
          m.insert(name, d);
        }
      }
      Some(Vars::Small(s)) => {
        let mut v = s.to_vec();
        for (name, d) in std::iter::once(first).chain(vars) {
          match v.iter_mut().find(|e| e.0 == name) {
            Some(e) => e.1 = d,
            None => v.push((name, d)),
          }
        }
        *s = self.a.slice(&v);
      }
      None => rs.no_variables.set(false),
    }
  }

  /// Replaces a shared declaration of `rs` with a copy of its own.
  fn unshare_in(&self, rs: &'a Ruleset<'a>, d: &'a Declaration<'a>) -> &'a Declaration<'a> {
    let copy = self.a.declaration_copy(d);
    let i = rs.rules.borrow().as_ref().and_then(|v| {
      v.iter()
        .position(|n| matches!(n, Node::Declaration(x) if std::ptr::eq(*x, d)))
    });
    if let Some(i) = i {
      rs.set_rule(i, Node::Declaration(copy));
      // the scope's maps now name the copy (a decl is no mixin: lookups stay)
      if let Some(c) = rs.cache.get() {
        let swap = |x: &mut &'a Declaration<'a>| {
          if std::ptr::eq(*x, d) {
            *x = copy;
          }
        };
        match &mut *c.variables.borrow_mut() {
          Some(Vars::Map(m)) => m.values_mut().for_each(swap),
          Some(Vars::Small(s)) if s.iter().any(|e| std::ptr::eq(e.1, d)) => {
            let mut v = s.to_vec();
            v.iter_mut().for_each(|e| swap(&mut e.1));
            *s = self.a.slice(&v);
          }
          _ => {}
        }
        if let Some(p) = c.properties.get() {
          p.borrow_mut()
            .values_mut()
            .for_each(|l| l.iter_mut().for_each(swap));
        }
      }
    }
    copy
  }

  /// `Ruleset.find(selector, self, filter)`
  pub(crate) fn find(
    &mut self,
    rs: &'a Ruleset<'a>,
    selector: &'a Selector<'a>,
    self_rs: Option<&'a Ruleset<'a>>,
    filter: bool,
  ) -> LResult<&'a [FoundMixin<'a>]> {
    let key = selector_key(selector)?;
    self.find_keyed(rs, selector, &key, self_rs, filter)
  }

  pub(crate) fn find_keyed(
    &mut self,
    rs: &'a Ruleset<'a>,
    selector: &'a Selector<'a>,
    key: &str,
    self_rs: Option<&'a Ruleset<'a>>,
    filter: bool,
  ) -> LResult<&'a [FoundMixin<'a>]> {
    let self_rs = self_rs.unwrap_or(rs);
    if let Some(v) = rs
      .cache(self.a)
      .lookups
      .get()
      .and_then(|l| l.borrow().get(key).copied())
    {
      return Ok(v);
    }
    let mut rules: Vec<FoundMixin<'a>> = Vec::new();
    // No evaluation happens below (matchArgs(null) never evaluates), so
    // the rules can stay borrowed.
    let rules_ref = rs.rules.borrow();
    let all_rules: &[Node<'a>] = rules_ref.as_deref().unwrap_or(&[]);
    let candidates = self.find_candidates(rs, all_rules, selector);
    let n = candidates.as_ref().map_or(all_rules.len(), |c| c.len());
    for k in 0..n {
      let rule = match &candidates {
        Some(c) => all_rules[c[k] as usize],
        None => all_rules[k],
      };
      let r = match rule.as_ruleset() {
        Some(r) => r,
        None => continue,
      };
      if std::ptr::eq(r, self_rs) {
        continue;
      }
      for sel in r.selectors() {
        let mt = selector_match(self.a, selector, sel);
        if mt > 0 {
          if selector.elements.len() > mt {
            if !filter || self.match_args(rule, None)? {
              let sub = self.a.selector(
                &selector.elements[mt..],
                None,
                None,
                Pos::NONE,
                VisInfo::default(),
              );
              let found = self.find(r, sub, Some(self_rs), filter)?;
              for f in found {
                let mut path: Vec<Node<'a>> = f.path.to_vec();
                path.push(rule);
                rules.push(FoundMixin {
                  rule: f.rule,
                  path: self.a.slice(&path),
                });
              }
            }
          } else {
            rules.push(FoundMixin { rule, path: &[] });
          }
          break;
        }
      }
    }
    drop(rules_ref);
    let rules = self.a.slice(&rules);
    let key = self.str(key);
    let cache = rs.cache(self.a);
    let lookups = match cache.lookups.get() {
      Some(l) => l,
      None => {
        let l = self
          .a
          .alloc(std::cell::RefCell::new(LookupMap::with_hasher_in(
            Default::default(),
            self.a.bump,
          )));
        cache.lookups.set(Some(l));
        l
      }
    };
    lookups.borrow_mut().insert(key, rules);
    Ok(rules)
  }

  /// The first mixin tokens of a rule's selectors (`None`: not indexable,
  /// always a candidate).
  fn rule_keys(&self, rule: Node<'a>) -> Option<smallvec::SmallVec<[&'a str; 2]>> {
    let mut keys = smallvec::SmallVec::new();
    let r = match rule.as_ruleset() {
      Some(r) => r,
      None => return Some(keys),
    };
    for sel in r.selectors() {
      if sel.list().is_some()
        || sel
          .elements
          .iter()
          .any(|e| e.is_variable || !matches!(e.value, ElValue::Str(_)))
      {
        return None;
      }
      if let Some(t) = mixin_tokens(self.a, sel).first()
        && !keys.contains(&t)
      {
        keys.push(t);
      }
    }
    Some(keys)
  }

  fn index_rule(&self, ix: &FindIndex<'a>, i: u32, rule: Node<'a>) {
    match self.rule_keys(rule) {
      None => ix.always.borrow_mut().push(i),
      Some(keys) => {
        let mut by_first = ix.by_first.borrow_mut();
        for k in keys {
          by_first
            .entry(k)
            .or_insert_with(|| BVec::new_in(self.a.bump))
            .push(i);
        }
      }
    }
  }

  /// Keeps a scope's index current when a rule is replaced in place.
  fn find_index_replaced(&self, rs: &'a Ruleset<'a>, i: usize, rule: Node<'a>) {
    if let Some(ix) = rs.cache.get().and_then(|c| c.find_index.get())
      && ix.generation.get() == rs.generation()
    {
      self.index_rule(ix, i as u32, rule);
    }
  }

  /// The indices (ascending) of the rules of a large scope that can match
  /// `selector`, or `None` to scan them all. The index is built once a
  /// scope keeps its shape across lookups.
  fn find_candidates(
    &self,
    rs: &'a Ruleset<'a>,
    all_rules: &[Node<'a>],
    selector: &'a Selector<'a>,
  ) -> Option<smallvec::SmallVec<[u32; 16]>> {
    if all_rules.len() < 64 {
      return None;
    }
    let cache = rs.cache(self.a);
    let generation = rs.generation();
    let ix = match cache.find_index.get() {
      Some(ix) if ix.generation.get() == generation => ix,
      old => {
        let (g, misses) = cache.find_misses.get();
        let misses = if g == generation { misses + 1 } else { 1 };
        cache.find_misses.set((generation, misses));
        if misses < 2 {
          return None;
        }
        // rebuilt in place: the lists keep their capacity
        let ix = match old {
          Some(ix) => {
            for list in ix.by_first.borrow_mut().values_mut() {
              list.clear();
            }
            ix.always.borrow_mut().clear();
            ix.generation.set(generation);
            ix
          }
          None => self.a.alloc(FindIndex {
            generation: std::cell::Cell::new(generation),
            by_first: std::cell::RefCell::new(hashbrown::HashMap::with_hasher_in(
              Default::default(),
              self.a.bump,
            )),
            always: std::cell::RefCell::new(BVec::new_in(self.a.bump)),
          }),
        };
        // staged on the heap so the arena lists get their exact size
        let mut staged: rustc_hash::FxHashMap<&'a str, smallvec::SmallVec<[u32; 4]>> =
          Default::default();
        {
          let mut always = ix.always.borrow_mut();
          for (i, rule) in all_rules.iter().enumerate() {
            match self.rule_keys(*rule) {
              None => always.push(i as u32),
              Some(keys) => {
                for k in keys {
                  staged.entry(k).or_default().push(i as u32);
                }
              }
            }
          }
        }
        {
          let mut by_first = ix.by_first.borrow_mut();
          by_first.reserve(staged.len());
          for (k, list) in staged {
            let slot = by_first
              .entry(k)
              .or_insert_with(|| BVec::with_capacity_in(list.len(), self.a.bump));
            slot.extend_from_slice(&list);
          }
        }
        cache.find_index.set(Some(ix));
        ix
      }
    };
    let mut out: smallvec::SmallVec<[u32; 16]> = smallvec::SmallVec::new();
    if let Some(first) = selector.elements.first().and_then(|e| e.value.as_str())
      && let Some(list) = ix.by_first.borrow().get(first)
    {
      out.extend_from_slice(list);
    }
    out.extend_from_slice(&ix.always.borrow());
    out.sort_unstable();
    out.dedup();
    Some(out)
  }

  /// Whether `Ruleset.eval` of this (evaluated) ruleset yields a
  /// structurally identical ruleset without side effects: plain selectors,
  /// default visibility, and only literal declarations, comments and such
  /// rulesets inside.
  pub(crate) fn is_fixed_point(&self, r: &'a Ruleset<'a>) -> bool {
    match r.fixed.get() {
      1 => return true,
      2 => return false,
      _ => {}
    }
    let ok = self.compute_fixed_point(r);
    r.fixed.set(if ok { 1 } else { 2 });
    ok
  }

  fn compute_fixed_point(&self, r: &'a Ruleset<'a>) -> bool {
    let default_vis = |v: VisInfo| v == VisInfo::default();
    if r.mixin.is_some()
      || !default_vis(r.vis.info())
      || r.multi_media.get()
      || r.extend_on_every_path.get()
    {
      return false;
    }
    if !r
      .original()
      .is_some_and(|o| default_vis(o.visibility_info()))
    {
      return false;
    }
    for sel in r.selectors() {
      if sel.has_aux()
        || sel.extend_list().is_some_and(|l| !l.is_empty())
        || !sel.evald_condition.get()
        || sel.media_empty.get()
        || sel
          .elements
          .iter()
          .any(|e| e.is_variable || !matches!(e.value, ElValue::Str(_)))
      {
        return false;
      }
    }
    let rules = r.rules.borrow();
    for n in rules.iter().flat_map(|v| v.iter()) {
      let ok = match *n {
        Node::Comment(_) => true,
        Node::Ruleset(c) => self.is_fixed_point(c),
        Node::Declaration(d) => {
          d.name_str().is_some()
            && default_vis(d.vis.info())
            && important_str(self.a, d.important()) == d.important()
            && self.fixed_value(d.value.get())
        }
        _ => false,
      };
      if !ok {
        return false;
      }
    }
    true
  }

  /// A value whose evaluation returns itself or an identical copy.
  fn fixed_value(&self, n: Node<'a>) -> bool {
    let default_vis = |v: VisInfo| v == VisInfo::default();
    match n {
      Node::Dimension(_) | Node::Color(_) | Node::Keyword(_) | Node::UnicodeDescriptor(_) => true,
      Node::Quoted(q) => {
        !q.permissive
          && default_vis(q.vis.info())
          && !q.value.bytes().any(|b| b == b'@' || b == b'$')
      }
      Node::Anonymous(a) => default_vis(a.vis.info()) && a.no_spacing.get().is_none(),
      Node::Expression(e) => {
        e.value.len() > 1
          && !e.parens
          && !e.parens_in_op.get()
          && default_vis(e.vis.info())
          && e.value.iter().all(|v| self.fixed_value(*v))
      }
      Node::Value(v) => {
        v.value.len() > 1
          && default_vis(v.vis.info())
          && v.value.iter().all(|x| self.fixed_value(*x))
      }
      // a plain CSS function call (not a Less function)
      Node::Call(c) => {
        default_vis(c.vis.info())
          && self
            .s
            .functions
            .get(&c.name.cow_to_ascii_lowercase())
            .is_none()
          && c.args.iter().all(|x| self.fixed_value(*x))
      }
      Node::Url(u) => u.is_evald && self.fixed_value(u.value),
      Node::Paren(p) => p.no_spacing.get() != Some(false) && self.fixed_value(p.value),
      _ => false,
    }
  }

  /// `new Ruleset(null, rules)`, reusing a dead container when possible.
  pub(crate) fn pooled_ruleset(&self, rules: Option<&[Node<'a>]>) -> &'a Ruleset<'a> {
    let pooled = self.s.ruleset_pool.borrow_mut().pop();
    let Some(r) = pooled else {
      return self.a.ruleset(None, rules.map(|s| self.a.vec_from(s)));
    };
    // the fields that are not `Cell`s are the same for every such ruleset
    r.vis.copy_from(VisInfo::default());
    r.selectors.set(None);
    {
      let mut b = r.rules.borrow_mut();
      match rules {
        Some(src) => match b.as_mut() {
          Some(v) => {
            v.clear();
            v.extend_from_slice(src);
          }
          None => *b = Some(self.a.vec_from(src)),
        },
        None => *b = None,
      }
    }
    r.root.set(false);
    r.first_root.set(false);
    r.allow_imports.set(false);
    r.multi_media.set(false);
    r.extend_on_every_path.set(false);
    r.fixed.set(0);
    r.original_ruleset.set(None);
    r.set_debug_info(self.a, None);
    r.paths.set(None);
    r.bump_generation();
    r.reset_cache();
    if let Some(c) = r.cache.get() {
      // a stale generation; the index is rebuilt in place when needed
      if let Some(ix) = c.find_index.get() {
        ix.generation.set(r.generation().wrapping_sub(1));
      }
      c.find_misses.set((u32::MAX, 0));
    }
    r
  }

  /// Returns a dead `new Ruleset(null, …)` container to the pool.
  pub(crate) fn recycle_ruleset(&self, r: &'a Ruleset<'a>) {
    let is_placeholder = self
      .s
      .plain_original
      .get()
      .is_some_and(|p| std::ptr::eq(*p, r));
    if r.mixin.is_none() && !r.strict_imports && r.pos.is_none() && !is_placeholder {
      self.s.ruleset_pool.borrow_mut().push(r);
    }
  }

  /// `Ruleset.makeImportant` / `MixinDefinition.makeImportant`
  pub(crate) fn make_important(&self, n: Node<'a>) -> Node<'a> {
    match n {
      Node::Declaration(d) => {
        let variable = matches!(d.name, DeclName::Str(s) if s.starts_with('@'));
        Node::Declaration(self.a.declaration_ref(
          d.name,
          d.value.get(),
          " !important",
          d.merge.get(),
          d.pos,
          d.inline,
          Some(variable),
        ))
      }
      Node::Ruleset(r) => Node::Ruleset(self.make_ruleset_important(r)),
      Node::MixinDefinition(r) => {
        let def = r.mixin.expect("mixin definition has metadata");
        let rules: Vec<Node<'a>> = r
          .rules_vec()
          .into_iter()
          .map(|x| self.make_important(x))
          .collect();
        crate::syntax::parser::make_mixin_definition(
          self.a,
          Some(def.name),
          def.params,
          self.a.vec_from(&rules),
          def.condition,
          def.variadic,
          def.frames,
          VisInfo::default(),
        )
      }
      other => other,
    }
  }

  pub(crate) fn make_ruleset_important(&self, r: &'a Ruleset<'a>) -> &'a Ruleset<'a> {
    let rules: Vec<Node<'a>> = r
      .rules_vec()
      .into_iter()
      .map(|x| self.make_important(x))
      .collect();
    self.a.ruleset_full(
      r.selectors.get(),
      Some(self.a.vec_from(&rules)),
      r.strict_imports,
      r.vis.info(),
      None,
      r.pos,
    )
  }
}

fn splice<'a>(rs: &'a Ruleset<'a>, i: usize, items: &[Node<'a>]) {
  rs.bump_generation();
  let mut b = rs.rules.borrow_mut();
  if let Some(v) = b.as_mut() {
    v.splice(i..i + 1, items.iter().copied());
  }
}

fn remove_at<'a>(rs: &'a Ruleset<'a>, i: usize) {
  rs.bump_generation();
  if let Some(v) = rs.rules.borrow_mut().as_mut() {
    v.remove(i);
  }
}

fn insert_at<'a>(rs: &'a Ruleset<'a>, i: usize, n: Node<'a>) {
  rs.bump_generation();
  if let Some(v) = rs.rules.borrow_mut().as_mut() {
    v.insert(i.min(v.len()), n);
  }
}

/// A cache key equivalent to `selector.toCSS({})` (equal keys ⇔ equal CSS).
pub(crate) fn selector_key(s: &Selector<'_>) -> LResult<String> {
  let mut k = String::new();
  for e in s.elements {
    k.push_str(e.combinator.get().value());
    k.push('\u{1}');
    match e.value {
      ElValue::Str(v) => k.push_str(v),
      ElValue::Node(_) => k.push_str(&crate::css::element_to_css(
        e,
        &mut crate::css::CssCtx::empty(),
      )?),
    }
    k.push('\u{2}');
  }
  Ok(k)
}

/// `Selector.match(other)`: number of matched elements.
pub(crate) fn selector_match<'a>(
  a: Alloc<'a>,
  this: &'a Selector<'a>,
  other: &'a Selector<'a>,
) -> usize {
  let tokens = mixin_tokens(a, other);
  let olen = tokens.len();
  if olen == 0 || this.elements.len() < olen {
    return 0;
  }
  for i in 0..olen {
    match this.elements[i].value {
      ElValue::Str(s) if s == tokens.get(i) => {}
      _ => return 0,
    }
  }
  olen
}

/// A selector's mixin tokens: its element strings when every element is a
/// single token (no array needed), else the tokenized text.
#[derive(Copy, Clone)]
pub(crate) enum Tokens<'a> {
  Simple(&'a [&'a Element<'a>]),
  Complex(&'a [&'a str]),
}

impl<'a> Tokens<'a> {
  pub fn len(&self) -> usize {
    match self {
      Tokens::Simple(e) => e.len(),
      Tokens::Complex(t) => t.len(),
    }
  }
  pub fn get(&self, i: usize) -> &'a str {
    match self {
      Tokens::Simple(e) => e[i].value.as_str().unwrap_or(""),
      Tokens::Complex(t) => t[i],
    }
  }
  pub fn first(&self) -> Option<&'a str> {
    (self.len() > 0).then(|| self.get(0))
  }
}

/// Marks a simple selector in `Selector::mixin_elements`.
static SIMPLE_TOKENS: [&str; 1] = ["\u{0}"];
static SIMPLE_REF: &[&str] = &SIMPLE_TOKENS;

pub(crate) fn mixin_tokens<'a>(a: Alloc<'a>, s: &'a Selector<'a>) -> Tokens<'a> {
  let t = mixin_elements(a, s);
  if std::ptr::eq(t.as_ptr(), SIMPLE_TOKENS.as_ptr()) {
    let skip = matches!(s.elements.first().map(|e| e.value), Some(ElValue::Str("&"))) as usize;
    Tokens::Simple(&s.elements[skip..])
  } else {
    Tokens::Complex(t)
  }
}

/// `Selector.mixinElements()`
pub(crate) fn mixin_elements<'a>(a: Alloc<'a>, s: &'a Selector<'a>) -> &'a [&'a str] {
  mixin_elements_ref(a, s)
}

pub(crate) fn mixin_elements_ref<'a>(a: Alloc<'a>, s: &'a Selector<'a>) -> &'a [&'a str] {
  if let Some(t) = s.mixin_elements_cached() {
    return t;
  }
  // Fast path (not cached: the check is cheap and most selectors never
  // need the out-of-line cache): every element is a single token (`.name`,
  // `#name`, `name`, `&`) joined by combinators that contain no token chars.
  {
    let simple = s.elements.iter().all(|e| {
      let c = e.combinator.get().value();
      let comb_ok = c
        .bytes()
        .all(|b| matches!(b, b' ' | b'>' | b'+' | b'~' | b'|' | b'^' | b'/'));
      comb_ok
        && match e.value {
          ElValue::Str(v) => {
            let b = v.as_bytes();
            !b.is_empty()
              && (v == "&"
                || (matches!(b[0], b'.' | b'#' | b'*' | b',')
                  || b[0].is_ascii_alphanumeric()
                  || b[0] == b'_'
                  || b[0] == b'-')
                  && b[1..]
                    .iter()
                    .all(|c| c.is_ascii_alphanumeric() || *c == b'_' || *c == b'-'))
          }
          _ => false,
        }
    });
    if simple {
      // the element strings are the tokens (see `mixin_tokens`)
      return SIMPLE_REF;
    }
  }
  s.mixin_elements_cache(a).get_or_init(|| {
    let mut joined = bumpalo::collections::String::new_in(a.bump);
    for e in s.elements {
      joined.push_str(e.combinator.get().value());
      match e.value {
        ElValue::Str(v) => joined.push_str(v),
        ElValue::Node(n) => match n {
          Node::Keyword(k) => joined.push_str(k.value),
          Node::Anonymous(a) => joined.push_str(a.value),
          Node::Quoted(q) => joined.push_str(q.value),
          Node::Attribute(_) | Node::Paren(_) | Node::Selector(_) => {
            joined.push_str("[object Object]")
          }
          _ => joined.push_str("[object Object]"),
        },
      }
    }
    let joined: &'a str = joined.into_bump_str();
    let mut v: smallvec::SmallVec<[&'a str; 8]> = tokenize_mixin_elements(joined);
    if v.first() == Some(&"&") {
      v.remove(0);
    }
    a.slice(&v)
  })
}

/// `/[,&#*.\w-]([\w-]|(\\.))*/g`
fn tokenize_mixin_elements(s: &str) -> smallvec::SmallVec<[&str; 8]> {
  let b = s.as_bytes();
  let mut out = smallvec::SmallVec::new();
  let mut i = 0;
  let is_w = |c: u8| c.is_ascii_alphanumeric() || c == b'_' || c == b'-';
  while i < b.len() {
    let c = b[i];
    if c == b',' || c == b'&' || c == b'#' || c == b'*' || c == b'.' || is_w(c) {
      let start = i;
      i += 1;
      loop {
        if i < b.len() && is_w(b[i]) {
          i += 1;
        } else if i + 1 < b.len() && b[i] == b'\\' && b[i + 1] != b'\n' && b[i + 1] != b'\r' {
          i += 2;
          while i < b.len() && !s.is_char_boundary(i) {
            i += 1;
          }
        } else {
          break;
        }
      }
      out.push(&s[start..i]);
    } else {
      i += 1;
      while i < b.len() && !s.is_char_boundary(i) {
        i += 1;
      }
    }
  }
  out
}

/// `mergeRules(rules)` from `tree/merge-rules.js`.
pub(crate) fn merge_rules<'a>(a: Alloc<'a>, rules: &mut Vec<Node<'a>>) {
  let mut groups: Vec<(&'a str, Vec<&'a Declaration<'a>>)> = Vec::new();
  let mut i = 0;
  while i < rules.len() {
    if let Node::Declaration(d) = rules[i]
      && d.merge.get() != Merge::None
    {
      let key = d.name_str().unwrap_or("");
      if let Some(g) = groups.iter_mut().find(|(k, _)| *k == key) {
        rules.remove(i);
        g.1.push(d);
        continue;
      } else {
        groups.push((key, vec![d]));
      }
    }
    i += 1;
  }
  for (_, group) in groups {
    if group.is_empty() {
      continue;
    }
    let result = group[0];
    let mut comma: Vec<Vec<Node<'a>>> = vec![Vec::new()];
    for rule in &group {
      if rule.merge.get() == Merge::Comma
        && !comma
          .last()
          .expect("merge groups start with one group")
          .is_empty()
      {
        comma.push(Vec::new());
      }
      comma
        .last_mut()
        .expect("merge groups start with one group")
        .push(rule.value.get());
      if result.important().is_empty() {
        result.important.set(rule.important.get());
      }
    }
    let exprs: Vec<Node<'a>> = comma
      .iter()
      .map(|space| Node::Expression(a.expression(space, None)))
      .collect();
    result.value.set(a.value(&exprs));
  }
}

/// Builds a [`Vars`] snapshot (the last declaration of a name wins).
struct VarsBuilder<'a> {
  small: smallvec::SmallVec<[(&'a str, &'a Declaration<'a>); 16]>,
  big: Option<VarMap<'a>>,
  capacity: usize,
}

impl<'a> VarsBuilder<'a> {
  fn insert(&mut self, a: Alloc<'a>, name: &'a str, d: &'a Declaration<'a>) {
    if let Some(m) = self.big.as_mut() {
      m.insert(name, d);
      return;
    }
    if let Some(e) = self.small.iter_mut().find(|e| e.0 == name) {
      e.1 = d;
    } else if self.small.len() < 16 {
      self.small.push((name, d));
    } else {
      // sized for the rules up front: large scopes are rebuilt after
      // every splice, and growing would rehash and strand old tables
      let mut m =
        VarMap::with_capacity_and_hasher_in(self.capacity.max(32), Default::default(), a.bump);
      m.extend(self.small.drain(..));
      m.insert(name, d);
      self.big = Some(m);
    }
  }
  fn is_empty(&self) -> bool {
    self.small.is_empty() && self.big.as_ref().is_none_or(|m| m.is_empty())
  }
  fn finish(self, a: Alloc<'a>) -> Vars<'a> {
    match self.big {
      Some(m) => Vars::Map(m),
      None => Vars::Small(a.slice(&self.small)),
    }
  }
}
