use rspack_core::{
  ChunkCodeTemplate, MODULE_GLOBALS, RuntimeGlobals, RuntimeVariable, property_access,
  runtime_variable_name,
};
use rspack_error::{Result, error};
use rspack_util::SpanExt;
use swc_experimental_allocator::Allocator;
use swc_experimental_ecma_ast::{
  AssignExpr, AssignTarget, CallExpr, Callee, Expr, GetSpan, Ident, MemberExpr, MemberProp,
  SimpleAssignTarget, UnaryExpr, UnaryOp, UpdateExpr, Visit, VisitWith,
};
use swc_experimental_ecma_parser::{EsSyntax, Lexer, Parser, StringSource, Syntax};
use swc_experimental_ecma_semantic::resolver::{Semantic, resolver};

pub(crate) struct CustomRuntimeModuleCompat {
  pub source: String,
  pub runtime_requirements: RuntimeGlobals,
  pub write_runtime_requirements: RuntimeGlobals,
}

pub(crate) fn replace_custom_runtime_module_compat(
  source: String,
  runtime_template: &ChunkCodeTemplate,
) -> Result<CustomRuntimeModuleCompat> {
  let table = RuntimeCompatTable::new(runtime_template);
  let mut replacements = {
    let allocator = Allocator::new();
    let lexer = Lexer::new(
      &allocator,
      Syntax::Es(EsSyntax {
        allow_return_outside_function: true,
        ..Default::default()
      }),
      swc_experimental_ecma_ast::EsVersion::EsNext,
      StringSource::new(source.as_ref()),
      None,
    );
    let mut parser = Parser::new_from(&allocator, lexer);
    let program = parser.parse_program().map_err(|e| {
      let mut errors = parser.take_errors();
      errors.push(e);
      error!("Failed to parse custom runtime module source: {errors:?}")
    })?;

    let parse_errors = parser.take_errors();
    if !parse_errors.is_empty() {
      return Err(error!(
        "Failed to parse custom runtime module source: {parse_errors:?}"
      ));
    }

    let semantic = resolver(&program);
    let mut collector = RuntimeCompatCollector {
      semantic: &semantic,
      source: &source,
      table: &table,
      replacements: Vec::new(),
      runtime_requirements: RuntimeGlobals::default(),
      write_runtime_requirements: RuntimeGlobals::default(),
    };
    program.visit_with(&mut collector);
    (
      collector.replacements,
      collector.runtime_requirements,
      collector.write_runtime_requirements,
    )
  };

  let mut result = source;
  replacements
    .0
    .sort_unstable_by(|a, b| b.start.cmp(&a.start));
  for replacement in replacements.0 {
    result.replace_range(replacement.start..replacement.end, &replacement.value);
  }

  Ok(CustomRuntimeModuleCompat {
    source: result,
    runtime_requirements: replacements.1,
    write_runtime_requirements: replacements.2,
  })
}

struct RuntimeCompatTable {
  require_scope: String,
  require_sources: Vec<String>,
  runtime_variables: Vec<RuntimeVariableCompat>,
  runtime_globals: Vec<RuntimeGlobalCompat>,
}

struct RuntimeVariableCompat {
  source: &'static str,
  target: String,
  runtime_global: Option<RuntimeGlobals>,
}

struct RuntimeGlobalCompat {
  runtime_global: RuntimeGlobals,
  sources: Vec<String>,
  target: String,
}

impl RuntimeCompatTable {
  fn new(runtime_template: &ChunkCodeTemplate) -> Self {
    let require_scope = runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE_SCOPE);
    let mut runtime_variables = Vec::new();
    for (runtime_variable, runtime_global) in [
      (RuntimeVariable::Require, Some(RuntimeGlobals::REQUIRE)),
      (
        RuntimeVariable::Context,
        Some(RuntimeGlobals::REQUIRE_SCOPE),
      ),
      (
        RuntimeVariable::Modules,
        Some(RuntimeGlobals::MODULE_FACTORIES),
      ),
      (
        RuntimeVariable::ModuleCache,
        Some(RuntimeGlobals::MODULE_CACHE),
      ),
      (RuntimeVariable::Module, Some(RuntimeGlobals::MODULE)),
      (RuntimeVariable::Exports, Some(RuntimeGlobals::EXPORTS)),
      (RuntimeVariable::StartupExec, None),
    ] {
      runtime_variables.push(RuntimeVariableCompat {
        source: runtime_variable_name(&runtime_variable),
        target: runtime_template.render_runtime_variable(&runtime_variable),
        runtime_global,
      });
    }

