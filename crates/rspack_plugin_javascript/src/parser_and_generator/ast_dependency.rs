use std::{borrow::Cow, sync::Arc};

use rspack_core::{
  DependencyCodeGeneration, DependencyId, DependencyRange, InitFragmentExt, InitFragmentKey,
  InitFragmentStage, ModuleType, NormalInitFragment, RuntimeGlobals, TemplateContext,
  TemplateReplaceSource,
};
use rspack_util::SpanExt;
use rustc_hash::FxHashSet;
use swc_core::{
  common::{
    FileName, GLOBALS, Globals, SourceMap, Spanned, comments::SingleThreadedComments,
    input::SourceFileInput,
  },
  ecma::{
    ast::{
      EsVersion as StableEsVersion, Expr, Ident as StableIdent, Lit as StableLit,
      ModuleItem as StableModuleItem, Program as StableProgram, PropName as StablePropName,
      Stmt as StableStmt,
    },
    parser::{
      EsSyntax as StableEsSyntax, Parser as StableParser, Syntax as StableSyntax,
      lexer::Lexer as StableLexer,
    },
    visit::{VisitMut, VisitMutWith},
  },
};
use swc_experimental_allocator::Allocator;
use swc_experimental_ecma_ast::{
  Expr as ExperimentalExpr, GetSpan, Ident, Lit, MemberExpr, ModuleDecl, ModuleItem, Program, Stmt,
  Str, Visit, VisitWith,
};
use swc_experimental_ecma_parser::{EsSyntax, Lexer, Parser, StringSource, Syntax};

#[derive(Debug, Clone, Copy)]
pub enum AstDependencyRenderKey {
  Dependency(DependencyId),
  Presentational(usize),
}

pub struct AstDependencyRenderEntry {
  pub key: AstDependencyRenderKey,
  range: DependencyRange,
  applied: bool,
}

impl AstDependencyRenderEntry {
  pub fn new(
    key: AstDependencyRenderKey,
    dependency: &dyn DependencyCodeGeneration,
  ) -> Option<Self> {
    let range = dependency.ast_dependency_range()?;
    if range.start >= range.end {
      return None;
    }

    Some(Self {
      key,
      range,
      applied: false,
    })
  }

  pub fn applied(&self) -> bool {
    self.applied
  }
}

#[derive(Debug, Clone)]
pub enum AstDependencySideEffect {
  RuntimeRequirements(RuntimeGlobals),
  CachedConst {
    identifier: Box<str>,
    content: Box<str>,
  },
  RenderTemplate(AstDependencyRenderKey),
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
      Self::RenderTemplate(_) => {}
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
  ValidatedReplacements {
    replacements: Vec<(Box<str>, u32, u32)>,
  },
  SourceWithReplacements {
    replacement: DependencyRange,
    replacements: Vec<(Box<str>, u32, u32)>,
  },
}

impl AstDependencyAction {
  pub fn expr(range: DependencyRange, content: impl Into<Box<str>>) -> Option<Self> {
    if range.start >= range.end {
      return None;
    }

    Some(Self {
      range,
      replacement: AstDependencyReplacement::Generated(content.into()),
    })
  }

  pub fn raw_expr(range: DependencyRange, content: impl Into<Box<str>>) -> Option<Self> {
    if range.start >= range.end {
      return None;
    }

    Some(Self {
      range,
      replacement: AstDependencyReplacement::RawExpr(content.into()),
    })
  }

