use std::borrow::Cow;

use rspack_core::{
  DependencyRange, InitFragmentExt, InitFragmentKey, InitFragmentStage, NormalInitFragment,
  RuntimeGlobals, TemplateContext, TemplateReplaceSource,
};
use rustc_hash::FxHashSet;
use swc_atoms::Atom;

#[derive(Debug, Clone)]
pub enum AstDependencySideEffect {
  RuntimeRequirements(RuntimeGlobals),
  CachedConst {
    identifier: Box<str>,
    content: Box<str>,
  },
  CommonJsExportsVar(Box<str>),
  ProvidedDependency {
    identifier: Box<str>,
    expression: Box<str>,
  },
}

impl AstDependencySideEffect {
  pub fn apply(&self, context: &mut TemplateContext<'_, '_, '_>) {
    match self {
      Self::RuntimeRequirements(runtime_requirements) => {
        context
          .runtime_template
          .runtime_requirements_mut()
          .insert(*runtime_requirements);
      }
      Self::CachedConst {
        identifier,
        content,
      } => context.init_fragments.push(
        NormalInitFragment::new(
          format!("var {identifier} = {content};\n"),
          InitFragmentStage::StageConstants,
          0,
          InitFragmentKey::Const(identifier.to_string()),
          None,
        )
        .boxed(),
      ),
      Self::CommonJsExportsVar(identifier) => context.init_fragments.push(
        NormalInitFragment::new(
          format!("var {};\n", identifier.as_ref()),
          InitFragmentStage::StageConstants,
          0,
          InitFragmentKey::CommonJsExports(identifier.to_string()),
          None,
        )
        .boxed(),
      ),
      Self::ProvidedDependency {
        identifier,
        expression,
      } => context.init_fragments.push(
        NormalInitFragment::new(
          format!(
            "/* provided dependency */ var {} = {};\n",
            identifier.as_ref(),
            expression.as_ref()
          ),
          InitFragmentStage::StageProvides,
          1,
          InitFragmentKey::ModuleExternal(format!("provided {}", identifier.as_ref())),
          None,
        )
        .with_top_level_decl_symbols(vec![Atom::from(identifier.as_ref())])
        .boxed(),
      ),
    }
  }
}

#[derive(Debug, Clone)]
pub struct AstDependencyAction {
  range: DependencyRange,
  replacement: AstDependencyReplacement,
}

#[derive(Debug, Clone)]
enum AstDependencyReplacement {
  Generated(Box<str>),
  RawExpr(Box<str>),
  RawIdent(Box<str>),
  RawIdentWithSuffix(Box<str>),
  Insert {
    position: u32,
    content: Box<str>,
  },
  WrappedSource {
    replacement: DependencyRange,
    prefix: Box<str>,
    suffix: Box<str>,
  },
  WrappedSourceWithReplacements {
    replacement: DependencyRange,
    prefix: Box<str>,
    suffix: Box<str>,
    replacements: Vec<(Box<str>, u32, u32)>,
  },
  WrappedSourceTrimTrailingSemicolon {
    prefix: Box<str>,
    suffix: Box<str>,
  },
  RangeReplacements {
    replacements: Vec<(Box<str>, u32, u32)>,
  },
  SourceWithReplacements {
    replacement: DependencyRange,
    replacements: Vec<(Box<str>, u32, u32)>,
  },
}

impl AstDependencyAction {
  pub fn expr(range: DependencyRange, content: impl Into<Box<str>>) -> Option<Self> {
    if range.start > range.end {
      return None;
    }

    Some(Self {
      range,
      replacement: AstDependencyReplacement::Generated(content.into()),
    })
  }

  pub fn raw_expr(range: DependencyRange, content: impl Into<Box<str>>) -> Option<Self> {
    if range.start > range.end {
      return None;
    }

    Some(Self {
      range,
      replacement: AstDependencyReplacement::RawExpr(content.into()),
    })
  }

  pub fn raw_ident(range: DependencyRange, content: impl Into<Box<str>>) -> Option<Self> {
    if range.start > range.end {
      return None;
    }

    Some(Self {
      range,
      replacement: AstDependencyReplacement::RawIdent(content.into()),
    })
  }

