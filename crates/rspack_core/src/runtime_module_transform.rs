use rspack_error::{Result, error};
use rspack_sources::{RawStringSource, ReplaceSource, Source};
use swc_core::{
  common::{FileName, GLOBALS, Globals, Mark, SourceMap, Span, Spanned, SyntaxContext},
  ecma::{
    ast::{
      AssignExpr, AssignTarget, Expr, Ident, Lit, MemberExpr, MemberProp, Prop, SimpleAssignTarget,
      UpdateExpr,
    },
    parser::{EsSyntax, Syntax, parse_file_as_program},
    transforms::base::resolver,
    visit::{Visit, VisitMutWith, VisitWith},
  },
};

use crate::{
  RuntimeGlobals, RuntimeGlobalsRenderMode, RuntimeModuleCustomSource, RuntimeVariable,
  property_access,
};

/// Runtime extensions use the same bindings as native runtime modules. Every
/// referenced built-in global is collected as writable by the metadata pass.
pub(crate) fn transform_runtime_module_source(
  source: String,
  render_mode: RuntimeGlobalsRenderMode,
) -> Result<RuntimeModuleCustomSource> {
  GLOBALS.set(&Globals::new(), || {
    let cm = SourceMap::default();
    let fm = cm.new_source_file(
      FileName::Custom("runtime module".into()).into(),
      source.clone(),
    );
    let mut errors = Vec::new();
    let mut program = parse_file_as_program(
      &fm,
      Syntax::Es(EsSyntax {
        allow_return_outside_function: true,
        ..Default::default()
      }),
      Default::default(),
      None,
      &mut errors,
    )
    .map_err(|e| error!("Cannot parse runtime module: {:?}", e.kind()))?;
    if let Some(e) = errors.first() {
      return Err(error!("Cannot parse runtime module: {:?}", e.kind()));
    }
    let unresolved_mark = Mark::new();
    program.visit_mut_with(&mut resolver(unresolved_mark, Mark::new(), false));
    let mut visitor = RuntimeModuleVisitor {
      source: ReplaceSource::new(RawStringSource::from(source.clone())),
      source_start: fm.start_pos.0,
      unresolved: SyntaxContext::empty().apply_mark(unresolved_mark),
      render_mode,
      context_require: RuntimeGlobalsRenderMode::RspackContext
        .render_runtime_globals(&RuntimeGlobals::REQUIRE),
      runtime_requirements: RuntimeGlobals::default(),
    };
    program.visit_with(&mut visitor);
    Ok(RuntimeModuleCustomSource {
      source: Some(source),
      transformed_source: Some(visitor.source.source().into_string_lossy().into_owned()),
      generated: false,
      runtime_requirements: visitor.runtime_requirements.with_require_scope(),
    })
  })
}

struct RuntimeModuleVisitor {
  source: ReplaceSource,
  source_start: u32,
  unresolved: SyntaxContext,
  render_mode: RuntimeGlobalsRenderMode,
  context_require: String,
  runtime_requirements: RuntimeGlobals,
}

fn static_property(property: &MemberProp) -> Option<String> {
  match property {
    MemberProp::Ident(property) => Some(property.sym.to_string()),
    MemberProp::Computed(property) => match property.expr.as_ref() {
      Expr::Lit(Lit::Str(property)) => Some(property.value.to_string_lossy().into_owned()),
      Expr::Tpl(template) if template.exprs.is_empty() => template.quasis[0]
        .cooked
        .as_ref()
        .map(|value| value.to_string_lossy().into_owned()),
      _ => None,
    },
    _ => None,
  }
}

impl RuntimeModuleVisitor {
  fn replace(&mut self, span: Span, replacement: &str) {
    self.source.replace(
      span.lo.0 - self.source_start,
      span.hi.0 - self.source_start,
      replacement.to_owned(),
      None,
    );
  }

  fn ident_global(&self, ident: &Ident) -> Option<RuntimeGlobals> {
    if ident.ctxt != self.unresolved {
      return None;
    }
    match ident.sym.as_ref() {
      "__webpack_require__" => Some(RuntimeGlobals::REQUIRE),
      "__rspack_context" => Some(RuntimeGlobals::REQUIRE_SCOPE),
      "__webpack_modules__" => Some(RuntimeGlobals::MODULE_FACTORIES),
      "__webpack_module_cache__" => Some(RuntimeGlobals::MODULE_CACHE),
      _ => None,
    }
  }

  fn is_context_require(&self, property: &str) -> bool {
    let context =
      RuntimeGlobalsRenderMode::RspackContext.render_runtime_variable(&RuntimeVariable::Context);
    format!("{context}{}", property_access([property], 0)) == self.context_require
  }