  pub fn raw_ident(range: DependencyRange, content: impl Into<Box<str>>) -> Option<Self> {
    if range.start >= range.end {
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
    if range.start >= range.end {
      return None;
    }

    Some(Self {
      range,
      replacement: AstDependencyReplacement::RawIdentWithSuffix(suffix.into()),
    })
  }

  pub fn insert(
    validate_range: DependencyRange,
    position: u32,
    content: impl Into<Box<str>>,
  ) -> Option<Self> {
    if validate_range.start >= validate_range.end {
      return None;
    }

    Some(Self {
      range: validate_range,
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
    if range.start >= range.end || replacement.start > replacement.end {
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
    if range.start >= range.end || replacement.start > replacement.end {
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

  pub fn source_with_replacements(
    range: DependencyRange,
    replacement: DependencyRange,
    replacements: Vec<(String, u32, u32)>,
  ) -> Option<Self> {
    if range.start >= range.end
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

  pub fn validated_replacements(
    validate_range: DependencyRange,
    replacements: Vec<(String, u32, u32)>,
  ) -> Option<Self> {
    if validate_range.start >= validate_range.end {
      return None;
    }

    Some(Self {
      range: validate_range,
      replacement: AstDependencyReplacement::ValidatedReplacements {
        replacements: replacements
          .into_iter()
          .map(|(content, start, end)| (content.into_boxed_str(), start, end))
          .collect(),
      },
    })
  }

  fn replacement_content<'a>(&'a self, source_text: &'a str) -> Option<Cow<'a, str>> {
    match &self.replacement {
      AstDependencyReplacement::Generated(content) => Some(Cow::Borrowed(content.as_ref())),
      AstDependencyReplacement::RawExpr(content) => Some(Cow::Borrowed(content.as_ref())),
      AstDependencyReplacement::RawIdent(content) => Some(Cow::Borrowed(content.as_ref())),
      AstDependencyReplacement::RawIdentWithSuffix(suffix) => source_text
        .get(self.range.start as usize..self.range.end as usize)
        .map(|source| Cow::Owned(format!("{source}{suffix}"))),
      AstDependencyReplacement::Insert { content, .. } => Some(Cow::Borrowed(content.as_ref())),
      AstDependencyReplacement::WrappedSource {
        replacement,
        prefix,
        suffix,
      } => source_text
        .get(replacement.start as usize..replacement.end as usize)
        .map(|source| Cow::Owned(format!("{prefix}{source}{suffix}"))),
      AstDependencyReplacement::WrappedSourceWithReplacements {
        replacement,
        prefix,
        suffix,
        replacements,
      } => apply_inner_replacements(source_text, *replacement, replacements)
        .map(|source| Cow::Owned(format!("{prefix}{source}{suffix}"))),
      AstDependencyReplacement::ValidatedReplacements { .. } => None,
      AstDependencyReplacement::SourceWithReplacements {
        replacement,
        replacements,
      } => {
        let prefix = source_text.get(self.range.start as usize..replacement.start as usize)?;
        let suffix = source_text.get(replacement.end as usize..self.range.end as usize)?;
        apply_inner_replacements(source_text, *replacement, replacements)
          .map(|source| Cow::Owned(format!("{prefix}{source}{suffix}")))
      }
    }
  }

  fn edit_range(&self) -> DependencyRange {
    match &self.replacement {
      AstDependencyReplacement::Insert { position, .. } => {
        DependencyRange::new(*position, *position)
      }
      AstDependencyReplacement::ValidatedReplacements { .. } => self.range,
      _ => self.range,
    }
  }

  fn edit_ranges(&self) -> Vec<DependencyRange> {
    match &self.replacement {
      AstDependencyReplacement::ValidatedReplacements { replacements } => replacements
        .iter()
        .map(|(_, start, end)| DependencyRange::new(*start, *end))
        .collect(),
      _ => vec![self.edit_range()],
    }
  }
}

fn apply_inner_replacements(
  source_text: &str,
  range: DependencyRange,
  replacements: &[(Box<str>, u32, u32)],
) -> Option<String> {
  if range.start > range.end
    || range.end as usize > source_text.len()
    || !source_text.is_char_boundary(range.start as usize)
    || !source_text.is_char_boundary(range.end as usize)
  {
    return None;
  }

  let mut replacements = replacements.to_vec();
  replacements.sort_by_key(|(_, start, _)| *start);

  let mut last_end = range.start;
  for (_, start, end) in &replacements {
    if *start < last_end
      || start > end
      || *start < range.start
      || *end > range.end
      || !source_text.is_char_boundary(*start as usize)
      || !source_text.is_char_boundary(*end as usize)
    {
      return None;
    }
    last_end = *end;
  }

  let mut output = String::new();
  let mut cursor = range.start as usize;
  for (content, start, end) in replacements {
    let start = start as usize;
    let end = end as usize;
    output.push_str(&source_text[cursor..start]);
    output.push_str(&content);
    cursor = end;
  }
  output.push_str(&source_text[cursor..range.end as usize]);
  Some(output)
}

#[derive(Debug, Default)]
pub struct AstDependencyRenderPlan {
  actions: Vec<AstDependencyAction>,
  side_effects: Vec<AstDependencySideEffect>,
}

impl AstDependencyRenderPlan {
  pub fn push_action(&mut self, action: AstDependencyAction) {
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

struct ParsedAstDependencyAction {
  range: DependencyRange,
  expr_replacement: Option<Box<Expr>>,
  ident_replacement: bool,
  validate_only: bool,
  stmt_replacement: Option<Option<StableStmt>>,
  module_item_replacement: Option<Option<StableModuleItem>>,
  applied: bool,
}

impl ParsedAstDependencyAction {
  fn new(action: &AstDependencyAction, source_text: &str) -> Option<Self> {
    if matches!(action.replacement, AstDependencyReplacement::RawExpr(_)) {
      return Some(Self {
        range: action.range,
        expr_replacement: parse_replacement_expr("__rspack_ast_dependency_raw_replacement__"),
        ident_replacement: false,
        validate_only: false,
        stmt_replacement: None,
        module_item_replacement: None,
        applied: false,
      });
    }

    if matches!(
      action.replacement,
      AstDependencyReplacement::RawIdent(_) | AstDependencyReplacement::RawIdentWithSuffix(_)
    ) {
      return Some(Self {
        range: action.range,
        expr_replacement: None,
        ident_replacement: true,
        validate_only: false,
        stmt_replacement: None,
        module_item_replacement: None,
        applied: false,
      });
    }

    if matches!(
      action.replacement,
      AstDependencyReplacement::Insert { .. }
        | AstDependencyReplacement::ValidatedReplacements { .. }
    ) {
      return Some(Self {
        range: action.range,
        expr_replacement: None,
        ident_replacement: false,
        validate_only: true,
        stmt_replacement: None,
        module_item_replacement: None,
        applied: false,
      });
    }

    let content = action.replacement_content(source_text)?;
    let delete = content.trim().is_empty();
    let expr_replacement = if delete {
      None
    } else {
      parse_replacement_expr(&content)
    };
    let stmt_replacement = if delete {
      Some(None)
    } else {
      parse_replacement_stmt(&content).map(Some)
    };
    let module_item_replacement = if delete {
      Some(None)
    } else {
      parse_replacement_module_item(&content).map(Some)
    };

    if !delete
      && expr_replacement.is_none()
      && stmt_replacement.is_none()
      && module_item_replacement.is_none()
    {
      return None;
    }

    Some(Self {
      range: action.range,
      expr_replacement,
      ident_replacement: false,
      validate_only: false,
      stmt_replacement,
      module_item_replacement,
      applied: false,
    })
  }
}

struct StableAstDependencyApplier {
  actions: Vec<ParsedAstDependencyAction>,
}

impl StableAstDependencyApplier {
  fn new(actions: Vec<ParsedAstDependencyAction>) -> Self {
    Self { actions }
  }

  fn is_fully_applied(&self) -> bool {
    self.actions.iter().all(|action| action.applied)
  }

  fn replacement_for_expr(&mut self, range: DependencyRange) -> Option<Box<Expr>> {
    let action = self.actions.iter_mut().find(|action| {
      !action.applied && action.range == range && action.expr_replacement.is_some()
    })?;
    action.applied = true;
    action.expr_replacement.take()
  }

  fn replacement_for_ident(&mut self, range: DependencyRange) -> bool {
    let Some(action) = self
      .actions
      .iter_mut()
      .find(|action| !action.applied && action.range == range && action.ident_replacement)
    else {
      return false;
    };
    action.applied = true;
    true
  }

  fn validate_node(&mut self, range: DependencyRange) {
    for action in self
      .actions
      .iter_mut()
      .filter(|action| !action.applied && action.range == range && action.validate_only)
    {
      action.applied = true;
    }
  }

  fn replacement_for_stmt_list(&mut self, range: DependencyRange) -> Option<Option<StableStmt>> {
    let action = self.actions.iter_mut().find(|action| {
      !action.applied && action.range == range && action.stmt_replacement.is_some()
    })?;
    action.applied = true;
    action.stmt_replacement.take()
  }

  fn replacement_for_stmt_node(&mut self, range: DependencyRange) -> Option<StableStmt> {
    let action = self.actions.iter_mut().find(|action| {
      !action.applied
        && action.range == range
        && action
          .stmt_replacement
          .as_ref()
          .is_some_and(|replacement| replacement.is_some())
    })?;
    action.applied = true;
    action.stmt_replacement.take().flatten()
  }

  fn replacement_for_module_item(
    &mut self,
    range: DependencyRange,
  ) -> Option<Option<StableModuleItem>> {
    let action = self.actions.iter_mut().find(|action| {
      !action.applied && action.range == range && action.module_item_replacement.is_some()
    })?;
    action.applied = true;
    action.module_item_replacement.take()
  }
}

impl VisitMut for StableAstDependencyApplier {
  fn visit_mut_module_items(&mut self, node: &mut Vec<StableModuleItem>) {
    let mut next = Vec::with_capacity(node.len());

    for mut item in std::mem::take(node) {
      let range = DependencyRange::from(item.span());
      self.validate_node(range);
      if let Some(replacement) = self.replacement_for_module_item(range) {
        if let Some(replacement) = replacement {
          next.push(replacement);
        }
        continue;
      }

      item.visit_mut_children_with(self);
      next.push(item);
    }

    *node = next;
  }

  fn visit_mut_stmts(&mut self, node: &mut Vec<StableStmt>) {
    let mut next = Vec::with_capacity(node.len());

    for mut stmt in std::mem::take(node) {
      let range = DependencyRange::from(stmt.span());
      self.validate_node(range);
      if let Some(replacement) = self.replacement_for_stmt_list(range) {
        if let Some(replacement) = replacement {
          next.push(replacement);
        }
        continue;
      }

      stmt.visit_mut_children_with(self);
      next.push(stmt);
    }

    *node = next;
  }

  fn visit_mut_stmt(&mut self, node: &mut StableStmt) {
    let range = DependencyRange::from(node.span());
    self.validate_node(range);
    if let Some(replacement) = self.replacement_for_stmt_node(range) {
      *node = replacement;
      return;
    }

    node.visit_mut_children_with(self);
  }

  fn visit_mut_expr(&mut self, node: &mut Expr) {
    let range = DependencyRange::from(node.span());
    self.validate_node(range);
    if let Some(replacement) = self.replacement_for_expr(range) {
      *node = *replacement;
      return;
    }

    node.visit_mut_children_with(self);
  }

  fn visit_mut_ident(&mut self, node: &mut StableIdent) {
    let range = DependencyRange::from(node.span());
    self.validate_node(range);
    if self.replacement_for_ident(range) {
      return;
    }

    node.visit_mut_children_with(self);
  }

  fn visit_mut_lit(&mut self, node: &mut StableLit) {
    let range = DependencyRange::from(node.span());
    self.validate_node(range);
    if self.replacement_for_ident(range) {
      return;
    }

    node.visit_mut_children_with(self);
  }

  fn visit_mut_prop_name(&mut self, node: &mut StablePropName) {
    let range = DependencyRange::from(node.span());
    self.validate_node(range);
    if self.replacement_for_ident(range) {
      return;
    }

    node.visit_mut_children_with(self);
  }
}

fn stable_syntax(module_type: &ModuleType) -> StableSyntax {
  StableSyntax::Es(StableEsSyntax {
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

fn parse_stable_program(
  cm: Arc<SourceMap>,
  source_text: &str,
  module_type: &ModuleType,
  comments: &SingleThreadedComments,
) -> Option<StableProgram> {
  let fm = cm.new_source_file(
    Arc::new(FileName::Custom("<rspack ast dependency>".to_string())),
    source_text.to_string(),
  );
  let lexer = StableLexer::new(
    stable_syntax(module_type),
    StableEsVersion::EsNext,
    SourceFileInput::from(&*fm),
    Some(comments),
  );
  let mut parser = StableParser::new_from(lexer);
  let program = match module_type {
    ModuleType::JsEsm => parser.parse_module().map(StableProgram::Module),
    ModuleType::JsDynamic => parser.parse_commonjs().map(StableProgram::Script),
    _ => parser.parse_program(),
  }
  .ok()?;

  parser.take_errors().is_empty().then_some(program)
}

fn parse_replacement_expr(content: &str) -> Option<Box<Expr>> {
  let cm = Arc::<SourceMap>::default();
  let fm = cm.new_source_file(
    Arc::new(FileName::Custom(
      "<rspack ast dependency replacement>".to_string(),
    )),
    content.to_string(),
  );
  let lexer = StableLexer::new(
    stable_syntax(&ModuleType::JsAuto),
    StableEsVersion::EsNext,
    SourceFileInput::from(&*fm),
    None,
  );
  let mut parser = StableParser::new_from(lexer);
  let expr = parser.parse_expr().ok()?;

  parser.take_errors().is_empty().then_some(expr)
}

fn parse_replacement_stmt(content: &str) -> Option<StableStmt> {
  let cm = Arc::<SourceMap>::default();
  let fm = cm.new_source_file(
    Arc::new(FileName::Custom(
      "<rspack ast dependency replacement>".to_string(),
    )),
    content.to_string(),
  );
  let lexer = StableLexer::new(
    stable_syntax(&ModuleType::JsAuto),
    StableEsVersion::EsNext,
    SourceFileInput::from(&*fm),
    None,
  );
  let mut parser = StableParser::new_from(lexer);
  let stmt = parser.parse_stmt().ok()?;

  parser.take_errors().is_empty().then_some(stmt)
}

fn parse_replacement_module_item(content: &str) -> Option<StableModuleItem> {
  let cm = Arc::<SourceMap>::default();
  let fm = cm.new_source_file(
    Arc::new(FileName::Custom(
      "<rspack ast dependency replacement>".to_string(),
    )),
    content.to_string(),
  );
  let lexer = StableLexer::new(
    stable_syntax(&ModuleType::JsAuto),
    StableEsVersion::EsNext,
    SourceFileInput::from(&*fm),
    None,
  );
  let mut parser = StableParser::new_from(lexer);
  let module_item = parser.parse_module_item().ok()?;

  parser.take_errors().is_empty().then_some(module_item)
}

pub fn render_ast_dependencies(
  source_text: &str,
  module_type: &ModuleType,
  plan: &AstDependencyRenderPlan,
) -> Option<String> {
  if !plan.has_actions() {
    return None;
  }

  validate_ast_dependency_actions(source_text, module_type, &plan.actions)?;
  render_source_replacements(source_text, &plan.actions)
}

pub fn apply_ast_dependency_replacements(
  source_text: &str,
  module_type: &ModuleType,
  plan: &AstDependencyRenderPlan,
  source: &mut TemplateReplaceSource,
) -> bool {
  if validate_ast_dependency_actions(source_text, module_type, &plan.actions).is_none() {
    return false;
  }

  let Some(replacements) = source_replacements(source_text, &plan.actions) else {
    return false;
  };

  for (range, replacement) in replacements {
    source.replace(range.start, range.end, replacement, None);
  }

  true
}

fn validate_ast_dependency_actions(
  source_text: &str,
  module_type: &ModuleType,
  actions: &[AstDependencyAction],
) -> Option<()> {
  let mut seen_ranges = FxHashSet::default();
  let actions = actions
    .iter()
    .map(|action| {
      for range in action.edit_ranges() {
        if !seen_ranges.insert(range) {
          return None;
        }
      }
      ParsedAstDependencyAction::new(action, source_text)
    })
    .collect::<Option<Vec<_>>>()?;

  let globals = Globals::new();
  GLOBALS.set(&globals, || {
    let cm = Arc::<SourceMap>::default();
    let comments = SingleThreadedComments::default();
    let mut program = parse_stable_program(cm.clone(), source_text, module_type, &comments)?;
    let mut applier = StableAstDependencyApplier::new(actions);
    program.visit_mut_with(&mut applier);
    applier.is_fully_applied().then_some(())
  })
}

fn render_source_replacements(
  source_text: &str,
  actions: &[AstDependencyAction],
) -> Option<String> {
  let replacements = source_replacements(source_text, actions)?;

  let mut output = String::with_capacity(source_text.len());
  let mut cursor = 0usize;
  for (range, replacement) in replacements {
    let start = range.start as usize;
    let end = range.end as usize;
    output.push_str(&source_text[cursor..start]);
    output.push_str(&replacement);
    cursor = end;
  }
  output.push_str(&source_text[cursor..]);
  Some(output)
}

fn source_replacements(
  source_text: &str,
  actions: &[AstDependencyAction],
) -> Option<Vec<(DependencyRange, String)>> {
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
        );
        push_source_replacement(
          &mut replacements,
          DependencyRange::new(replacement.end, action.range.end),
          suffix.to_string(),
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
        );
        replacements.extend(
          inner_replacements
            .iter()
            .map(|(content, start, end)| (DependencyRange::new(*start, *end), content.to_string())),
        );
        push_source_replacement(
          &mut replacements,
          DependencyRange::new(replacement.end, action.range.end),
          suffix.to_string(),
        );
        continue;
      }
      AstDependencyReplacement::ValidatedReplacements { replacements: r }
      | AstDependencyReplacement::SourceWithReplacements {
        replacements: r, ..
      } => {
        replacements
          .extend(r.iter().map(|(content, start, end)| {
            (DependencyRange::new(*start, *end), content.to_string())
          }));
        continue;
      }
      _ => {}
    }

    replacements.push((
      action.edit_range(),
      action.replacement_content(source_text)?.into_owned(),
    ));
  }

  replacements.sort_by_key(|(range, _)| (range.start, range.end));
  let mut last_end = 0;
  for (range, _) in &replacements {
    if range.start < last_end
      || range.start > range.end
      || range.end as usize > source_text.len()
      || !source_text.is_char_boundary(range.start as usize)
      || !source_text.is_char_boundary(range.end as usize)
    {
      return None;
    }
    last_end = range.end;
  }

  Some(replacements)
}

fn push_source_replacement(
  replacements: &mut Vec<(DependencyRange, String)>,
  range: DependencyRange,
  content: String,
) {
  if range.start == range.end && content.is_empty() {
    return;
  }

  replacements.push((range, content));
}

struct AstDependencyMarker<'a> {
  entries: &'a mut [AstDependencyRenderEntry],
}

impl AstDependencyMarker<'_> {
  fn mark_span(&mut self, span: swc_experimental_ecma_ast::Span) {
    let range = DependencyRange::new(span.real_lo(), span.real_hi());
    if range.start >= range.end {
      return;
    }

    for entry in self.entries.iter_mut() {
      if !entry.applied && entry.range == range {
        entry.applied = true;
      }
    }
  }
}

impl<'ast> Visit<'ast> for AstDependencyMarker<'_> {
  fn visit_module_item(&mut self, node: &ModuleItem<'ast>) {
    self.mark_span(node.span());
    node.visit_children_with(self);
  }

  fn visit_module_decl(&mut self, node: &ModuleDecl<'ast>) {
    self.mark_span(node.span());
    node.visit_children_with(self);
  }

  fn visit_stmt(&mut self, node: &Stmt<'ast>) {
    self.mark_span(node.span());
    node.visit_children_with(self);
  }

  fn visit_expr(&mut self, node: &ExperimentalExpr<'ast>) {
    self.mark_span(node.span());
    node.visit_children_with(self);
  }

  fn visit_member_expr(&mut self, node: &MemberExpr<'ast>) {
    self.mark_span(node.span());
    node.visit_children_with(self);
  }

  fn visit_ident(&mut self, node: &Ident<'ast>) {
    self.mark_span(node.span);
    node.visit_children_with(self);
  }

  fn visit_lit(&mut self, node: &Lit<'ast>) {
    self.mark_span(node.span());
    node.visit_children_with(self);
  }

  fn visit_str(&mut self, node: &Str<'ast>) {
    self.mark_span(node.span);
    node.visit_children_with(self);
  }
}

pub fn mark_ast_dependencies(
  source_text: &str,
  module_type: &ModuleType,
  entries: &mut [AstDependencyRenderEntry],
) -> bool {
  let allocator = Allocator::new();
  let mut comments = Default::default();
  let lexer = Lexer::new(
    &allocator,
    Syntax::Es(EsSyntax {
      jsx: true,
      allow_return_outside_function: matches!(
        module_type,
        ModuleType::JsDynamic | ModuleType::JsAuto
      ),
      explicit_resource_management: true,
      import_attributes: true,
      ..Default::default()
    }),
    swc_experimental_ecma_ast::EsVersion::EsNext,
    StringSource::new(source_text),
    Some(&mut comments),
  );
  let mut parser = Parser::new_from(&allocator, lexer);

  let program = match match module_type {
    ModuleType::JsEsm => parser
      .parse_module()
      .map(|module| Program::Module(allocator.boxed(module))),
    ModuleType::JsDynamic => parser
      .parse_commonjs()
      .map(|script| Program::Script(allocator.boxed(script))),
    _ => parser.parse_program(),
  } {
    Ok(program) => program,
    Err(_) => return false,
  };

  if !parser.take_errors().is_empty() {
    return false;
  }
  drop(parser);

  program.visit_with(&mut AstDependencyMarker { entries });
  true
}

#[cfg(test)]
mod tests {
  use super::*;

  fn first_top_level_range(source: &str, module_type: ModuleType) -> DependencyRange {
    GLOBALS.set(&Globals::new(), || {
      let cm = Arc::<SourceMap>::default();
      let comments = SingleThreadedComments::default();
      let program = parse_stable_program(cm, source, &module_type, &comments).unwrap();
      match program {
        StableProgram::Module(module) => DependencyRange::from(module.body[0].span()),
        StableProgram::Script(script) => DependencyRange::from(script.body[0].span()),
      }
    })
  }

  fn if_consequent_range(source: &str) -> DependencyRange {
    GLOBALS.set(&Globals::new(), || {
      let cm = Arc::<SourceMap>::default();
      let comments = SingleThreadedComments::default();
      let program = parse_stable_program(cm, source, &ModuleType::JsAuto, &comments).unwrap();
      let StableProgram::Script(script) = program else {
        unreachable!()
      };
      let StableStmt::If(if_stmt) = &script.body[0] else {
        unreachable!()
      };
      DependencyRange::from(if_stmt.cons.span())
    })
  }

  #[test]
  fn replaces_expression_by_ast_range() {
    let source = "if (process.env.NODE_ENV) console.log(1);\n";
    let start = source.find("process.env.NODE_ENV").unwrap() as u32;
    let end = start + "process.env.NODE_ENV".len() as u32;
    let mut plan = AstDependencyRenderPlan::default();
    plan.push_action(AstDependencyAction::expr((start, end).into(), "\"production\"").unwrap());

    let output = render_ast_dependencies(source, &ModuleType::JsEsm, &plan).unwrap();

    assert!(output.contains("if (\"production\")"));
  }

  #[test]
  fn deletes_top_level_statement_by_ast_range() {
    let source = "\"use strict\";\nconsole.log(1);\n";
    let mut plan = AstDependencyRenderPlan::default();
    plan.push_action(
      AstDependencyAction::expr(first_top_level_range(source, ModuleType::JsAuto), "").unwrap(),
    );

    let output = render_ast_dependencies(source, &ModuleType::JsAuto, &plan).unwrap();

    assert!(!output.contains("use strict"));
    assert!(output.contains("console.log(1)"));
  }

  #[test]
  fn replaces_nested_statement_by_ast_range() {
    let source = "if (true) console.log('x');\n";
    let mut plan = AstDependencyRenderPlan::default();
    plan.push_action(AstDependencyAction::expr(if_consequent_range(source), "{}").unwrap());

    let output = render_ast_dependencies(source, &ModuleType::JsAuto, &plan).unwrap();

    assert!(output.contains("if (true) {}"));
  }

  #[test]
  fn deletes_module_item_by_ast_range() {
    let source = "import './a';\nconsole.log(1);\n";
    let mut plan = AstDependencyRenderPlan::default();
    plan.push_action(
      AstDependencyAction::expr(first_top_level_range(source, ModuleType::JsEsm), "").unwrap(),
    );

    let output = render_ast_dependencies(source, &ModuleType::JsEsm, &plan).unwrap();

    assert!(!output.contains("import"));
    assert!(output.contains("console.log(1)"));
  }

  #[test]
  fn deletes_module_item_prefix_after_validating_whole_item() {
    let source = "export const answer = 42;\n";
    let range = first_top_level_range(source, ModuleType::JsEsm);
    let replacement_start = source.find("const").unwrap() as u32;
    let mut plan = AstDependencyRenderPlan::default();
    plan.push_action(
      AstDependencyAction::validated_replacements(
        range,
        vec![("".to_string(), range.start, replacement_start)],
      )
      .unwrap(),
    );

    let output = render_ast_dependencies(source, &ModuleType::JsEsm, &plan).unwrap();

    assert!(!output.contains("export"));
    assert!(output.contains("const answer = 42"));
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

    let output = render_ast_dependencies(source, &ModuleType::JsAuto, &plan).unwrap();

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

    let replacements = source_replacements(source, &plan.actions).unwrap();

    assert_eq!(
      replacements,
      vec![
        (
          DependencyRange::new(start, start),
          "(/* unused pure expression or super */ null && (".to_string()
        ),
        (DependencyRange::new(end, end), "))".to_string()),
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

    let output = render_ast_dependencies(source, &ModuleType::JsAuto, &plan).unwrap();

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

    let output = render_ast_dependencies(source, &ModuleType::JsAuto, &plan).unwrap();

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

    let output = render_ast_dependencies(source, &ModuleType::JsAuto, &plan).unwrap();

    assert_eq!(output, "const id = /*require.resolve*/(\"./a\");\n");
  }

  #[test]
  fn inserts_content_after_validating_outer_expression() {
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

    let output = render_ast_dependencies(source, &ModuleType::JsAuto, &plan).unwrap();

    assert_eq!(
      output,
      "__webpack_module__.hot.accept(\"./a\", function(){});\n"
    );
  }

  #[test]
  fn inserts_prefix_while_replacing_the_same_validated_expression() {
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

    let output = render_ast_dependencies(source, &ModuleType::JsAuto, &plan).unwrap();

    assert_eq!(output, "var x;baz(bar);\n");
  }

  #[test]
  fn applies_validated_disjoint_replacements_around_nested_action() {
    let source = "define([\"a\"], function(a) {});\n";
    let call_start = source.find("define").unwrap() as u32;
    let call_end = source.find(");").unwrap() as u32 + 1;
    let array_start = source.find("[\"a\"]").unwrap() as u32;
    let array_end = array_start + "[\"a\"]".len() as u32;
    let function_start = source.find("function").unwrap() as u32;
    let function_end = source.find(");").unwrap() as u32;
    let mut plan = AstDependencyRenderPlan::default();
    plan.push_action(
      AstDependencyAction::validated_replacements(
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

    let output = render_ast_dependencies(source, &ModuleType::JsAuto, &plan).unwrap();

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

    let output = render_ast_dependencies(source, &ModuleType::JsAuto, &plan).unwrap();

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

    let output = render_ast_dependencies(source, &ModuleType::JsAuto, &plan).unwrap();

    assert_eq!(output, "const { renamed: value } = source;\n");
  }

  #[test]
  fn replaces_raw_literal_with_property_fragment() {
    let source = "const { \"value\": local } = source;\n";
    let start = source.find("\"value\"").unwrap() as u32;
    let end = start + "\"value\"".len() as u32;
    let mut plan = AstDependencyRenderPlan::default();
    plan.push_action(AstDependencyAction::raw_ident((start, end).into(), "renamed").unwrap());

    let output = render_ast_dependencies(source, &ModuleType::JsAuto, &plan).unwrap();

    assert_eq!(output, "const { renamed: local } = source;\n");
  }
}