  pub fn raw_ident_with_suffix(
    range: DependencyRange,
    suffix: impl Into<Box<str>>,
  ) -> Option<Self> {
    if range.start > range.end {
      return None;
    }

    Some(Self {
      range,
      replacement: AstDependencyReplacement::RawIdentWithSuffix(suffix.into()),
    })
  }

  pub fn insert(
    range: DependencyRange,
    position: u32,
    content: impl Into<Box<str>>,
  ) -> Option<Self> {
    if range.start > range.end {
      return None;
    }

    Some(Self {
      range,
      replacement: AstDependencyReplacement::Insert {
        position,
        content: content.into(),
      },
    })
  }

  pub fn wrapped_source(
    range: DependencyRange,
    replacement: DependencyRange,
    prefix: impl Into<Box<str>>,
    suffix: impl Into<Box<str>>,
  ) -> Option<Self> {
    if range.start > range.end || replacement.start > replacement.end {
      return None;
    }

    Some(Self {
      range,
      replacement: AstDependencyReplacement::WrappedSource {
        replacement,
        prefix: prefix.into(),
        suffix: suffix.into(),
      },
    })
  }

  pub fn wrapped_source_with_replacements(
    range: DependencyRange,
    replacement: DependencyRange,
    prefix: impl Into<Box<str>>,
    suffix: impl Into<Box<str>>,
    replacements: Vec<(String, u32, u32)>,
  ) -> Option<Self> {
    if range.start > range.end || replacement.start > replacement.end {
      return None;
    }

    Some(Self {
      range,
      replacement: AstDependencyReplacement::WrappedSourceWithReplacements {
        replacement,
        prefix: prefix.into(),
        suffix: suffix.into(),
        replacements: replacements
          .into_iter()
          .map(|(content, start, end)| (content.into_boxed_str(), start, end))
          .collect(),
      },
    })
  }

  pub fn wrapped_source_trim_trailing_semicolon(
    range: DependencyRange,
    prefix: impl Into<Box<str>>,
    suffix: impl Into<Box<str>>,
  ) -> Option<Self> {
    if range.start > range.end {
      return None;
    }

    Some(Self {
      range,
      replacement: AstDependencyReplacement::WrappedSourceTrimTrailingSemicolon {
        prefix: prefix.into(),
        suffix: suffix.into(),
      },
    })
  }

  pub fn source_with_replacements(
    range: DependencyRange,
    replacement: DependencyRange,
    replacements: Vec<(String, u32, u32)>,
  ) -> Option<Self> {
    if range.start > range.end
      || replacement.start > replacement.end
      || replacement.start < range.start
      || replacement.end > range.end
    {
      return None;
    }

    Some(Self {
      range,
      replacement: AstDependencyReplacement::SourceWithReplacements {
        replacement,
        replacements: replacements
          .into_iter()
          .map(|(content, start, end)| (content.into_boxed_str(), start, end))
          .collect(),
      },
    })
  }

  pub fn range_replacements(
    range: DependencyRange,
    replacements: Vec<(String, u32, u32)>,
  ) -> Option<Self> {
    if range.start > range.end {
      return None;
    }

    Some(Self {
      range,
      replacement: AstDependencyReplacement::RangeReplacements {
        replacements: replacements
          .into_iter()
          .map(|(content, start, end)| (content.into_boxed_str(), start, end))
          .collect(),
      },
    })
  }