  fn runtime_member(&self, member: &MemberExpr) -> Option<bool> {
    match member.obj.as_ref() {
      Expr::Ident(ident) => match self.ident_global(ident)? {
        RuntimeGlobals::REQUIRE => Some(true),
        RuntimeGlobals::REQUIRE_SCOPE => Some(false),
        _ => None,
      },
      Expr::Member(base) if self.member_global(base) == Some(RuntimeGlobals::REQUIRE) => Some(true),
      _ => None,
    }
  }

  fn member_global(&self, member: &MemberExpr) -> Option<RuntimeGlobals> {
    let legacy = self.runtime_member(member)?;
    let property = static_property(&member.prop)?;
    if legacy {
      RuntimeGlobals::from_property_name(&property)
    } else if self.is_context_require(&property) {
      Some(RuntimeGlobals::REQUIRE)
    } else {
      RuntimeGlobals::from_rspack_context_property_name(&property)
    }
  }

  fn expr_global(&self, expr: &Expr) -> Option<RuntimeGlobals> {
    match expr {
      Expr::Ident(ident) => self.ident_global(ident),
      Expr::Member(member) => self.member_global(member),
      Expr::Paren(paren) => self.expr_global(&paren.expr),
      _ => None,
    }
  }

  fn render_global(&mut self, global: RuntimeGlobals) -> String {
    self.runtime_requirements.insert(global);
    self.render_mode.render_runtime_globals(&global)
  }

  fn export_setter(&mut self, global: RuntimeGlobals) -> Option<String> {
    if self.render_mode != RuntimeGlobalsRenderMode::RspackExport
      || global == RuntimeGlobals::REQUIRE_SCOPE
    {
      return None;
    }
    self.runtime_requirements.insert(global);
    global.to_rspack_export_setter_name()
  }
}

impl Visit for RuntimeModuleVisitor {
  fn visit_member_expr(&mut self, member: &MemberExpr) {
    if let Some(global) = self.member_global(member) {
      let replacement = self.render_global(global);
      self.replace(member.span(), &replacement);
      return;
    }
    let Some(legacy) = self.runtime_member(member) else {
      member.visit_children_with(self);
      return;
    };
    let property = static_property(&member.prop);
    let global = if legacy
      && property
        .as_deref()
        .is_none_or(|property| matches!(property, "call" | "apply" | "bind"))
    {
      RuntimeGlobals::REQUIRE
    } else {
      RuntimeGlobals::REQUIRE_SCOPE
    };
    let replacement = self.render_global(global);
    self.replace(member.obj.span(), &replacement);
    member.prop.visit_with(self);
  }

  fn visit_ident(&mut self, ident: &Ident) {
    if let Some(global) = self.ident_global(ident) {
      let replacement = self.render_global(global);
      self.replace(ident.span, &replacement);
    }
  }

  fn visit_prop(&mut self, prop: &Prop) {
    if let Prop::Shorthand(ident) = prop
      && let Some(global) = self.ident_global(ident)
    {
      let replacement = self.render_global(global);
      self.replace(ident.span, &format!("{}: {replacement}", ident.sym));
    } else {
      prop.visit_children_with(self);
    }
  }

  fn visit_assign_expr(&mut self, assignment: &AssignExpr) {
    let global = match &assignment.left {
      AssignTarget::Simple(SimpleAssignTarget::Ident(ident)) => self.ident_global(&ident.id),
      AssignTarget::Simple(SimpleAssignTarget::Member(member)) => self.member_global(member),
      AssignTarget::Simple(SimpleAssignTarget::Paren(paren)) => self.expr_global(&paren.expr),
      _ => None,
    };
    let Some((global, setter)) =
      global.and_then(|global| self.export_setter(global).map(|setter| (global, setter)))
    else {
      assignment.visit_children_with(self);
      return;
    };
    let binding = self.render_global(global);
    let (prefix, suffix) = if let Some(operation) = assignment.op.to_update() {
      if assignment.op.may_short_circuit() {
        (format!("({binding} {operation} {setter}("), "))")
      } else {
        (format!("({setter}({binding} {operation} ("), ")))")
      }
    } else {
      (format!("({setter}("), "))")
    };
    self.replace(
      Span::new(assignment.span.lo, assignment.right.span().lo),
      &prefix,
    );
    assignment.right.visit_with(self);
    self.replace(
      Span::new(assignment.right.span().hi, assignment.span.hi),
      suffix,
    );
  }

  fn visit_update_expr(&mut self, update: &UpdateExpr) {
    let Some((global, setter)) = self
      .expr_global(&update.arg)
      .and_then(|global| self.export_setter(global).map(|setter| (global, setter)))
    else {
      update.visit_children_with(self);
      return;
    };
    let binding = self.render_global(global);
    let operation = if update.prefix {
      format!("{}value", update.op)
    } else {
      format!("value{}", update.op)
    };
    self.replace(update.span, &format!("(function(value) {{ var result = {operation}; {setter}(value); return result; }})({binding})"));
  }
}