    let mut runtime_globals = Vec::new();
    for (_, runtime_global) in RuntimeGlobals::all().iter_names() {
      let Some(target) = render_runtime_global(runtime_template, runtime_global) else {
        continue;
      };
      let mut sources = vec![render_webpack_runtime_global(runtime_global)];
      let current_rendered = runtime_template.render_runtime_globals(&runtime_global);
      if !sources.iter().any(|source| source == &current_rendered) {
        sources.push(current_rendered);
      }
      runtime_globals.push(RuntimeGlobalCompat {
        runtime_global,
        sources,
        target,
      });
    }

    let require_sources = runtime_globals
      .iter()
      .find(|item| item.runtime_global == RuntimeGlobals::REQUIRE)
      .map(|item| item.sources.clone())
      .unwrap_or_else(|| vec![runtime_variable_name(&RuntimeVariable::Require).to_string()]);

    Self {
      require_scope,
      require_sources,
      runtime_variables,
      runtime_globals,
    }
  }

  fn runtime_variable(&self, name: &str) -> Option<&RuntimeVariableCompat> {
    self
      .runtime_variables
      .iter()
      .find(|item| item.source == name)
  }

  fn runtime_global_by_source(&self, source: &str) -> Option<&RuntimeGlobalCompat> {
    self
      .runtime_globals
      .iter()
      .find(|item| item.sources.iter().any(|candidate| candidate == source))
  }

  fn runtime_global_by_property(&self, property: &str) -> Option<&RuntimeGlobalCompat> {
    let runtime_global = RuntimeGlobals::from_property_name(property)?;
    self
      .runtime_globals
      .iter()
      .find(|item| item.runtime_global == runtime_global)
  }

  fn is_require_source(&self, source: &str) -> bool {
    self
      .require_sources
      .iter()
      .any(|candidate| candidate == source)
  }
}

fn render_webpack_runtime_global(runtime_global: RuntimeGlobals) -> String {
  if runtime_global == RuntimeGlobals::EXPORTS {
    return runtime_variable_name(&RuntimeVariable::Exports).to_string();
  }
  if runtime_global == RuntimeGlobals::REQUIRE {
    return runtime_variable_name(&RuntimeVariable::Require).to_string();
  }
  if runtime_global == RuntimeGlobals::MODULE {
    return "module".to_string();
  }
  if runtime_global == RuntimeGlobals::REQUIRE_SCOPE {
    return format!("{}.*", runtime_variable_name(&RuntimeVariable::Require));
  }
  let name = runtime_global
    .property_name()
    .expect("runtime global should have a property name");
  if runtime_global.renderable_require_scope() == runtime_global {
    return format!(
      "{}{}",
      runtime_variable_name(&RuntimeVariable::Require),
      property_access([name], 0)
    );
  }
  if MODULE_GLOBALS.contains(runtime_global) {
    return format!("module{}", property_access([name], 0));
  }
  name.to_string()
}

fn render_runtime_global(
  runtime_template: &ChunkCodeTemplate,
  runtime_global: RuntimeGlobals,
) -> Option<String> {
  if runtime_global == RuntimeGlobals::REQUIRE {
    return Some(runtime_template.render_runtime_variable(&RuntimeVariable::Require));
  }
  if runtime_global == RuntimeGlobals::REQUIRE_SCOPE {
    return Some(runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE_SCOPE));
  }
  if runtime_global == RuntimeGlobals::MODULE_FACTORIES {
    return Some(runtime_template.render_runtime_variable(&RuntimeVariable::Modules));
  }
  if runtime_global == RuntimeGlobals::EXPORTS {
    return Some(runtime_template.render_runtime_variable(&RuntimeVariable::Exports));
  }
  if runtime_global == RuntimeGlobals::MODULE {
    return Some(runtime_template.render_runtime_variable(&RuntimeVariable::Module));
  }
  if runtime_global.renderable_require_scope() == runtime_global {
    return runtime_global
      .to_lexical_name()
      .map(str::to_string)
      .or_else(|| Some(runtime_template.render_runtime_globals(&runtime_global)));
  }
  Some(runtime_template.render_runtime_globals(&runtime_global))
}