  fn replacement_content<'a>(&'a self, source_text: &'a str) -> Cow<'a, str> {
    match &self.replacement {
      AstDependencyReplacement::Generated(content) => Cow::Borrowed(content.as_ref()),
      AstDependencyReplacement::RawExpr(content) => Cow::Borrowed(content.as_ref()),
      AstDependencyReplacement::RawIdent(content) => Cow::Borrowed(content.as_ref()),
      AstDependencyReplacement::RawIdentWithSuffix(suffix) => {
        let source = &source_text[self.range.start as usize..self.range.end as usize];
        Cow::Owned(format!("{source}{suffix}"))
      }
      AstDependencyReplacement::Insert { content, .. } => Cow::Borrowed(content.as_ref()),
      AstDependencyReplacement::WrappedSource {
        replacement,
        prefix,
        suffix,
      } => {
        let source = &source_text[replacement.start as usize..replacement.end as usize];
        Cow::Owned(format!("{prefix}{source}{suffix}"))
      }
      AstDependencyReplacement::WrappedSourceWithReplacements {
        replacement,
        prefix,
        suffix,
        replacements,
      } => {
        let source = apply_inner_replacements(source_text, *replacement, replacements);
        Cow::Owned(format!("{prefix}{source}{suffix}"))
      }
      AstDependencyReplacement::WrappedSourceTrimTrailingSemicolon { .. } => {
        unreachable!("trimmed wrapped source is expanded by source_replacements")
      }
      AstDependencyReplacement::RangeReplacements { .. } => {
        unreachable!("range replacements are expanded by source_replacements")
      }
      AstDependencyReplacement::SourceWithReplacements {
        replacement,
        replacements,
      } => {
        let prefix = &source_text[self.range.start as usize..replacement.start as usize];
        let suffix = &source_text[replacement.end as usize..self.range.end as usize];
        let source = apply_inner_replacements(source_text, *replacement, replacements);
        Cow::Owned(format!("{prefix}{source}{suffix}"))
      }
    }
  }

  fn edit_range(&self) -> DependencyRange {
    match &self.replacement {
      AstDependencyReplacement::Insert { position, .. } => {
        DependencyRange::new(*position, *position)
      }
      AstDependencyReplacement::RangeReplacements { .. } => self.range,
      AstDependencyReplacement::WrappedSourceTrimTrailingSemicolon { .. } => self.range,
      _ => self.range,
    }
  }

  fn source_replacement_ranges(&self) -> Vec<DependencyRange> {
    match &self.replacement {
      AstDependencyReplacement::WrappedSource { replacement, .. } => vec![
        DependencyRange::new(self.range.start, replacement.start),
        DependencyRange::new(replacement.end, self.range.end),
      ],
      AstDependencyReplacement::WrappedSourceWithReplacements {
        replacement,
        replacements,
        ..
      } => {
        let mut ranges = vec![
          DependencyRange::new(self.range.start, replacement.start),
          DependencyRange::new(replacement.end, self.range.end),
        ];
        ranges.extend(
          replacements
            .iter()
            .map(|(_, start, end)| DependencyRange::new(*start, *end)),
        );
        ranges
      }
      AstDependencyReplacement::RangeReplacements { replacements }
      | AstDependencyReplacement::SourceWithReplacements { replacements, .. } => replacements
        .iter()
        .map(|(_, start, end)| DependencyRange::new(*start, *end))
        .collect(),
      AstDependencyReplacement::WrappedSourceTrimTrailingSemicolon { .. } => vec![
        DependencyRange::new(self.range.start, self.range.start),
        DependencyRange::new(self.range.end, self.range.end),
      ],
      _ => vec![self.edit_range()],
    }
  }

  fn is_redundant_range_replacement(&self, existing_ranges: &FxHashSet<DependencyRange>) -> bool {
    let AstDependencyReplacement::RangeReplacements { replacements } = &self.replacement else {
      return false;
    };
    if replacements.is_empty() {
      return false;
    }

    replacements
      .iter()
      .all(|(_, start, end)| existing_ranges.contains(&DependencyRange::new(*start, *end)))
  }
}

fn apply_inner_replacements(
  source_text: &str,
  range: DependencyRange,
  replacements: &[(Box<str>, u32, u32)],
) -> String {
  let mut replacements = replacements.iter().collect::<Vec<_>>();
  replacements.sort_by_key(|(_, start, _)| *start);

  let mut output = String::new();
  let mut cursor = range.start as usize;
  for (content, start, end) in replacements {
    let start = *start as usize;
    let end = *end as usize;
    output.push_str(&source_text[cursor..start]);
    output.push_str(content.as_ref());
    cursor = end;
  }
  output.push_str(&source_text[cursor..range.end as usize]);
  output
}

#[derive(Debug, Default)]
pub struct AstDependencyRenderPlan {
  actions: Vec<AstDependencyAction>,
  side_effects: Vec<AstDependencySideEffect>,
  source_replacement_ranges: FxHashSet<DependencyRange>,
}

