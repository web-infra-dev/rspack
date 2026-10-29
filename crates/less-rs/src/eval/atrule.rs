//! At-rules, media/container bubbling and imports (`atrule.js`,
//! `nested-at-rule.js`, `media.js`, `container.js`, `import.js`).

use std::cell::Cell;

use cow_utils::CowUtils;

use super::{EvalCtx, normalize_path, rewrite_path, with_string_value};
use crate::{
  error::{LResult, LessError},
  syntax::parser::{make_at_rule, make_import, make_media},
  tree::*,
};

pub(crate) struct ImportResult<'a> {
  pub nodes: std::borrow::Cow<'a, [Node<'a>]>,
  pub is_list: bool,
}

impl<'a> ImportResult<'a> {
  pub fn len(&self) -> usize {
    self.nodes.len()
  }
  fn list(nodes: Vec<Node<'a>>) -> Self {
    ImportResult {
      nodes: nodes.into(),
      is_list: true,
    }
  }
  fn one(n: Node<'a>) -> Self {
    ImportResult {
      nodes: vec![n].into(),
      is_list: false,
    }
  }
}

fn declarations_block(rules: &[Node<'_>], mergeable: bool) -> bool {
  rules.iter().all(|n| match n {
    Node::Declaration(d) => mergeable || d.merge.get() == Merge::None,
    Node::Comment(_) => true,
    _ => false,
  })
}

/// `startsWithMediaType(fragment)` (4.9.0).
fn starts_with_media_type(n: Node<'_>) -> bool {
  let head: Option<String> = match n {
    Node::Keyword(k) => Some(k.value.to_string()),
    Node::Anonymous(a) => Some(a.value.to_string()),
    Node::Expression(e) => {
      let parts: Vec<Node<'_>> = e.value.iter().copied().filter(|p| has_value(*p)).collect();
      let mut idx = 0;
      let first = parts
        .first()
        .map(|p| value_string(*p).cow_to_lowercase().into_owned());
      if first.as_deref() == Some("not") || first.as_deref() == Some("only") {
        idx += 1;
      }
      parts.get(idx).and_then(|p| match p {
        Node::Keyword(k) => Some(k.value.to_string()),
        Node::Anonymous(a) => Some(a.value.to_string()),
        Node::Quoted(q) => Some(q.value.to_string()),
        _ => None,
      })
    }
    _ => None,
  };
  let head = match head {
    Some(h) if !h.is_empty() => h,
    _ => return false,
  };
  let trimmed = head.trim();
  let first_token = trimmed
    .split(|c: char| c.is_whitespace() || c == '(')
    .next()
    .unwrap_or("");
  !first_token.is_empty() && !matches!(first_token.cow_to_lowercase().as_ref(), "and" | "or")
}

/// `p.value !== undefined`
fn has_value(n: Node<'_>) -> bool {
  !matches!(
    n,
    Node::Variable(_)
      | Node::Property(_)
      | Node::Call(_)
      | Node::Operation(_)
      | Node::Condition(_)
      | Node::MixinCall(_)
      | Node::VariableCall(_)
  )
}

fn value_string(n: Node<'_>) -> String {
  match n {
    Node::Keyword(k) => k.value.to_string(),
    Node::Anonymous(a) => a.value.to_string(),
    Node::Quoted(q) => q.value.to_string(),
    Node::Dimension(d) => crate::num::to_js_string(d.value),
    _ => "[object Object]".to_string(),
  }
}

impl<'a, 's> EvalCtx<'a, 's> {
  pub(crate) fn eval_at_rule(&mut self, at: &'a AtRule<'a>) -> LResult<Node<'a>> {
    let media_path_backup = self.media_path.replace(Vec::new());
    let media_blocks_backup = self.media_blocks.replace(Vec::new());
    let r = self.eval_at_rule_inner(at);
    self.media_path = media_path_backup;
    self.media_blocks = media_blocks_backup;
    r
  }

  fn eval_at_rule_inner(&mut self, at: &'a AtRule<'a>) -> LResult<Node<'a>> {
    let value = match at.value {
      Some(v) => Some(self.eval(v)?),
      None => None,
    };
    let mut rules: Option<Vec<Node<'a>>> =
      at.rules.get().or(at.declarations.get()).map(|r| r.to_vec());
    if let Some(r) = rules.take() {
      rules = Some(self.eval_root(at, r)?);
    }
    if let Some(r) = rules.as_ref()
      && let Some(first) = r.first().and_then(|n| n.as_ruleset())
    {
      let inner = first.rules_vec();
      if first.has_rules() && !inner.is_empty() {
        let all_mergeable = declarations_block(&inner, true);
        if all_mergeable && !at.is_rooted && value.is_none() {
          let mut inner = inner;
          super::merge_rules(self.a, &mut inner);
          for n in &inner {
            if let Node::Declaration(d) = n {
              d.merge.set(Merge::None);
            }
          }
          rules = Some(inner);
        }
      }
    }
    if at.simple_block
      && let Some(r) = rules.take()
    {
      let mut out = Vec::with_capacity(r.len());
      for n in r {
        out.push(self.eval(n)?);
      }
      rules = Some(out);
    }
    Ok(make_at_rule(
      self.a,
      at.name,
      value,
      rules,
      at.pos,
      at.debug_info,
      at.is_rooted,
      at.vis.info(),
    ))
  }

  /// `AtRule.evalRoot`
  fn eval_root(&mut self, at: &'a AtRule<'a>, rules: Vec<Node<'a>>) -> LResult<Vec<Node<'a>>> {
    let mut ampersand_count = 0;
    let mut no_ampersand_count = 0;
    let mut no_ampersands = true;
    let rules = if !at.simple_block {
      match rules.first() {
        Some(r) => vec![self.eval(*r)?],
        None => vec![],
      }
    } else {
      rules
    };
    let mut preceding: Vec<&'a Selector<'a>> = Vec::new();
    for fi in 0..self.frames.len() {
      let frame = self.frames[fi];
      if !frame.is_mixin()
        && frame.rules_len() > 0
        && !frame.root.get()
        && let Some(sels) = frame.selectors.get()
      {
        preceding.extend_from_slice(sels);
      }
      if !preceding.is_empty() {
        let all_amp = preceding.iter().all(|s| {
          !s.elements.is_empty()
            && s
              .elements
              .iter()
              .all(|e| matches!(e.value, ElValue::Str("&")))
        });
        if all_amp {
          no_ampersands = false;
          no_ampersand_count += 1;
        } else {
          ampersand_count += 1;
        }
      }
    }
    let mixed = ampersand_count > 0 && no_ampersand_count > 0 && !no_ampersands;
    if ((at.is_rooted && ampersand_count > 0 && no_ampersand_count == 0 && no_ampersands) || !mixed)
      && let Some(first) = rules.first()
      && let Some(rs) = first.as_ruleset()
    {
      rs.root.set(true);
    }
    Ok(rules)
  }

  pub(crate) fn eval_media(&mut self, at: &'a AtRule<'a>) -> LResult<Node<'a>> {
    if at.kind == AtRuleKind::Container && at.evaluated {
      return Ok(at.node());
    }
    if self.media_blocks.is_none() {
      self.media_blocks = Some(Vec::new());
      self.media_path = Some(Vec::new());
    }
    let body = at.body().expect("media body");
    if let Some(d) = at.debug_info {
      body.set_debug_info(self.a, Some(self.a.alloc(d)));
    }
    let features = match at.features.get() {
      Some(f) => Some(self.eval(f)?),
      None => None,
    };
    let media = self.a.alloc(AtRule {
      kind: at.kind,
      vis: Vis::new(at.vis.info()),
      name: at.name,
      value: None,
      features: Cell::new(features),
      rules: Cell::new(None),
      declarations: Cell::new(None),
      simple_block: false,
      is_rooted: false,
      debug_info: at.debug_info,
      pos: at.pos,
      evaluated: true,
    });
    self.media_path.get_or_insert_with(Vec::new).push(media);
    self.media_blocks.get_or_insert_with(Vec::new).push(media);
    self.frames.insert(0, body);
    let r = self.eval_ruleset(body);
    self.frames.remove(0);
    let evaluated = r?;
    media
      .rules
      .set(Some(self.a.slice(&[Node::Ruleset(evaluated)])));
    if let Some(p) = self.media_path.as_mut() {
      p.pop();
    }
    if self
      .media_path
      .as_ref()
      .map(|p| p.is_empty())
      .unwrap_or(true)
    {
      Ok(self.eval_top(media))
    } else {
      Ok(self.eval_nested(media))
    }
  }

  /// `evalFunction`: joins `keyword (paren)` pairs.
  fn media_eval_function(&self, media: &'a AtRule<'a>) {
    let features = match media.features.get() {
      Some(f) => f,
      None => return,
    };
    let values = match features.array_value() {
      Some(v) if !v.is_empty() => v,
      _ => return,
    };
    let mut out: Vec<Node<'a>> = values.to_vec();
    let mut changed = false;
    let mut index = 0;
    while index < out.len() {
      let expr = out[index];
      if matches!(expr, Node::Keyword(_) | Node::Variable(_))
        && index + 1 < out.len()
        && (expr.no_spacing() != Some(false))
      {
        let paren = out[index + 1];
        if matches!(paren, Node::Paren(p) if p.no_spacing.get() == Some(true)) {
          let e = self.a.expression(&[expr, paren], Some(true));
          out[index] = Node::Expression(e);
          out.remove(index + 1);
          changed = true;
        }
      }
      index += 1;
    }
    if changed {
      let nf = match features {
        Node::Value(_) => self.a.value(&out),
        Node::Expression(e) => {
          let ne = self.a.expression(&out, e.no_spacing.get());
          Node::Expression(ne)
        }
        other => other,
      };
      media.features.set(Some(nf));
    }
  }

  fn eval_top(&mut self, media: &'a AtRule<'a>) -> Node<'a> {
    self.media_eval_function(media);
    let mut result = media.node();
    let blocks = self.media_blocks.take().unwrap_or_default();
    if blocks.len() > 1 {
      let selectors = self.a.empty_selectors(media.pos);
      let rules: Vec<Node<'a>> = blocks.iter().map(|b| b.node()).collect();
      let rs = self
        .a
        .ruleset(Some(selectors), Some(self.a.vec_from(&rules)));
      rs.multi_media.set(true);
      rs.vis.copy_from(media.vis.info());
      result = Node::Ruleset(rs);
    }
    self.media_path = None;
    result
  }

  fn eval_nested(&mut self, media: &'a AtRule<'a>) -> Node<'a> {
    self.media_eval_function(media);
    let mut path: Vec<&'a AtRule<'a>> = self.media_path.clone().unwrap_or_default();
    path.push(media);
    let mut groups: Vec<Vec<Node<'a>>> = Vec::with_capacity(path.len());
    for p in &path {
      if p.kind != media.kind {
        if let Some(mb) = self.media_blocks.as_mut()
          && let Some(idx) = mb.iter().position(|m| std::ptr::eq(*m, media))
        {
          mb.remove(idx);
        }
        return media.node();
      }
      let f = p.features.get();
      let value: Vec<Node<'a>> = match f {
        Some(Node::Value(v)) => v.value.to_vec(),
        Some(other) => match other {
          Node::Rules(r) => r.to_vec(),
          n => vec![n],
        },
        None => vec![],
      };
      groups.push(value);
    }
    let permuted = permute(&groups);
    let mut exprs = Vec::with_capacity(permuted.len());
    for p in permuted {
      let mut p = p;
      let types: Vec<Node<'a>> = p
        .iter()
        .copied()
        .filter(|f| starts_with_media_type(*f))
        .collect();
      if !types.is_empty() && types.len() < p.len() {
        let mut np = types.clone();
        np.extend(p.iter().copied().filter(|f| !starts_with_media_type(*f)));
        p = np;
      }
      let mut i = p.len();
      while i > 1 {
        i -= 1;
        p.insert(i, self.a.anonymous("and", Pos::NONE));
      }
      exprs.push(Node::Expression(self.a.expression(&p, None)));
    }
    media.features.set(Some(self.a.value(&exprs)));
    Node::Ruleset(self.a.ruleset(Some(&[]), Some(self.a.vec())))
  }

  /// `bubbleSelectors(selectors)`
  pub(crate) fn bubble_selectors(
    &mut self,
    media: &'a AtRule<'a>,
    selectors: Option<&'a [&'a Selector<'a>]>,
  ) {
    let selectors = match selectors {
      Some(s) => s,
      None => return,
    };
    let first = match media.rules.get().and_then(|r| r.first()) {
      Some(f) => *f,
      None => return,
    };
    let mut v = self.a.vec();
    v.push(first);
    let rs = self.a.ruleset(Some(self.a.slice(selectors)), Some(v));
    media.rules.set(Some(self.a.slice(&[Node::Ruleset(rs)])));
  }

  // ------------------------------------------------------------------
  // imports
  // ------------------------------------------------------------------

  pub(crate) fn eval_import(&mut self, imp: &'a Import<'a>) -> LResult<ImportResult<'a>> {
    let mut result = self.import_do_eval(imp)?;
    if imp.options.reference || imp.vis.blocks_visibility() {
      self.s.vis_blocked.set(true);
      if result
        .nodes
        .iter()
        .any(|n| matches!(n, Node::Declaration(d) if d.shared()))
      {
        let nodes: Vec<Node<'a>> = result.nodes.iter().map(|n| self.unshared(*n)).collect();
        result.nodes = std::borrow::Cow::Owned(nodes);
      }
      for n in result.nodes.iter() {
        n.add_visibility_block();
      }
    }
    Ok(result)
  }

  fn is_layer_expr(n: Option<Node<'a>>) -> Option<&'a Expression<'a>> {
    if let Some(Node::Expression(e)) = n
      && e.value.len() >= 2
      && let (Node::Keyword(k), Node::Paren(_)) = (e.value[0], e.value[1])
      && k.value == "layer"
    {
      return Some(e);
    }
    None
  }

  /// Rewrites `layer(x) rest` into `[Expression(layer (x)), rest]` in the
  /// import's own features.
  fn rewrite_layer_features(&self, imp: &'a Import<'a>, expr: &'a Expression<'a>) {
    let first = self.a.expression(&expr.value[..2], Some(true));
    let mut items = vec![Node::Expression(first)];
    items.extend_from_slice(&expr.value[2..]);
    let ne = self.a.alloc(Expression {
      vis: Vis::default(),
      value: self.a.slice(&items),
      no_spacing: Cell::new(expr.no_spacing.get()),
      parens: expr.parens,
      parens_in_op: Cell::new(expr.parens_in_op.get()),
      pos: expr.pos,
    });
    let feats = imp.features();
    let nf = match feats {
      Some(Node::Value(v)) => {
        let mut vals = v.value.to_vec();
        if !vals.is_empty() {
          vals[0] = Node::Expression(ne);
        }
        self.a.value(&vals)
      }
      _ => Node::Expression(ne),
    };
    imp.features_override.set(Some(nf));
  }

  fn import_do_eval(&mut self, imp: &'a Import<'a>) -> LResult<ImportResult<'a>> {
    let features = match imp.features() {
      Some(f) => Some(self.eval(f)?),
      None => None,
    };
    if imp.options.is_plugin {
      return Ok(ImportResult::list(vec![]));
    }
    match imp.skip.get() {
      Skip::Yes => return Ok(ImportResult::list(vec![])),
      Skip::Once(id) => {
        let skip = !self.s.once_files.borrow_mut().insert(id);
        imp.skip.set(if skip { Skip::Yes } else { Skip::No });
        if skip {
          return Ok(ImportResult::list(vec![]));
        }
      }
      Skip::No => {}
    }
    let feature_values: Option<&'a [Node<'a>]> = imp.features().and_then(|f| match f {
      Node::Value(v) => Some(v.value),
      _ => None,
    });
    if let Some(fv) = feature_values
      && !fv.is_empty()
      && Self::is_layer_expr(Some(fv[0])).is_some()
    {
      imp.css.set(Some(false));
    }
    if imp.options.inline {
      let text = match imp.root.get() {
        Some(ImportRoot::Text(t)) => t,
        _ => "",
      };
      let pos = match imp.imported_file.get() {
        Some(fid) => Pos(self.s.file_base(fid)),
        None => Pos::NONE,
      };
      let an = self.a.alloc(Anonymous {
        vis: Vis::default(),
        value: text,
        pos,
        map_lines: true,
        ruleset_like: true,
        prevent_reparse: false,
        no_spacing: Cell::new(None),
        is_null: false,
        has_file: true,
      });
      let contents = Node::Anonymous(an);
      return Ok(match imp.features() {
        Some(f) => {
          let fv: Vec<Node<'a>> = match f {
            Node::Value(v) => v.value.to_vec(),
            other => vec![other],
          };
          ImportResult::one(make_media(
            self.a,
            AtRuleKind::Media,
            vec![contents],
            fv,
            Pos::NONE,
            VisInfo::default(),
            None,
          ))
        }
        None => ImportResult::list(vec![contents]),
      });
    }
    if imp.css.get() == Some(true) || imp.layer_css.get() {
      let path = self.eval_import_path(imp)?;
      // `new Import(path, features, options, index)` has no fileInfo
      let new_import = make_import(
        self.a,
        path,
        features,
        imp.options,
        Pos::NONE,
        VisInfo::default(),
      );
      new_import
        .path_reference
        .set(self.pos_is_reference(imp.pos));
      if imp.layer_css.get() {
        new_import.css.set(Some(true));
      }
      if new_import.css.get() != Some(true)
        && let Some(eid) = imp.error.get()
      {
        let e = self.s.import_errors.borrow()[eid as usize].clone();
        return Err(e);
      }
      return Ok(ImportResult::one(Node::Import(new_import)));
    }
    if let Some(root) = imp.root.get() {
      if let Some(fv) = feature_values
        && fv.len() == 1
        && let Some(e) = Self::is_layer_expr(Some(fv[0]))
      {
        imp.layer_css.set(true);
        self.rewrite_layer_features(imp, e);
        return Ok(ImportResult::one(Node::Import(imp)));
      }
      // `new Ruleset(null, copyArray(this.root.rules)).evalImports(context)`:
      // that ruleset is in no frame, so nothing observes it while its
      // imports are expanded, and the list can be built in one pass
      let src: Vec<Node<'a>> = match root {
        ImportRoot::Tree(r) => r.rules_vec(),
        ImportRoot::Text(_) => Vec::new(),
      };
      let mut out: BVec<'a, Node<'a>> = BVec::with_capacity_in(src.len(), self.a.bump);
      for n in src {
        match n {
          Node::Import(i) => {
            let r = self.eval_import(i)?;
            out.extend_from_slice(&r.nodes);
          }
          other => out.push(other),
        }
      }
      let rules: &'a [Node<'a>] = out.into_bump_slice();
      return Ok(match imp.features() {
        Some(f) => {
          let fv: Vec<Node<'a>> = match f {
            Node::Value(v) => v.value.to_vec(),
            other => vec![other],
          };
          ImportResult::one(make_media(
            self.a,
            AtRuleKind::Media,
            rules.to_vec(),
            fv,
            Pos::NONE,
            VisInfo::default(),
            None,
          ))
        }
        None => ImportResult {
          nodes: rules.into(),
          is_list: true,
        },
      });
    }
    if let Some(fv) = feature_values
      && !fv.is_empty()
      && let Some(e) = Self::is_layer_expr(Some(fv[0]))
    {
      imp.css.set(Some(true));
      self.rewrite_layer_features(imp, e);
      return Ok(ImportResult::one(Node::Import(imp)));
    }
    Ok(ImportResult::list(vec![]))
  }

  fn pos_is_reference(&self, pos: Pos) -> bool {
    self
      .s
      .pos_file_info(pos)
      .map(|i| i.reference)
      .unwrap_or(false)
  }

  /// `Import.evalPath`
  fn eval_import_path(&mut self, imp: &'a Import<'a>) -> LResult<Node<'a>> {
    let path = self.eval(imp.path)?;
    if let Node::Url(_) = path {
      return Ok(path);
    }
    let info = self.s.pos_file_info(imp.pos);
    let pv = super::js_string_value(path);
    if let Some(pv) = pv {
      let new_value = match info {
        Some(i) if !pv.is_empty() && self.path_requires_rewrite(pv) => rewrite_path(pv, i.rootpath),
        _ => normalize_path(pv),
      };
      return Ok(with_string_value(self.a, path, self.str(&new_value)));
    }
    Ok(path)
  }

  /// `Import.evalForImport`
  pub(crate) fn eval_for_import(&mut self, imp: &'a Import<'a>) -> LResult<&'a Import<'a>> {
    let path = match imp.path {
      Node::Url(u) => u.value,
      p => p,
    };
    let p = self.eval(path)?;
    let ni = make_import(
      self.a,
      p,
      imp.features,
      imp.options,
      imp.pos,
      imp.vis.info(),
    );
    Ok(ni)
  }
}

/// `permute(arr)`
fn permute<'a>(arr: &[Vec<Node<'a>>]) -> Vec<Vec<Node<'a>>> {
  if arr.is_empty() {
    return vec![];
  }
  if arr.len() == 1 {
    return arr[0].iter().map(|n| vec![*n]).collect();
  }
  let rest = permute(&arr[1..]);
  let mut result = Vec::new();
  for r in &rest {
    for j in &arr[0] {
      let mut v = vec![*j];
      v.extend_from_slice(r);
      result.push(v);
    }
  }
  result
}

#[allow(dead_code)]
fn _unused(e: Box<LessError>) -> Box<LessError> {
  e
}
