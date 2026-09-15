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

pub(super) fn may_have_side_effects(expression: &Expr<'_>, ctx: SideEffectsContext<'_>) -> bool {
  let Some(ctx) = ctx.consume_depth() else {
    return true;
  };

  if is_pure_callee(expression, ctx) {
    return false;
  }

  match expression {
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
    Expr::Paren(parenthesized) => may_have_side_effects(&parenthesized.expr, ctx),
    Expr::Fn(..) | Expr::Arrow(..) => false,
    Expr::Class(class) => class_has_side_effects(ctx, &class.class),
    Expr::Array(array) => array
      .elems
      .iter()
      .flatten()
      .any(|element| element.spread.is_some() || may_have_side_effects(&element.expr, ctx)),
    Expr::Unary(unary) => match unary.op {
      UnaryOp::Delete => true,
      _ => may_have_side_effects(&unary.arg, ctx),
    },
    Expr::Bin(binary) => {
      may_have_side_effects(&binary.left, ctx) || may_have_side_effects(&binary.right, ctx)
    }
    Expr::Member(member)
      if matches!(
        &member.obj,
        Expr::Object(_) | Expr::Fn(_) | Expr::Arrow(_) | Expr::Class(_)
      ) =>
    {
      let object = &member.obj;
      if may_have_side_effects(object, ctx) {
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
        MemberProp::Computed(computed) => may_have_side_effects(&computed.expr, ctx),
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
          .any(|argument| may_have_side_effects(&argument.expr, ctx))
      })
    }
    Expr::New(_) => true,
    Expr::Call(call_expression) => {
      let Callee::Expr(callee) = &call_expression.callee else {
        return true;
      };

      if is_pure_callee(callee, ctx) {
        call_expression
          .args
          .iter()
          .any(|argument| may_have_side_effects(&argument.expr, ctx))
      } else {
        true
      }
    }
    Expr::OptChain(optional_chain) => match &optional_chain.base {
      OptChainBase::Call(call) if is_pure_callee(&call.callee, ctx) => call
        .args
        .iter()
        .any(|argument| may_have_side_effects(&argument.expr, ctx)),
      _ => true,
    },
    Expr::Seq(sequence) => sequence
      .exprs
      .iter()
      .any(|expression| may_have_side_effects(expression, ctx)),
    Expr::Cond(conditional) => {
      may_have_side_effects(&conditional.test, ctx)
        || may_have_side_effects(&conditional.cons, ctx)
        || may_have_side_effects(&conditional.alt, ctx)
    }
    Expr::Object(object) => object.props.iter().any(|property| match property {
      PropOrSpread::Prop(property) => match &**property {
        Prop::Shorthand(..) => false,
        Prop::KeyValue(key_value) => {
          let key_has_side_effects = match &key_value.key {
            PropName::Computed(computed) => may_have_side_effects(&computed.expr, ctx),
            _ => false,
          };
          key_has_side_effects || may_have_side_effects(&key_value.value, ctx)
        }
        Prop::Getter(getter) => match &getter.key {
          PropName::Computed(computed) => may_have_side_effects(&computed.expr, ctx),
          _ => false,
        },
        Prop::Setter(setter) => match &setter.key {
          PropName::Computed(computed) => may_have_side_effects(&computed.expr, ctx),
          _ => false,
        },
        Prop::Method(method) => match &method.key {
          PropName::Computed(computed) => may_have_side_effects(&computed.expr, ctx),
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

fn is_pure_callee(expression: &Expr<'_>, ctx: SideEffectsContext<'_>) -> bool {
  if is_global_ref_to(expression, ctx, "Date") {
    return true;
  }

  match expression {
    Expr::Member(member) => {
      let object = &member.obj;
      let property = &member.prop;

      if let MemberProp::Ident(property) = property {
        if is_global_ref_to(object, ctx, "Math") {
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

fn is_global_ref_to(expression: &Expr<'_>, ctx: SideEffectsContext<'_>, id: &str) -> bool {
  match expression {
    Expr::Ident(identifier) => {
      ctx.semantic.node_scope(identifier) == ctx.semantic.unresolved_scope_id()
        && identifier.sym == id
    }
    _ => false,
  }
}

fn statement_may_have_side_effects(statement: &Stmt<'_>, ctx: SideEffectsContext<'_>) -> bool {
  match statement {
    Stmt::Block(block) => block
      .stmts
      .iter()
      .any(|statement| statement_may_have_side_effects(statement, ctx)),
    Stmt::Empty(_) => false,
    Stmt::Labeled(labeled) => statement_may_have_side_effects(&labeled.body, ctx),
    Stmt::If(if_statement) => {
      may_have_side_effects(&if_statement.test, ctx)
        || statement_may_have_side_effects(&if_statement.cons, ctx)
        || if_statement
          .alt
          .as_ref()
          .is_some_and(|statement| statement_may_have_side_effects(statement, ctx))
    }
    Stmt::Switch(switch) => {
      may_have_side_effects(&switch.discriminant, ctx)
        || switch.cases.iter().any(|case| {
          case
            .test
            .as_ref()
            .is_some_and(|expression| may_have_side_effects(expression, ctx))
            || case
              .cons
              .iter()
              .any(|statement| statement_may_have_side_effects(statement, ctx))
        })
    }
    Stmt::Try(try_statement) => {
      try_statement
        .block
        .stmts
        .iter()
        .any(|statement| statement_may_have_side_effects(statement, ctx))
        || try_statement.handler.as_ref().is_some_and(|handler| {
          handler
            .body
            .stmts
            .iter()
            .any(|statement| statement_may_have_side_effects(statement, ctx))
        })
        || try_statement.finalizer.as_ref().is_some_and(|finalizer| {
          finalizer
            .stmts
            .iter()
            .any(|statement| statement_may_have_side_effects(statement, ctx))
        })
    }
    Stmt::Decl(declaration) => match &**declaration {
      Decl::Class(class) => class_has_side_effects(ctx, &class.class),
      Decl::Fn(_) => !ctx.in_strict,
      Decl::Var(variable) => variable.kind == VarDeclKind::Var,
      _ => false,
    },
    Stmt::Expr(expression) => may_have_side_effects(&expression.expr, ctx),
    _ => true,
  }
}

fn class_has_side_effects(ctx: SideEffectsContext<'_>, class: &Class<'_>) -> bool {
  if let Some(super_class) = &class.super_class
    && may_have_side_effects(super_class, ctx)
  {
    return true;
  }

  for member in &class.body {
    match member {
      ClassMember::Method(method) => {
        if let PropName::Computed(key) = &method.key
          && may_have_side_effects(&key.expr, ctx)
        {
          return true;
        }
      }
      ClassMember::ClassProp(property) => {
        if let PropName::Computed(key) = &property.key
          && may_have_side_effects(&key.expr, ctx)
        {
          return true;
        }
        if let Some(value) = &property.value
          && may_have_side_effects(value, ctx)
        {
          return true;
        }
      }
      ClassMember::PrivateProp(property) => {
        if let Some(value) = &property.value
          && may_have_side_effects(value, ctx)
        {
          return true;
        }
      }
      ClassMember::StaticBlock(block)
        if block
          .body
          .stmts
          .iter()
          .any(|statement| statement_may_have_side_effects(statement, ctx)) =>
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