impl AstDependencyRenderPlan {
  pub fn push_action(&mut self, action: AstDependencyAction) {
    if action.is_redundant_range_replacement(&self.source_replacement_ranges) {
      return;
    }

    for range in action.source_replacement_ranges() {
      self.source_replacement_ranges.insert(range);
    }
    self.actions.push(action);
  }

  pub fn push_side_effect(&mut self, side_effect: AstDependencySideEffect) {
    self.side_effects.push(side_effect);
  }

  pub fn has_actions(&self) -> bool {
    !self.actions.is_empty()
  }

  pub fn side_effects(&self) -> &[AstDependencySideEffect] {
    &self.side_effects
  }
}

pub fn render_ast_dependencies(source_text: &str, plan: &AstDependencyRenderPlan) -> String {
  if !plan.has_actions() {
    return source_text.to_string();
  }

  render_source_replacements(source_text, &plan.actions)
}

pub fn apply_ast_dependency_replacements(
  source_text: &str,
  plan: &AstDependencyRenderPlan,
  source: &mut TemplateReplaceSource,
) {
  for (range, replacement, _) in source_replacements(source_text, &plan.actions) {
    source.replace(range.start, range.end, replacement, None);
  }
}

fn render_source_replacements(source_text: &str, actions: &[AstDependencyAction]) -> String {
  let replacements = source_replacements(source_text, actions);

  let mut output = String::with_capacity(source_text.len());
  let mut cursor = 0usize;
  let source_len = source_text.len();
  for (range, replacement, _) in replacements {
    let start = range.start as usize;
    let end = range.end as usize;
    if start > cursor {
      let prefix_end = start.min(source_len);
      output.push_str(&source_text[cursor..prefix_end]);
      cursor = prefix_end;
    }
    output.push_str(&replacement);
    cursor = cursor.max(end.min(source_len));
  }
  if cursor < source_len {
    output.push_str(&source_text[cursor..]);
  }
  output
}

fn source_replacements(
  source_text: &str,
  actions: &[AstDependencyAction],
) -> Vec<(DependencyRange, String, i8)> {
  let mut replacements = Vec::new();
  for action in actions {
    match &action.replacement {
      AstDependencyReplacement::WrappedSource {
        replacement,
        prefix,
        suffix,
      } => {
        push_source_replacement(
          &mut replacements,
          DependencyRange::new(action.range.start, replacement.start),
          prefix.to_string(),
          0,
        );
        push_source_replacement(
          &mut replacements,
          DependencyRange::new(replacement.end, action.range.end),
          suffix.to_string(),
          0,
        );
        continue;
      }
      AstDependencyReplacement::WrappedSourceWithReplacements {
        replacement,
        prefix,
        suffix,
        replacements: inner_replacements,
      } => {
        push_source_replacement(
          &mut replacements,
          DependencyRange::new(action.range.start, replacement.start),
          prefix.to_string(),
          0,
        );
        replacements.extend(inner_replacements.iter().map(|(content, start, end)| {
          (DependencyRange::new(*start, *end), content.to_string(), 0)
        }));
        push_source_replacement(
          &mut replacements,
          DependencyRange::new(replacement.end, action.range.end),
          suffix.to_string(),
          0,
        );
        continue;
      }
      AstDependencyReplacement::WrappedSourceTrimTrailingSemicolon { prefix, suffix } => {
        let suffix_position = trim_trailing_semicolon(source_text, action.range);
        push_source_replacement(
          &mut replacements,
          DependencyRange::new(action.range.start, action.range.start),
          prefix.to_string(),
          0,
        );
        push_source_replacement(
          &mut replacements,
          DependencyRange::new(suffix_position, suffix_position),
          suffix.to_string(),
          -1,
        );
        continue;
      }
      AstDependencyReplacement::RangeReplacements { replacements: r }
      | AstDependencyReplacement::SourceWithReplacements {
        replacements: r, ..
      } => {
        replacements.extend(r.iter().map(|(content, start, end)| {
          (DependencyRange::new(*start, *end), content.to_string(), 0)
        }));
        continue;
      }
      _ => {}
    }

    replacements.push((
      action.edit_range(),
      action.replacement_content(source_text).into_owned(),
      0,
    ));
  }

  replacements.sort_by_key(|(range, _, enforce)| (range.start, range.end, *enforce));
  replacements
}