struct RuntimeCompatCollector<'a> {
  semantic: &'a Semantic,
  source: &'a str,
  table: &'a RuntimeCompatTable,
  replacements: Vec<RuntimeCompatReplacement>,
  runtime_requirements: RuntimeGlobals,
  write_runtime_requirements: RuntimeGlobals,
}

struct RuntimeCompatReplacement {
  start: usize,
  end: usize,
  value: String,
}

#[derive(Clone, Copy)]
enum RuntimeUsage {
  Read,
  Write,
}

impl<'a> RuntimeCompatCollector<'a> {
  fn collect_ident(&mut self, ident: &Ident<'a>, usage: RuntimeUsage) {
    let Some(runtime_variable) = self.table.runtime_variable(ident.sym.as_str()) else {
      return;
    };
    if self.semantic.node_scope(ident) != self.semantic.unresolved_scope_id() {
      return;
    }
    self.collect_runtime_requirement(runtime_variable.runtime_global, usage);
    self.replacements.push(RuntimeCompatReplacement {
      start: ident.span.real_lo() as usize,
      end: ident.span.real_hi() as usize,
      value: runtime_variable.target.clone(),
    });
  }

  fn collect_member_expr(&mut self, expr: &MemberExpr<'a>, usage: RuntimeUsage) -> bool {
    let Some(object_source) = self.expr_source(&expr.obj) else {
      return false;
    };
    if !self.table.is_require_source(object_source) {
      return false;
    }
    let runtime_global = self
      .member_property_name(&expr.prop)
      .and_then(|property| self.table.runtime_global_by_property(property));
    let value = runtime_global.map(|item| item.target.clone()).or_else(|| {
      self
        .member_property_access(&expr.prop)
        .map(|property_access| format!("{}{}", self.table.require_scope, property_access))
    });
    let Some(value) = value else {
      return false;
    };
    self.collect_runtime_requirement(runtime_global.map(|item| item.runtime_global), usage);
    self.replacements.push(RuntimeCompatReplacement {
      start: expr.span.real_lo() as usize,
      end: expr.span.real_hi() as usize,
      value,
    });
    true
  }

  fn collect_expr_by_source(&mut self, expr: &Expr<'a>, usage: RuntimeUsage) -> bool {
    let Some(source) = self.expr_source(expr) else {
      return false;
    };
    if let Some(runtime_global) = self.table.runtime_global_by_source(source) {
      self.collect_runtime_requirement(Some(runtime_global.runtime_global), usage);
      self.replacements.push(RuntimeCompatReplacement {
        start: expr.span().real_lo() as usize,
        end: expr.span().real_hi() as usize,
        value: runtime_global.target.clone(),
      });
      return true;
    }
    false
  }

  fn collect_member_expr_by_source(&mut self, expr: &MemberExpr<'a>, usage: RuntimeUsage) -> bool {
    let start = expr.span.real_lo() as usize;
    let end = expr.span.real_hi() as usize;
    let Some(source) = self.source.get(start..end) else {
      return false;
    };
    if let Some(runtime_global) = self.table.runtime_global_by_source(source) {
      self.collect_runtime_requirement(Some(runtime_global.runtime_global), usage);
      self.replacements.push(RuntimeCompatReplacement {
        start,
        end,
        value: runtime_global.target.clone(),
      });
      return true;
    }
    false
  }

  fn collect_runtime_requirement(
    &mut self,
    runtime_global: Option<RuntimeGlobals>,
    usage: RuntimeUsage,
  ) {
    let Some(runtime_global) = runtime_global else {
      return;
    };
    self.runtime_requirements.insert(runtime_global);
    if matches!(usage, RuntimeUsage::Write) {
      self.write_runtime_requirements.insert(runtime_global);
    }
  }

