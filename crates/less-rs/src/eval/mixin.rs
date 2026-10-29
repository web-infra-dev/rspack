//! Mixins, detached rulesets and lookups (`mixin-call.js`,
//! `mixin-definition.js`, `detached-ruleset.js`, `variable-call.js`,
//! `namespace-value.js`).

use std::cell::Cell;

use super::EvalCtx;
use crate::{
  css,
  error::{LResult, LessError},
  session::DefaultState,
  syntax::parser::make_mixin_definition,
  tree::*,
};

/// An evaluated mixin call argument (`{ name, value }`).
#[derive(Copy, Clone)]
pub(crate) struct CallArg<'a> {
  pub name: Option<&'a str>,
  pub value: Node<'a>,
}

const DEF_FALSE_EITHER_CASE: i32 = -1;
const DEF_NONE: i32 = 0;
const DEF_TRUE: i32 = 1;
const DEF_FALSE: i32 = 2;

impl<'a, 's> EvalCtx<'a, 's> {
  pub(crate) fn eval_mixin_call(&mut self, mc: &'a MixinCall<'a>) -> LResult<Vec<Node<'a>>> {
    self.enter_pub()?;
    let r = if self.depth.is_multiple_of(4) {
      super::grow(|| self.eval_mixin_call_inner(mc))
    } else {
      self.eval_mixin_call_inner(mc)
    };
    self.leave_pub();
    r
  }

  fn eval_mixin_call_inner(&mut self, mc: &'a MixinCall<'a>) -> LResult<Vec<Node<'a>>> {
    let selector = self.eval_selector(mc.selector)?;
    let mut args: Vec<CallArg<'a>> = Vec::new();
    for arg in mc.arguments {
      let v = match arg.value {
        Some(v) => self.eval(v)?,
        None => continue,
      };
      if arg.expand
        && let Some(items) = v.array_value()
      {
        for item in items {
          args.push(CallArg {
            name: None,
            value: *item,
          });
        }
        continue;
      }
      if let Node::Expression(e) = v
        && e.value.is_empty()
      {
        continue;
      }
      args.push(CallArg {
        name: arg.name,
        value: v,
      });
    }

    let mut is_one_found = false;
    let mut rules: Vec<Node<'a>> = Vec::new();
    let key = super::ruleset::selector_key(selector)?;
    let mut fi = 0;
    while fi < self.frames.len() {
      let frame = self.frames[fi];
      fi += 1;
      let mixins = self.find_keyed(frame, selector, &key, None, true)?;
      if mixins.is_empty() {
        continue;
      }
      is_one_found = true;
      let mut candidates: Vec<(Node<'a>, i32)> = Vec::new();
      let mut matched = false;
      for found in mixins {
        let mixin = found.rule;
        let mut is_recursive = false;
        if !matches!(mixin, Node::MixinDefinition(_)) {
          for f in &self.frames {
            let orig = f.original().unwrap_or(Node::Ruleset(f));
            if orig.addr() == mixin.addr() {
              is_recursive = true;
              break;
            }
          }
        }
        if is_recursive {
          continue;
        }
        if self.match_args(mixin, Some(&args))? {
          let group = self.calc_def_group(mixin, found.path, &args)?;
          if group != DEF_FALSE_EITHER_CASE {
            candidates.push((mixin, group));
          }
          matched = true;
        }
      }
      self.s.default_state.set(DefaultState::Unset);
      let mut count = [0usize; 3];
      for (_, g) in &candidates {
        count[*g as usize] += 1;
      }
      let default_result;
      if count[DEF_NONE as usize] > 0 {
        default_result = DEF_FALSE;
      } else {
        default_result = DEF_TRUE;
        if count[DEF_TRUE as usize] + count[DEF_FALSE as usize] > 1 {
          return Err(LessError::at(
            "Runtime",
            format!(
              "Ambiguous use of `default()` found when matching for `{}`",
              self.format_call(selector, &args)
            ),
            mc.pos,
          ));
        }
      }
      for (mixin, group) in candidates {
        if group == DEF_NONE || group == default_result {
          let captured = self.s.frames_captured.get();
          let r = self.apply_mixin(mixin, &args, mc.important);
          match r {
            Ok(rs) => {
              {
                let b = rs.rules.borrow();
                let new_rules: &[Node<'a>] = b.as_deref().unwrap_or(&[]);
                if mc.vis.blocks_visibility() {
                  self.s.vis_blocked.set(true);
                  for r in new_rules {
                    let r = self.unshared(*r);
                    r.add_visibility_block();
                    rules.push(r);
                  }
                } else {
                  rules.extend_from_slice(new_rules);
                }
              }
              // only its rules live on
              if self.s.frames_captured.get() == captured {
                self.recycle_ruleset(rs);
              }
            }
            Err(mut e) => {
              // `throw { ...e, index, filename }`: a native
              // Error's message/type are not own enumerable
              if e.native {
                e.message = "undefined".to_string();
                e.typ = String::new();
                e.native = false;
              }
              e.set_pos(mc.pos);
              return Err(e);
            }
          }
        }
      }
      if matched {
        return Ok(rules);
      }
    }
    if is_one_found {
      Err(LessError::at(
        "Runtime",
        format!(
          "No matching definition was found for `{}`",
          self.format_call(selector, &args)
        ),
        mc.pos,
      ))
    } else {
      let sel = css::to_css_plain(Node::Selector(selector))?;
      Err(LessError::at(
        "Name",
        format!("{} is undefined", sel.trim()),
        mc.pos,
      ))
    }
  }

  fn apply_mixin(
    &mut self,
    mixin: Node<'a>,
    args: &[CallArg<'a>],
    important: bool,
  ) -> LResult<&'a Ruleset<'a>> {
    let def = match mixin {
      Node::MixinDefinition(d) => d,
      other => {
        let rs = other.as_ruleset().expect("mixin candidate is a ruleset");
        let original = rs.original().unwrap_or(other);
        let vis = original.visibility_info();
        let rules = rs
          .rules
          .borrow()
          .as_ref()
          .map(|r| self.a.vec_from(r))
          .unwrap_or_else(|| self.a.vec());
        let n = make_mixin_definition(self.a, Some(""), &[], rules, None, false, None, vis);
        let d = n.as_ruleset().expect("mixin definition is a ruleset");
        d.original_ruleset.set(original.as_ruleset());
        d
      }
    };
    self.mixin_eval_call(def, args, important)
  }

  fn calc_def_group(
    &mut self,
    mixin: Node<'a>,
    path: &'a [Node<'a>],
    args: &[CallArg<'a>],
  ) -> LResult<i32> {
    let mut cond = [true, true];
    for f in 0..2 {
      cond[f] = true;
      self.s.default_state.set(DefaultState::Value(f == 1));
      let mut p = 0;
      while p < path.len() && cond[f] {
        cond[f] = cond[f] && self.match_condition(path[p], None)?;
        p += 1;
      }
      cond[f] = cond[f] && self.match_condition(mixin, Some(args))?;
    }
    if cond[0] || cond[1] {
      if cond[0] != cond[1] {
        return Ok(if cond[1] { DEF_TRUE } else { DEF_FALSE });
      }
      return Ok(DEF_NONE);
    }
    Ok(DEF_FALSE_EITHER_CASE)
  }

  pub(crate) fn format_call(&self, selector: &'a Selector<'a>, args: &[CallArg<'a>]) -> String {
    let sel = css::to_css_plain(Node::Selector(selector)).unwrap_or_default();
    let parts: Vec<String> = args
      .iter()
      .map(|a| {
        let mut s = String::new();
        if let Some(n) = a.name {
          s.push_str(n);
          s.push(':');
        }
        s.push_str(&css::to_css_plain(a.value).unwrap_or_else(|_| "???".into()));
        s
      })
      .collect();
    format!("{}({})", sel.trim(), parts.join(", "))
  }

  /// `rule.matchArgs(args, context)` for rulesets and mixin definitions.
  pub(crate) fn match_args(
    &mut self,
    rule: Node<'a>,
    args: Option<&[CallArg<'a>]>,
  ) -> LResult<bool> {
    let rs = match rule.as_ruleset() {
      Some(r) => r,
      None => return Ok(false),
    };
    let def = match rs.mixin {
      None => return Ok(args.map(|a| a.is_empty()).unwrap_or(true)),
      Some(d) => d,
    };
    let args = args.unwrap_or(&[]);
    let params = def.params;
    let mut evald: smallvec::SmallVec<[Option<CallArg<'a>>; 8]> =
      smallvec::smallvec![None; params.len()];
    let mut positional: smallvec::SmallVec<[CallArg<'a>; 8]> = smallvec::SmallVec::new();
    for arg in args {
      match arg.name {
        None => positional.push(*arg),
        Some(name) => {
          let idx = params
            .iter()
            .enumerate()
            .position(|(i, p)| p.name == Some(name) && evald[i].is_none());
          match idx {
            Some(i) => evald[i] = Some(*arg),
            None => return Ok(false),
          }
        }
      }
    }
    let mut positional_index = 0;
    for i in 0..params.len() {
      if evald[i].is_some() {
        continue;
      }
      if params[i].variadic {
        positional_index = positional.len();
        continue;
      }
      if positional_index < positional.len() {
        evald[i] = Some(positional[positional_index]);
        positional_index += 1;
      } else if params[i].name.is_none() || params[i].value.is_none() {
        return Ok(false);
      }
    }
    if positional_index < positional.len() {
      return Ok(false);
    }
    for i in 0..def.arity {
      if params[i].name.is_none() && !params[i].variadic {
        let a = match evald[i] {
          Some(a) => a,
          None => return Ok(false),
        };
        let av = self.eval(a.value)?;
        let pv = match params[i].value {
          Some(v) => self.eval(v)?,
          None => return Ok(false),
        };
        if css::to_css_plain(av)? != css::to_css_plain(pv)? {
          return Ok(false);
        }
      }
    }
    Ok(true)
  }

  /// `rule.matchCondition(args, context)`
  pub(crate) fn match_condition(
    &mut self,
    rule: Node<'a>,
    args: Option<&[CallArg<'a>]>,
  ) -> LResult<bool> {
    let rs = match rule.as_ruleset() {
      Some(r) => r,
      None => return Ok(true),
    };
    match rs.mixin {
      None => {
        let sels = rs.selectors();
        let last = match sels.last() {
          Some(l) => *l,
          None => return Ok(true),
        };
        if !last.evald_condition.get() {
          return Ok(false);
        }
        if let Some(Node::Condition(c)) = last.condition() {
          let mut ctx = self.derive(self.frames.clone());
          if !ctx.eval_condition(c)? {
            return Ok(false);
          }
        }
        Ok(true)
      }
      Some(def) => {
        let cond = match def.condition {
          Some(Node::Condition(c)) => c,
          _ => return Ok(true),
        };
        // frames = [frame, ...mixin frames, ...context frames]; the
        // params are evaluated in frames[1..]
        let def_frames = def.frames.unwrap_or(&[]);
        let mut frames: Vec<&'a Ruleset<'a>> =
          Vec::with_capacity(1 + def_frames.len() + self.frames.len());
        frames.push(rs);
        frames.extend_from_slice(def_frames);
        frames.extend_from_slice(&self.frames);
        let captured = self.s.frames_captured.get();
        let mut evald = EvaldArgs::new();
        let frame = self.eval_params(rs, def, &frames[1..], args, &mut evald)?;
        frames[0] = frame;
        let mut ctx = self.derive(frames);
        let r = ctx.eval_condition(cond)?;
        drop(ctx);
        if self.s.frames_captured.get() == captured {
          self.recycle_ruleset(frame);
        }
        Ok(r)
      }
    }
  }

  /// `MixinDefinition.evalParams`
  fn eval_params(
    &mut self,
    _rs: &'a Ruleset<'a>,
    def: &'a MixinDef<'a>,
    mixin_frames: &[&'a Ruleset<'a>],
    args: Option<&[CallArg<'a>]>,
    evald_arguments: &mut EvaldArgs<'a>,
  ) -> LResult<&'a Ruleset<'a>> {
    let frame = self.pooled_ruleset(None);
    // the parameters and `@arguments`
    let cap = def.params.len() + 1;
    let params = def.params;
    let mut env: Option<EvalCtx<'a, 's>> = None;
    let mut args: smallvec::SmallVec<[CallArg<'a>; 8]> = args
      .map(|a| a.iter().copied().collect())
      .unwrap_or_default();
    let args_length = args.len();
    if evald_arguments.len() < params.len().max(args_length) {
      evald_arguments.resize(params.len().max(args_length), None);
    }
    let mut i = 0;
    while i < args.len() {
      let arg = args[i];
      if let Some(name) = arg.name {
        let mut found = false;
        for j in 0..params.len() {
          if evald_arguments[j].is_none() && Some(name) == params[j].name {
            evald_arguments[j] = Some(self.eval(arg.value)?);
            let v = self.eval(arg.value)?;
            prepend_rule_sized(
              self.a,
              frame,
              Node::Declaration(self.a.declaration_ref(
                self.s.decl_name(name),
                v,
                "",
                Merge::None,
                Pos::NONE,
                false,
                None,
              )),
              cap,
            );
            found = true;
            break;
          }
        }
        if found {
          args.remove(i);
          continue;
        } else {
          return Err(LessError::new(
            "Runtime",
            format!("Named argument for {} {} not found", def.name, name),
          ));
        }
      }
      i += 1;
    }
    let mut arg_index = 0;
    for i in 0..params.len() {
      if evald_arguments[i].is_some() {
        continue;
      }
      let arg = args.get(arg_index).copied();
      if let Some(name) = params[i].name {
        if params[i].variadic {
          let mut varargs: smallvec::SmallVec<[Node<'a>; 8]> = smallvec::SmallVec::new();
          for j in arg_index..args_length {
            if let Some(a) = args.get(j) {
              varargs.push(self.eval(a.value)?);
            }
          }
          let e = self.a.expression(&varargs, None);
          let v = self.eval(Node::Expression(e))?;
          prepend_rule_sized(
            self.a,
            frame,
            Node::Declaration(self.a.declaration_ref(
              self.s.decl_name(name),
              v,
              "",
              Merge::None,
              Pos::NONE,
              false,
              None,
            )),
            cap,
          );
        } else {
          let val = match arg {
            Some(a) => match a.value {
              Node::Rules(r) => {
                let rs = self.a.ruleset(None, Some(self.a.vec_from(r)));
                self.a.detached_ruleset(Node::Ruleset(rs), None)
              }
              v => self.eval(v)?,
            },
            None => match params[i].value {
              Some(dv) => {
                let env = env.get_or_insert_with(|| {
                  let mut f = Vec::with_capacity(1 + mixin_frames.len());
                  f.push(frame);
                  f.extend_from_slice(mixin_frames);
                  self.derive(f)
                });
                let v = env.eval(dv)?;
                frame.reset_cache();
                v
              }
              None => {
                return Err(LessError::new(
                  "Runtime",
                  format!(
                    "wrong number of arguments for {} ({} for {})",
                    def.name, args_length, def.arity
                  ),
                ));
              }
            },
          };
          prepend_rule_sized(
            self.a,
            frame,
            Node::Declaration(self.a.declaration_ref(
              self.s.decl_name(name),
              val,
              "",
              Merge::None,
              Pos::NONE,
              false,
              None,
            )),
            cap,
          );
          evald_arguments[i] = Some(val);
        }
      }
      if params[i].variadic {
        for j in arg_index..args_length {
          if let Some(a) = args.get(j) {
            let v = self.eval(a.value)?;
            if j >= evald_arguments.len() {
              evald_arguments.resize(j + 1, None);
            }
            evald_arguments[j] = Some(v);
          }
        }
      }
      arg_index += 1;
    }
    Ok(frame)
  }

  /// `MixinDefinition.evalCall`
  pub(crate) fn mixin_eval_call(
    &mut self,
    rs: &'a Ruleset<'a>,
    args: &[CallArg<'a>],
    important: bool,
  ) -> LResult<&'a Ruleset<'a>> {
    let def = rs.mixin.expect("mixin definition has metadata");
    // [this, frame, ...mixinFrames]; the frame slot is filled below
    let def_frames = def.frames.unwrap_or(&[]);
    check_frames_len(def_frames.len() + self.frames.len())?;
    let mut frames: Vec<&'a Ruleset<'a>> =
      Vec::with_capacity(2 + def_frames.len() + self.frames.len());
    frames.push(rs);
    frames.push(rs);
    frames.extend_from_slice(def_frames);
    frames.extend_from_slice(&self.frames);
    let captured = self.s.frames_captured.get();
    let mut evald = EvaldArgs::new();
    let frame = self.eval_params(rs, def, &frames[2..], Some(args), &mut evald)?;
    frames[1] = frame;
    let arguments: smallvec::SmallVec<[Node<'a>; 8]> = evald.iter().filter_map(|x| *x).collect();
    let e = self.a.expression(&arguments, None);
    let av = self.eval(Node::Expression(e))?;
    prepend_rule(
      self.a,
      frame,
      Node::Declaration(self.a.declaration(
        DeclName::Str("@arguments"),
        av,
        "",
        Merge::None,
        Pos::NONE,
        false,
        None,
      )),
    );
    // `ruleset = new Ruleset(null, rules); ruleset.originalRuleset = this;
    // ruleset.eval(...)`
    let mut ctx = self.derive(frames);
    let evaluated = ctx.eval_ruleset_plain(rs)?;
    drop(ctx);
    let result = if important {
      self.make_ruleset_important(evaluated)
    } else {
      evaluated
    };
    if self.s.frames_captured.get() == captured {
      // nothing kept the frame list: the parameter frame is dead
      self.recycle_ruleset(frame);
      if important {
        self.recycle_ruleset(evaluated);
      }
    }
    Ok(result)
  }

  /// `MixinDefinition.eval`: captures the current frames.
  pub(crate) fn eval_mixin_definition(&mut self, rs: &'a Ruleset<'a>) -> Node<'a> {
    let def = rs.mixin.expect("mixin definition has metadata");
    let frames = match def.frames {
      Some(f) => f,
      None => {
        self.s.frames_captured.set(self.s.frames_captured.get() + 1);
        self.a.slice(&self.frames)
      }
    };
    let rules = rs
      .rules
      .borrow()
      .as_ref()
      .map(|r| self.a.vec_from(r))
      .unwrap_or_else(|| self.a.vec());
    let name = Some(def.name);

    make_mixin_definition(
      self.a,
      name,
      def.params,
      rules,
      def.condition,
      def.variadic,
      Some(frames),
      VisInfo::default(),
    )
  }

  // ------------------------------------------------------------------
  // detached rulesets, variable calls, lookups
  // ------------------------------------------------------------------

  /// `DetachedRuleset.callEval`
  pub(crate) fn call_eval_detached(
    &mut self,
    d: &'a DetachedRuleset<'a>,
  ) -> LResult<&'a Ruleset<'a>> {
    let rs = match d.ruleset {
      Node::Ruleset(r) | Node::MixinDefinition(r) => r,
      _ => {
        return Err(LessError::new(
          "Syntax",
          "Could not evaluate detached ruleset",
        ));
      }
    };
    match d.frames {
      Some(f) => {
        check_frames_len(f.len() + self.frames.len())?;
        let mut frames: Vec<&'a Ruleset<'a>> = f.to_vec();
        frames.extend(self.frames.iter().copied());
        let mut ctx = self.derive(frames);
        ctx.eval_ruleset(rs)
      }
      None => self.eval_ruleset(rs),
    }
  }

  fn variable_call_target(&mut self, vc: &'a VariableCall<'a>) -> LResult<&'a DetachedRuleset<'a>> {
    let var = self.a.alloc(Variable {
      name: vc.variable,
      pos: vc.pos,
      evaluating: Cell::new(false),
      no_spacing: Cell::new(None),
    });
    let v = self.eval_variable(var)?;
    let err = || {
      LessError::new(
        "Syntax",
        format!("Could not evaluate variable call {}", vc.variable),
      )
    };
    match v {
      Node::DetachedRuleset(d) => Ok(d),
      Node::Ruleset(r) | Node::MixinDefinition(r) => Ok(self.a.alloc(DetachedRuleset {
        vis: Vis::default(),
        ruleset: Node::Ruleset(r),
        frames: None,
      })),
      Node::Rules(list) => {
        let rs = self.a.ruleset(None, Some(self.a.vec_from(list)));
        Ok(self.a.alloc(DetachedRuleset {
          vis: Vis::default(),
          ruleset: Node::Ruleset(rs),
          frames: None,
        }))
      }
      other => match other.array_value() {
        Some(list) => {
          let rs = self.a.ruleset(None, Some(self.a.vec_from(list)));
          Ok(self.a.alloc(DetachedRuleset {
            vis: Vis::default(),
            ruleset: Node::Ruleset(rs),
            frames: None,
          }))
        }
        None => Err(err()),
      },
    }
  }

  pub(crate) fn eval_variable_call(&mut self, vc: &'a VariableCall<'a>) -> LResult<Node<'a>> {
    let d = self.variable_call_target(vc)?;
    Ok(Node::Ruleset(self.call_eval_detached(d)?))
  }

  pub(crate) fn eval_variable_call_rules(
    &mut self,
    vc: &'a VariableCall<'a>,
  ) -> LResult<Vec<Node<'a>>> {
    let d = self.variable_call_target(vc)?;
    Ok(self.call_eval_detached(d)?.rules_vec())
  }

  pub(crate) fn eval_namespace_value(&mut self, nv: &'a NamespaceValue<'a>) -> LResult<Node<'a>> {
    let mut rules: Node<'a> = match nv.value {
      Node::MixinCall(mc) => {
        let r = self.eval_mixin_call(mc)?;
        Node::Rules(self.a.rules(&r))
      }
      other => self.eval(other)?,
    };
    for &lookup in nv.lookups {
      let mut name: &'a str = lookup;
      if let Node::Rules(list) = rules {
        let sel = self.a.selector_amp(Pos::NONE);
        rules = Node::Ruleset(
          self
            .a
            .ruleset(Some(self.a.slice(&[sel])), Some(self.a.vec_from(list))),
        );
      }
      let rs = rules.as_ruleset();
      let found: Option<Node<'a>>;
      if name.is_empty() {
        found = match rs {
          Some(r) => self.last_declaration(r)?.map(Node::Declaration),
          None => None,
        };
        if found.is_none() {
          return Err(self.lookup_type_error(nv, rules, "lastDeclaration"));
        }
      } else if name.starts_with('@') {
        if name.starts_with("@@") {
          let v = self.a.variable(&name[1..], Pos::NONE);
          let r = self.eval(v)?;
          name = self.str(&format!("@{}", super::js_value_string(r)));
        }
        found = match rs {
          Some(r) => self.frame_variable(r, name)?.map(Node::Declaration),
          None => None,
        };
        if found.is_none() {
          return Err(LessError::at(
            "Name",
            format!("variable {} not found", name),
            nv.pos,
          ));
        }
      } else {
        let pname: &'a str = if name.starts_with("$@") {
          let v = self.a.variable(&name[1..], Pos::NONE);
          let r = self.eval(v)?;
          self.str(&format!("${}", super::js_value_string(r)))
        } else if name.starts_with('$') {
          name
        } else {
          self.str(&format!("${}", name))
        };
        let list = match rs {
          Some(r) => self.frame_property(r, pname)?,
          None => None,
        };
        match list {
          Some(l) if !l.is_empty() => {
            found = Some(Node::Declaration(
              l.last().expect("property declarations are nonempty"),
            ))
          }
          _ => {
            return Err(LessError::at(
              "Name",
              format!("property \"{}\" not found", &pname[1..]),
              nv.pos,
            ));
          }
        }
      }
      let mut current = found.expect("missing lookup value was handled above");
      if let Node::Declaration(d) = current {
        let ed = self.eval_declaration(d)?;
        current = ed.value.get();
      }
      if let Node::DetachedRuleset(dr) = current {
        current = match dr.ruleset {
          Node::Ruleset(r) => Node::Ruleset(self.eval_ruleset(r)?),
          other => self.eval(other)?,
        };
      }
      rules = current;
    }
    Ok(rules)
  }

  fn lookup_type_error(
    &self,
    nv: &'a NamespaceValue<'a>,
    _rules: Node<'a>,
    method: &str,
  ) -> Box<LessError> {
    LessError::at("", format!("rules.{} is not a function", method), nv.pos)
  }
}

/// Frame lists are concatenated on every detached-ruleset / mixin call; a
/// self-referencing call doubles them each level. less.js eventually fails
/// with `RangeError: Invalid array length`; stop long before memory runs out.
fn check_frames_len(n: usize) -> LResult<()> {
  if n > (1 << 20) {
    return Err(LessError::native("Invalid array length"));
  }
  Ok(())
}

/// `ruleset.prependRule(rule)`
type EvaldArgs<'a> = smallvec::SmallVec<[Option<Node<'a>>; 8]>;

pub(crate) fn prepend_rule<'a>(a: Alloc<'a>, rs: &'a Ruleset<'a>, rule: Node<'a>) {
  prepend_rule_sized(a, rs, rule, 1);
}

/// `prependRule`, sizing a new list for `capacity` rules (frames get one
/// rule per parameter; growing would strand the old lists in the arena).
pub(crate) fn prepend_rule_sized<'a>(
  a: Alloc<'a>,
  rs: &'a Ruleset<'a>,
  rule: Node<'a>,
  capacity: usize,
) {
  rs.bump_generation();
  let mut b = rs.rules.borrow_mut();
  match b.as_mut() {
    Some(v) => v.insert(0, rule),
    None => {
      let mut v = BVec::with_capacity_in(capacity.max(1), a.bump);
      v.push(rule);
      *b = Some(v);
    }
  }
}