fn trim_trailing_semicolon(source_text: &str, range: DependencyRange) -> u32 {
  let bytes = source_text.as_bytes();
  let start = range.start as usize;
  let mut end = range.end as usize;
  while end > start && matches!(bytes[end - 1], b' ' | b'\t' | b'\n' | b'\r') {
    end -= 1;
  }
  if end > start && bytes[end - 1] == b';' {
    (end - 1) as u32
  } else {
    range.end
  }
}

fn push_source_replacement(
  replacements: &mut Vec<(DependencyRange, String, i8)>,
  range: DependencyRange,
  content: String,
  enforce: i8,
) {
  if range.start == range.end && content.is_empty() {
    return;
  }

  replacements.push((range, content, enforce));
}

#[cfg(test)]
mod tests {
  use rspack_core::ModuleType;
  use swc_experimental_allocator::Allocator;
  use swc_experimental_ecma_ast::{EsVersion, GetSpan, Program, Stmt};
  use swc_experimental_ecma_parser::{EsSyntax, Lexer, Parser, StringSource, Syntax};

  use super::*;

  fn ast_syntax(module_type: &ModuleType) -> Syntax {
    Syntax::Es(EsSyntax {
      jsx: true,
      allow_return_outside_function: matches!(
        module_type,
        ModuleType::JsDynamic | ModuleType::JsAuto
      ),
      explicit_resource_management: true,
      import_attributes: true,
      ..Default::default()
    })
  }

