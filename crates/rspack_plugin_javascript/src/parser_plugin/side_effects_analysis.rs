/**
 * Some code is modified based on
 * https://github.com/CPunisher/swc-experimental/blob/57cf78c4bc1b963d4e4946492b88164958e6f764/crates/swc_ecma_utils/src/lib.rs
 * Apache-2.0 licensed
 */
// Conservative, syntax-level side-effects analysis.
//
// Rspack-specific policies such as `/*#__PURE__*/`, `pureFunctions`, parser hooks, and deferred
// import checks stay in `side_effects_parser_plugin`.
use swc_experimental_ecma_ast::{
  Callee, Class, ClassMember, Decl, Expr, Lit, MemberProp, MethodKind, OptChainBase, Pat, Prop,
  PropName, PropOrSpread, Stmt, UnaryOp, VarDeclKind,
};
use swc_experimental_ecma_semantic::resolver::Semantic;

#[derive(Clone, Copy)]
pub(super) struct SideEffectsContext<'a> {
  semantic: &'a Semantic,
  is_unresolved_ref_safe: bool,
  in_strict: bool,
  remaining_depth: u8,
}

impl<'a> SideEffectsContext<'a> {
  pub(super) fn new(semantic: &'a Semantic, is_unresolved_ref_safe: bool) -> Self {
    Self {
      semantic,
      is_unresolved_ref_safe,
      in_strict: false,
      remaining_depth: 4,
    }
  }

  fn consume_depth(self) -> Option<Self> {
    if self.remaining_depth == 0 {
      return None;
    }

    Some(Self {
      remaining_depth: self.remaining_depth - 1,
      ..self
    })
  }
}