  fn expr_source(&self, expr: &Expr<'a>) -> Option<&str> {
    let start = expr.span().real_lo() as usize;
    let end = expr.span().real_hi() as usize;
    self.source.get(start..end)
  }

  fn member_property_name<'b>(&self, prop: &'b MemberProp<'a>) -> Option<&'b str> {
    match prop {
      MemberProp::Ident(ident) => Some(ident.sym.as_str()),
      MemberProp::Computed(computed) => match &computed.expr {
        Expr::Lit(lit) => lit.as_str().and_then(|value| value.value.as_str()),
        _ => None,
      },
      MemberProp::PrivateName(_) => None,
    }
  }

  fn member_property_access(&self, prop: &MemberProp<'a>) -> Option<String> {
    match prop {
      MemberProp::Ident(ident) => Some(property_access([ident.sym.as_str()], 0)),
      MemberProp::Computed(_) => {
        let start = prop.span().real_lo() as usize;
        let end = prop.span().real_hi() as usize;
        self.source.get(start..end).map(str::to_string)
      }
      MemberProp::PrivateName(_) => None,
    }
  }

  fn visit_assignment_target(&mut self, target: &AssignTarget<'a>) {
    match target {
      AssignTarget::Simple(target) => match target.as_ref() {
        SimpleAssignTarget::Ident(ident) => {
          self.collect_ident(&ident.id, RuntimeUsage::Write);
        }
        SimpleAssignTarget::Member(expr) => {
          if !self.collect_member_expr(expr, RuntimeUsage::Write)
            && !self.collect_member_expr_by_source(expr, RuntimeUsage::Write)
          {
            expr.visit_children_with(self);
          }
        }
        _ => target.visit_with(self),
      },
      _ => target.visit_with(self),
    }
  }
}

impl<'a> Visit<'a> for RuntimeCompatCollector<'a> {
  fn visit_member_expr(&mut self, expr: &MemberExpr<'a>) {
    if self.collect_member_expr(expr, RuntimeUsage::Read) {
      return;
    }
    expr.visit_children_with(self);
  }

  fn visit_assign_expr(&mut self, expr: &AssignExpr<'a>) {
    self.visit_assignment_target(&expr.left);
    expr.right.visit_with(self);
  }

  fn visit_update_expr(&mut self, expr: &UpdateExpr<'a>) {
    match &expr.arg {
      Expr::Ident(ident) => self.collect_ident(ident, RuntimeUsage::Write),
      Expr::Member(member) => {
        if !self.collect_member_expr(member, RuntimeUsage::Write)
          && !self.collect_member_expr_by_source(member, RuntimeUsage::Write)
        {
          expr.arg.visit_with(self);
        }
      }
      _ => expr.arg.visit_with(self),
    }
  }

  fn visit_unary_expr(&mut self, expr: &UnaryExpr<'a>) {
    if expr.op == UnaryOp::Delete {
      match &expr.arg {
        Expr::Ident(ident) => {
          self.collect_ident(ident, RuntimeUsage::Write);
          return;
        }
        Expr::Member(member) => {
          if self.collect_member_expr(member, RuntimeUsage::Write)
            || self.collect_member_expr_by_source(member, RuntimeUsage::Write)
          {
            return;
          }
        }
        _ => {}
      }
    }
    expr.arg.visit_with(self);
  }

  fn visit_call_expr(&mut self, expr: &CallExpr<'a>) {
    if let Callee::Expr(callee) = &expr.callee
      && self.collect_expr_by_source(callee, RuntimeUsage::Read)
    {
      for arg in &expr.args {
        arg.visit_with(self);
      }
      return;
    }
    expr.visit_children_with(self);
  }

  fn visit_expr(&mut self, expr: &Expr<'a>) {
    if let Expr::Ident(ident) = expr {
      self.collect_ident(ident, RuntimeUsage::Read);
      return;
    }
    if self.collect_expr_by_source(expr, RuntimeUsage::Read) {
      return;
    }
    expr.visit_children_with(self);
  }
}