  fn parse_program_for_test<'ast>(
    allocator: &'ast Allocator,
    source_text: &'ast str,
    module_type: &ModuleType,
  ) -> Program<'ast> {
    let lexer = Lexer::new(
      allocator,
      ast_syntax(module_type),
      EsVersion::EsNext,
      StringSource::new(source_text),
      None,
    );
    let mut parser = Parser::new_from(allocator, lexer);
    let program = match module_type {
      ModuleType::JsEsm => parser
        .parse_module()
        .map(|module| Program::Module(allocator.boxed(module))),
      ModuleType::JsDynamic => parser
        .parse_commonjs()
        .map(|script| Program::Script(allocator.boxed(script))),
      _ => parser.parse_program(),
    }
    .unwrap();

    assert!(parser.take_errors().is_empty());
    program
  }

  fn first_top_level_range(source: &str, module_type: ModuleType) -> DependencyRange {
    let allocator = Allocator::new();
    let source = allocator.alloc_str(source);
    let program = parse_program_for_test(&allocator, source, &module_type);
    match program {
      Program::Module(module) => DependencyRange::from(module.body[0].span()),
      Program::Script(script) => DependencyRange::from(script.body[0].span()),
    }
  }

  fn if_consequent_range(source: &str) -> DependencyRange {
    let allocator = Allocator::new();
    let source = allocator.alloc_str(source);
    let program = parse_program_for_test(&allocator, source, &ModuleType::JsAuto);
    let Program::Script(script) = program else {
      unreachable!()
    };
    let Stmt::If(if_stmt) = &script.body[0] else {
      unreachable!()
    };
    DependencyRange::from(if_stmt.cons.span())
  }

  #[test]
  fn renders_original_source_for_empty_plan() {
    let source = "console.log(1);\n";
    let plan = AstDependencyRenderPlan::default();

    let output = render_ast_dependencies(source, &plan);

    assert_eq!(output, source);
  }

  #[test]
  fn replaces_expression_by_ast_range() {
    let source = "if (process.env.NODE_ENV) console.log(1);\n";
    let start = source.find("process.env.NODE_ENV").unwrap() as u32;
    let end = start + "process.env.NODE_ENV".len() as u32;
    let mut plan = AstDependencyRenderPlan::default();
    plan.push_action(AstDependencyAction::expr((start, end).into(), "\"production\"").unwrap());

    let output = render_ast_dependencies(source, &plan);

    assert!(output.contains("if (\"production\")"));
  }

  #[test]
  fn replaces_member_expression_assignment_target_by_ast_range() {
    let source = "Curve.create = function () {};\n";
    let start = source.find("Curve.create").unwrap() as u32;
    let end = start + "Curve.create".len() as u32;
    let mut plan = AstDependencyRenderPlan::default();
    plan.push_action(
      AstDependencyAction::expr(
        DependencyRange::new(start, end),
        "_extras_core_Curve_js__rspack_import_10.Curve.create",
      )
      .unwrap(),
    );

    let output = render_ast_dependencies(source, &plan);

    assert_eq!(
      output,
      "_extras_core_Curve_js__rspack_import_10.Curve.create = function () {};\n"
    );
  }

  #[test]
  fn deletes_top_level_statement_by_ast_range() {
    let source = "\"use strict\";\nconsole.log(1);\n";
    let mut plan = AstDependencyRenderPlan::default();
    plan.push_action(
      AstDependencyAction::expr(first_top_level_range(source, ModuleType::JsAuto), "").unwrap(),
    );

    let output = render_ast_dependencies(source, &plan);

    assert!(!output.contains("use strict"));
    assert!(output.contains("console.log(1)"));
  }

  #[test]
  fn replaces_nested_statement_by_ast_range() {
    let source = "if (true) console.log('x');\n";
    let mut plan = AstDependencyRenderPlan::default();
    plan.push_action(AstDependencyAction::expr(if_consequent_range(source), "{}").unwrap());

    let output = render_ast_dependencies(source, &plan);

    assert!(output.contains("if (true) {}"));
  }

  #[test]
  fn deletes_module_item_by_ast_range() {
    let source = "import './a';\nconsole.log(1);\n";
    let mut plan = AstDependencyRenderPlan::default();
    plan.push_action(
      AstDependencyAction::expr(first_top_level_range(source, ModuleType::JsEsm), "").unwrap(),
    );

    let output = render_ast_dependencies(source, &plan);

    assert!(!output.contains("import"));
    assert!(output.contains("console.log(1)"));
  }

  #[test]
  fn deletes_module_item_prefix_with_replacements() {
    let source = "export const answer = 42;\n";
    let range = first_top_level_range(source, ModuleType::JsEsm);
    let replacement_start = source.find("const").unwrap() as u32;
    let mut plan = AstDependencyRenderPlan::default();
    plan.push_action(
      AstDependencyAction::range_replacements(
        range,
        vec![(String::new(), range.start, replacement_start)],
      )
      .unwrap(),
    );

    let output = render_ast_dependencies(source, &plan);

    assert!(!output.contains("export"));
    assert!(output.contains("const answer = 42"));
  }

  #[test]
  fn replaces_export_default_template_with_leading_comment() {
    let source = "export default /* glsl */`\nvoid main() {}\n`;\n";
    let range_stmt = DependencyRange::new(0, source.find(';').unwrap() as u32 + 1);
    let range = DependencyRange::new(
      source.find('`').unwrap() as u32,
      source.rfind('`').unwrap() as u32 + 1,
    );
    let mut plan = AstDependencyRenderPlan::default();
    plan.push_action(
      AstDependencyAction::source_with_replacements(
        range_stmt,
        range_stmt,
        vec![
          (
            "/* export default */ const __WEBPACK_DEFAULT_EXPORT__ = (/* glsl */".to_string(),
            range_stmt.start,
            range.start,
          ),
          (");".to_string(), range.end, range_stmt.end),
        ],
      )
      .unwrap(),
    );
    plan.push_action(
      AstDependencyAction::range_replacements(
        range_stmt,
        vec![(String::new(), range_stmt.start, range.start)],
      )
      .unwrap(),
    );

    let output = render_ast_dependencies(source, &plan);

    assert_eq!(
      output,
      "/* export default */ const __WEBPACK_DEFAULT_EXPORT__ = (/* glsl */`\nvoid main() {}\n`);\n"
    );
  }

  #[test]
  fn wraps_expression_source_slice() {
    let source = "new Worker(workerUrl);\n";
    let start = source.find("workerUrl").unwrap() as u32;
    let end = start + "workerUrl".len() as u32;
    let mut plan = AstDependencyRenderPlan::default();
    plan.push_action(
      AstDependencyAction::wrapped_source(
        DependencyRange::new(start, end),
        DependencyRange::new(start, end),
        "__webpack_require__.tu(",
        ")",
      )
      .unwrap(),
    );

    let output = render_ast_dependencies(source, &plan);

    assert!(output.contains("new Worker(__webpack_require__.tu(workerUrl))"));
  }

  #[test]
  fn emits_wrapped_source_as_insert_replacements() {
    let source = "const b = { a: 123 };\n";
    let start = source.find("{ a: 123 }").unwrap() as u32;
    let end = start + "{ a: 123 }".len() as u32;
    let mut plan = AstDependencyRenderPlan::default();
    plan.push_action(
      AstDependencyAction::wrapped_source(
        DependencyRange::new(start, end),
        DependencyRange::new(start, end),
        "(/* unused pure expression or super */ null && (",
        "))",
      )
      .unwrap(),
    );

    let replacements = source_replacements(source, &plan.actions);

    assert_eq!(
      replacements,
      vec![
        (
          DependencyRange::new(start, start),
          "(/* unused pure expression or super */ null && (".to_string(),
          0
        ),
        (DependencyRange::new(end, end), "))".to_string(), 0),
      ]
    );
  }

  #[test]
  fn wraps_expression_source_slice_with_inner_replacements() {
    let source = "const value = import(\"./dir/\" + name);\n";
    let range_start = source.find("import(").unwrap() as u32;
    let range_end = source.find(");").unwrap() as u32 + 1;
    let replacement_start = source.find("\"./dir/\"").unwrap() as u32;
    let replacement_end = range_end - 1;
    let replace_start = source.find("name").unwrap() as u32;
    let replace_end = replace_start + "name".len() as u32;
    let mut plan = AstDependencyRenderPlan::default();
    plan.push_action(
      AstDependencyAction::wrapped_source_with_replacements(
        DependencyRange::new(range_start, range_end),
        DependencyRange::new(replacement_start, replacement_end),
        "__webpack_require__(42)(",
        ")",
        vec![("\"tmpl\"".to_string(), replace_start, replace_end)],
      )
      .unwrap(),
    );

    let output = render_ast_dependencies(source, &plan);

    assert_eq!(
      output,
      "const value = __webpack_require__(42)(\"./dir/\" + \"tmpl\");\n"
    );
  }

  #[test]
  fn preserves_outer_source_while_replacing_inner_range() {
    let source = "const url = new MyURL(\"./asset.png\", import.meta.url);\n";
    let range_start = source.find("new MyURL").unwrap() as u32;
    let range_end = source.find(");").unwrap() as u32 + 1;
    let replacement_start = source.find("\"./asset.png\"").unwrap() as u32;
    let replacement_end = range_end - 1;
    let mut plan = AstDependencyRenderPlan::default();
    plan.push_action(
      AstDependencyAction::source_with_replacements(
        DependencyRange::new(range_start, range_end),
        DependencyRange::new(replacement_start, replacement_end),
        vec![(
          "/* asset import */__webpack_require__(1), __webpack_require__.b".to_string(),
          replacement_start,
          replacement_end,
        )],
      )
      .unwrap(),
    );

    let output = render_ast_dependencies(source, &plan);

    assert_eq!(
      output,
      "const url = new MyURL(/* asset import */__webpack_require__(1), __webpack_require__.b);\n"
    );
  }

  #[test]
  fn replaces_expression_with_raw_content() {
    let source = "const id = require.resolve(\"./a\");\n";
    let start = source.find("require.resolve").unwrap() as u32;
    let end = start + "require.resolve".len() as u32;
    let mut plan = AstDependencyRenderPlan::default();
    plan.push_action(
      AstDependencyAction::raw_expr((start, end).into(), "/*require.resolve*/").unwrap(),
    );

    let output = render_ast_dependencies(source, &plan);

    assert_eq!(output, "const id = /*require.resolve*/(\"./a\");\n");
  }

  #[test]
  fn inserts_content_around_outer_expression() {
    let source = "module.hot.accept(\"./a\");\n";
    let call_start = source.find("module.hot.accept").unwrap() as u32;
    let call_end = source.find(");").unwrap() as u32 + 1;
    let callee_end = call_start + "module.hot.accept".len() as u32;
    let insert_position = call_end - 1;
    let mut plan = AstDependencyRenderPlan::default();
    plan.push_action(
      AstDependencyAction::expr(
        DependencyRange::new(call_start, callee_end),
        "__webpack_module__.hot.accept",
      )
      .unwrap(),
    );
    plan.push_action(
      AstDependencyAction::insert(
        DependencyRange::new(call_start, call_end),
        insert_position,
        ", function(){}",
      )
      .unwrap(),
    );

    let output = render_ast_dependencies(source, &plan);

    assert_eq!(
      output,
      "__webpack_module__.hot.accept(\"./a\", function(){});\n"
    );
  }

  #[test]
  fn inserts_prefix_while_replacing_the_same_expression() {
    let source = "foo(bar);\n";
    let call_start = source.find("foo").unwrap() as u32;
    let call_end = source.find(");").unwrap() as u32 + 1;
    let mut plan = AstDependencyRenderPlan::default();
    plan.push_action(
      AstDependencyAction::insert(DependencyRange::new(call_start, call_end), 0, "var x;").unwrap(),
    );
    plan.push_action(
      AstDependencyAction::expr(DependencyRange::new(call_start, call_end), "baz(bar)").unwrap(),
    );

    let output = render_ast_dependencies(source, &plan);

    assert_eq!(output, "var x;baz(bar);\n");
  }

  #[test]
  fn applies_disjoint_replacements_around_nested_action() {
    let source = "define([\"a\"], function(a) {});\n";
    let call_start = source.find("define").unwrap() as u32;
    let call_end = source.find(");").unwrap() as u32 + 1;
    let array_start = source.find("[\"a\"]").unwrap() as u32;
    let array_end = array_start + "[\"a\"]".len() as u32;
    let function_start = source.find("function").unwrap() as u32;
    let function_end = source.find(");").unwrap() as u32;
    let mut plan = AstDependencyRenderPlan::default();
    plan.push_action(
      AstDependencyAction::range_replacements(
        DependencyRange::new(call_start, call_end),
        vec![
          ("wrap(".to_string(), call_start, array_start),
          (", ".to_string(), array_end, function_start),
          (")".to_string(), function_end, call_end),
        ],
      )
      .unwrap(),
    );
    plan.push_action(
      AstDependencyAction::expr(DependencyRange::new(array_start, array_end), "deps").unwrap(),
    );

    let output = render_ast_dependencies(source, &plan);

    assert_eq!(output, "wrap(deps, function(a) {});\n");
  }

  #[test]
  fn inserts_suffix_after_raw_identifier() {
    let source = "const obj = { value };\n";
    let start = source.find("value").unwrap() as u32;
    let end = start + "value".len() as u32;
    let mut plan = AstDependencyRenderPlan::default();
    plan.push_action(
      AstDependencyAction::raw_ident_with_suffix((start, end).into(), ": replacement").unwrap(),
    );

    let output = render_ast_dependencies(source, &plan);

    assert_eq!(output, "const obj = { value: replacement };\n");
  }

  #[test]
  fn replaces_raw_identifier_with_property_fragment() {
    let source = "const { value } = source;\n";
    let start = source.find("value").unwrap() as u32;
    let end = start + "value".len() as u32;
    let mut plan = AstDependencyRenderPlan::default();
    plan
      .push_action(AstDependencyAction::raw_ident((start, end).into(), "renamed: value").unwrap());

    let output = render_ast_dependencies(source, &plan);

    assert_eq!(output, "const { renamed: value } = source;\n");
  }

  #[test]
  fn replaces_raw_literal_with_property_fragment() {
    let source = "const { \"value\": local } = source;\n";
    let start = source.find("\"value\"").unwrap() as u32;
    let end = start + "\"value\"".len() as u32;
    let mut plan = AstDependencyRenderPlan::default();
    plan.push_action(AstDependencyAction::raw_ident((start, end).into(), "renamed").unwrap());

    let output = render_ast_dependencies(source, &plan);

    assert_eq!(output, "const { renamed: local } = source;\n");
  }
}