pub(super) trait MayHaveSideEffects {
  fn is_pure_callee(&self, ctx: SideEffectsContext<'_>) -> bool;
  fn may_have_side_effects(&self, ctx: SideEffectsContext<'_>) -> bool;
  fn is_global_ref_to(&self, ctx: SideEffectsContext<'_>, id: &str) -> bool;
}

impl MayHaveSideEffects for Expr<'_> {
  fn may_have_side_effects(&self, ctx: SideEffectsContext<'_>) -> bool {
    let Some(ctx) = ctx.consume_depth() else {
      return true;
    };

    if self.is_pure_callee(ctx) {
      return false;
    }

    match self {
      Expr::Ident(ident) => {
        if ctx.is_unresolved_ref_safe {
          return false;
        }

        if ctx.semantic.node_scope(ident) == ctx.semantic.unresolved_scope_id() {
          !matches!(
            ident.sym.as_str(),
            "Infinity"
              | "NaN"
              | "Math"
              | "undefined"
              | "Object"
              | "Array"
              | "Promise"
              | "Boolean"
              | "Number"
              | "String"
              | "BigInt"
              | "Error"
              | "RegExp"
              | "Function"
              | "document"
          )
        } else {
          false
        }
      }
      Expr::Lit(..) | Expr::This(..) | Expr::PrivateName(..) => false,
      Expr::Paren(parenthesized) => parenthesized.expr.may_have_side_effects(ctx),
      Expr::Fn(..) | Expr::Arrow(..) => false,
      Expr::Class(class) => class_has_side_effects(ctx, &class.class),
      Expr::Array(array) => array
        .elems
        .iter()
        .flatten()
        .any(|element| element.spread.is_some() || element.expr.may_have_side_effects(ctx)),
      Expr::Unary(unary) => match unary.op {
        UnaryOp::Delete => true,
        _ => unary.arg.may_have_side_effects(ctx),
      },
      Expr::Bin(binary) => {
        binary.left.may_have_side_effects(ctx) || binary.right.may_have_side_effects(ctx)
      }
      Expr::Member(member)
        if matches!(
          &member.obj,
          Expr::Object(_) | Expr::Fn(_) | Expr::Arrow(_) | Expr::Class(_)
        ) =>
      {
        let object = &member.obj;
        if object.may_have_side_effects(ctx) {
          return true;
        }

        match object {
          Expr::Class(class)
            if class.class.body.iter().any(|member| {
              matches!(
                member,
                ClassMember::Method(method)
                  if (method.kind == MethodKind::Getter || method.kind == MethodKind::Setter)
                    && method.is_static
              )
            }) =>
          {
            return true;
          }
          Expr::Object(object) => {
            let can_have_side_effects = |property: &PropOrSpread<'_>| match property {
              PropOrSpread::Spread(_) => true,
              PropOrSpread::Prop(property) => match &**property {
                Prop::Getter(_) | Prop::Setter(_) | Prop::Method(_) => true,
                Prop::Shorthand(identifier) => identifier.sym == "__proto__",
                Prop::KeyValue(key_value) => match &key_value.key {
                  PropName::Ident(identifier) => identifier.sym == "__proto__",
                  PropName::Str(string) => string.value.as_wtf8().as_str() == Some("__proto__"),
                  PropName::Computed(_) => true,
                  _ => false,
                },
                _ => false,
              },
            };
            if object.props.iter().any(can_have_side_effects) {
              return true;
            }
          }
          _ => {}
        }

        match &member.prop {
          MemberProp::Computed(computed) => computed.expr.may_have_side_effects(ctx),
          MemberProp::Ident(_) | MemberProp::PrivateName(_) => false,
        }
      }
      Expr::Tpl(_) | Expr::TaggedTpl(_) | Expr::MetaProp(_) => true,
      Expr::Await(_)
      | Expr::Yield(_)
      | Expr::Member(_)
      | Expr::SuperProp(_)
      | Expr::Update(_)
      | Expr::Assign(_) => true,
      Expr::OptChain(optional_chain) if matches!(&optional_chain.base, OptChainBase::Member(_)) => {
        true
      }
      Expr::New(new_expression) if is_pure_new_callee(&new_expression.callee, ctx) => {
        new_expression.args.as_ref().is_some_and(|arguments| {
          arguments
            .iter()
            .any(|argument| argument.expr.may_have_side_effects(ctx))
        })
      }
      Expr::New(_) => true,
      Expr::Call(call_expression) => {
        let Callee::Expr(callee) = &call_expression.callee else {
          return true;
        };

        if callee.is_pure_callee(ctx) {
          call_expression
            .args
            .iter()
            .any(|argument| argument.expr.may_have_side_effects(ctx))
        } else {
          true
        }
      }
      Expr::OptChain(optional_chain) => match &optional_chain.base {
        OptChainBase::Call(call) if call.callee.is_pure_callee(ctx) => call
          .args
          .iter()
          .any(|argument| argument.expr.may_have_side_effects(ctx)),
        _ => true,
      },
      Expr::Seq(sequence) => sequence
        .exprs
        .iter()
        .any(|expression| expression.may_have_side_effects(ctx)),
      Expr::Cond(conditional) => {
        conditional.test.may_have_side_effects(ctx)
          || conditional.cons.may_have_side_effects(ctx)
          || conditional.alt.may_have_side_effects(ctx)
      }
      Expr::Object(object) => object.props.iter().any(|property| match property {
        PropOrSpread::Prop(property) => match &**property {
          Prop::Shorthand(..) => false,
          Prop::KeyValue(key_value) => {
            let key_has_side_effects = match &key_value.key {
              PropName::Computed(computed) => computed.expr.may_have_side_effects(ctx),
              _ => false,
            };
            key_has_side_effects || key_value.value.may_have_side_effects(ctx)
          }
          Prop::Getter(getter) => match &getter.key {
            PropName::Computed(computed) => computed.expr.may_have_side_effects(ctx),
            _ => false,
          },
          Prop::Setter(setter) => match &setter.key {
            PropName::Computed(computed) => computed.expr.may_have_side_effects(ctx),
            _ => false,
          },
          Prop::Method(method) => match &method.key {
            PropName::Computed(computed) => computed.expr.may_have_side_effects(ctx),
            _ => false,
          },
          Prop::Assign(_) => true,
        },
        PropOrSpread::Spread(_) => true,
      }),
      Expr::JSXMember(..)
      | Expr::JSXNamespacedName(..)
      | Expr::JSXEmpty(..)
      | Expr::JSXElement(..)
      | Expr::JSXFragment(..)
      | Expr::Invalid(..) => true,
    }
  }

  fn is_pure_callee(&self, ctx: SideEffectsContext<'_>) -> bool {
    if self.is_global_ref_to(ctx, "Date") {
      return true;
    }

    match self {
      Expr::Member(member) => {
        let object = &member.obj;
        let property = &member.prop;

        if let MemberProp::Ident(property) = property {
          if object.is_global_ref_to(ctx, "Math") {
            return true;
          }

          match object {
            Expr::Ident(identifier) => identifier.sym == "Math",
            Expr::Lit(literal) if matches!(&**literal, Lit::Str(..)) => {
              is_pure_string_method(property.sym.as_str())
            }
            Expr::Tpl(template) if template.exprs.is_empty() => {
              is_pure_string_method(property.sym.as_str())
            }
            _ => false,
          }
        } else {
          false
        }
      }
      Expr::Fn(function) => {
        let function = &function.function;
        function
          .params
          .iter()
          .all(|parameter| matches!(&parameter.pat, Pat::Ident(_)))
          && function
            .body
            .as_ref()
            .is_some_and(|body| body.stmts.is_empty())
      }
      _ => false,
    }
  }

  fn is_global_ref_to(&self, ctx: SideEffectsContext<'_>, id: &str) -> bool {
    match self {
      Expr::Ident(identifier) => {
        ctx.semantic.node_scope(identifier) == ctx.semantic.unresolved_scope_id()
          && identifier.sym == id
      }
      _ => false,
    }
  }
}

trait StatementMayHaveSideEffects {
  fn may_have_side_effects(&self, ctx: SideEffectsContext<'_>) -> bool;
}

impl StatementMayHaveSideEffects for Stmt<'_> {
  fn may_have_side_effects(&self, ctx: SideEffectsContext<'_>) -> bool {
    match self {
      Stmt::Block(block) => block
        .stmts
        .iter()
        .any(|statement| statement.may_have_side_effects(ctx)),
      Stmt::Empty(_) => false,
      Stmt::Labeled(labeled) => labeled.body.may_have_side_effects(ctx),
      Stmt::If(if_statement) => {
        if_statement.test.may_have_side_effects(ctx)
          || if_statement.cons.may_have_side_effects(ctx)
          || if_statement
            .alt
            .as_ref()
            .is_some_and(|statement| statement.may_have_side_effects(ctx))
      }
      Stmt::Switch(switch) => {
        switch.discriminant.may_have_side_effects(ctx)
          || switch.cases.iter().any(|case| {
            case
              .test
              .as_ref()
              .is_some_and(|expression| expression.may_have_side_effects(ctx))
              || case
                .cons
                .iter()
                .any(|statement| statement.may_have_side_effects(ctx))
          })
      }
      Stmt::Try(try_statement) => {
        try_statement
          .block
          .stmts
          .iter()
          .any(|statement| statement.may_have_side_effects(ctx))
          || try_statement.handler.as_ref().is_some_and(|handler| {
            handler
              .body
              .stmts
              .iter()
              .any(|statement| statement.may_have_side_effects(ctx))
          })
          || try_statement.finalizer.as_ref().is_some_and(|finalizer| {
            finalizer
              .stmts
              .iter()
              .any(|statement| statement.may_have_side_effects(ctx))
          })
      }
      Stmt::Decl(declaration) => match &**declaration {
        Decl::Class(class) => class_has_side_effects(ctx, &class.class),
        Decl::Fn(_) => !ctx.in_strict,
        Decl::Var(variable) => variable.kind == VarDeclKind::Var,
        _ => false,
      },
      Stmt::Expr(expression) => expression.expr.may_have_side_effects(ctx),
      _ => true,
    }
  }
}

fn class_has_side_effects(ctx: SideEffectsContext<'_>, class: &Class<'_>) -> bool {
  if let Some(super_class) = &class.super_class
    && super_class.may_have_side_effects(ctx)
  {
    return true;
  }

  for member in &class.body {
    match member {
      ClassMember::Method(method) => {
        if let PropName::Computed(key) = &method.key
          && key.expr.may_have_side_effects(ctx)
        {
          return true;
        }
      }
      ClassMember::ClassProp(property) => {
        if let PropName::Computed(key) = &property.key
          && key.expr.may_have_side_effects(ctx)
        {
          return true;
        }
        if let Some(value) = &property.value
          && value.may_have_side_effects(ctx)
        {
          return true;
        }
      }
      ClassMember::PrivateProp(property) => {
        if let Some(value) = &property.value
          && value.may_have_side_effects(ctx)
        {
          return true;
        }
      }
      ClassMember::StaticBlock(block)
        if block
          .body
          .stmts
          .iter()
          .any(|statement| statement.may_have_side_effects(ctx)) =>
      {
        return true;
      }
      _ => {}
    }
  }

  false
}

fn is_pure_new_callee(expression: &Expr<'_>, ctx: SideEffectsContext<'_>) -> bool {
  match expression {
    Expr::Fn(function) => {
      let function = &function.function;
      function
        .params
        .iter()
        .all(|parameter| matches!(&parameter.pat, Pat::Ident(_)))
        && function
          .body
          .as_ref()
          .is_some_and(|body| body.stmts.is_empty())
    }
    Expr::Class(class) => {
      let class = &class.class;
      if class.super_class.is_some() || class_has_side_effects(ctx, class) {
        return false;
      }

      for member in &class.body {
        match member {
          ClassMember::ClassProp(property) if !property.is_static => return false,
          ClassMember::PrivateProp(property) if !property.is_static => return false,
          _ => {}
        }
      }

      for member in &class.body {
        if let ClassMember::Constructor(constructor) = member
          && let Some(body) = &constructor.body
          && !body.stmts.is_empty()
        {
          return false;
        }
      }

      true
    }
    _ => false,
  }
}

fn is_pure_string_method(method: &str) -> bool {
  matches!(
    method,
    "charAt"
      | "charCodeAt"
      | "concat"
      | "endsWith"
      | "includes"
      | "indexOf"
      | "lastIndexOf"
      | "localeCompare"
      | "slice"
      | "split"
      | "startsWith"
      | "substr"
      | "substring"
      | "toLocaleLowerCase"
      | "toLocaleUpperCase"
      | "toLowerCase"
      | "toString"
      | "toUpperCase"
      | "trim"
      | "trimEnd"
      | "trimStart"
  )
}
