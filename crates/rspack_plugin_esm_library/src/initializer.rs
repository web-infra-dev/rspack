use rspack_core::{
  ConcatenatedModuleInfo, ConcatenatedModuleInitializer, DependencyRange,
  InitializerClassDeclaration, InitializerVariableDeclaration,
  rspack_sources::{ConcatSource, RawStringSource, ReplaceSource, Source},
};
use rspack_util::{SpanExt, atom::Atom, fx_hash::FxHashSet};
use swc_experimental_ecma_ast::{
  Decl, ForHead, GetSpan, ModuleItem, ObjectPatProp, Pat, Program, Stmt, VarDecl, VarDeclOrExpr,
  Visit, VisitWith,
};
use swc_experimental_ecma_semantic::resolver::Semantic;

struct ForHeadCollector {
  declarations: FxHashSet<u32>,
}

impl<'a> Visit<'a> for ForHeadCollector {
  fn visit_for_stmt(&mut self, statement: &swc_experimental_ecma_ast::ForStmt<'a>) {
    if let Some(VarDeclOrExpr::VarDecl(declaration)) = &statement.init {
      self.declarations.insert(declaration.span.real_lo());
    }
    statement.visit_children_with(self);
  }

  fn visit_for_in_stmt(&mut self, statement: &swc_experimental_ecma_ast::ForInStmt<'a>) {
    if let ForHead::VarDecl(declaration) = &statement.left {
      self.declarations.insert(declaration.span.real_lo());
    }
    statement.visit_children_with(self);
  }

  fn visit_for_of_stmt(&mut self, statement: &swc_experimental_ecma_ast::ForOfStmt<'a>) {
    if let ForHead::VarDecl(declaration) = &statement.left {
      self.declarations.insert(declaration.span.real_lo());
    }
    statement.visit_children_with(self);
  }
}

fn collect_pattern_bindings<'a>(
  pattern: &'a Pat<'a>,
  bindings: &mut Vec<&'a swc_experimental_ecma_ast::Ident<'a>>,
) {
  let mut stack = vec![pattern];
  while let Some(pattern) = stack.pop() {
    match pattern {
      Pat::Ident(binding) => bindings.push(&binding.id),
      Pat::Array(array) => stack.extend(array.elems.iter().flatten()),
      Pat::Assign(assign) => stack.push(&assign.left),
      Pat::Object(object) => {
        for property in &object.props {
          match property {
            ObjectPatProp::KeyValue(property) => stack.push(&property.value),
            ObjectPatProp::Assign(property) => bindings.push(&property.key.id),
            ObjectPatProp::Rest(rest) => stack.push(&rest.arg),
          }
        }
      }
      Pat::Rest(rest) => stack.push(&rest.arg),
      Pat::Expr(_) | Pat::Invalid(_) => {}
    }
  }
}

fn contains_only_trivia(source: &str) -> bool {
  let bytes = source.as_bytes();
  let mut cursor = 0;
  while cursor < bytes.len() {
    if bytes[cursor].is_ascii_whitespace() {
      cursor += 1;
    } else if bytes[cursor..].starts_with(b"//") {
      cursor += 2;
      while cursor < bytes.len() && !matches!(bytes[cursor], b'\n' | b'\r') {
        cursor += 1;
      }
    } else if bytes[cursor..].starts_with(b"/*") {
      let Some(end) = source[cursor + 2..].find("*/") else {
        return false;
      };
      cursor += end + 4;
    } else {
      return false;
    }
  }
  true
}

struct VariableAnalyzer<'a> {
  semantic: &'a Semantic,
  for_heads: &'a FxHashSet<u32>,
  bindings: Vec<Atom>,
  seen_bindings: FxHashSet<Atom>,
  declarations: Vec<InitializerVariableDeclaration>,
}

impl VariableAnalyzer<'_> {
  fn add_binding(&mut self, binding: &swc_experimental_ecma_ast::Ident<'_>) {
    let binding = Atom::from(binding.sym.as_str());
    if self.seen_bindings.insert(binding.clone()) {
      self.bindings.push(binding);
    }
  }
}

impl<'a> Visit<'a> for VariableAnalyzer<'_> {
  fn visit_var_decl(&mut self, declaration: &VarDecl<'a>) {
    let mut bindings = Vec::new();
    for declarator in &declaration.decls {
      collect_pattern_bindings(&declarator.name, &mut bindings);
    }

    if bindings
      .iter()
      .any(|binding| self.semantic.node_scope(binding) == self.semantic.top_level_scope_id())
    {
      for binding in bindings {
        if self.semantic.node_scope(binding) == self.semantic.top_level_scope_id() {
          self.add_binding(binding);
        }
      }

      let keyword_len = match declaration.kind {
        swc_experimental_ecma_ast::VarDeclKind::Var
        | swc_experimental_ecma_ast::VarDeclKind::Let => 3,
        swc_experimental_ecma_ast::VarDeclKind::Const => 5,
      };
      let start = declaration.span.real_lo();
      let assignment_end = declaration
        .decls
        .last()
        .map_or(start + keyword_len, |declarator| {
          declarator.init.as_ref().map_or_else(
            || declarator.name.span().real_hi(),
            |init| init.span().real_hi(),
          )
        });
      self.declarations.push(InitializerVariableDeclaration {
        prefix: DependencyRange::new(start, start + keyword_len),
        end: assignment_end,
        in_for_head: self.for_heads.contains(&start),
        needs_asi_boundary: false,
      });
    }

    declaration.visit_children_with(self);
  }
}

pub(super) fn analyze_initializer(
  program: &Program<'_>,
  semantic: &Semantic,
  source: &str,
) -> ConcatenatedModuleInitializer {
  let mut for_head_collector = ForHeadCollector {
    declarations: FxHashSet::default(),
  };
  program.visit_with(&mut for_head_collector);

  let mut variable_analyzer = VariableAnalyzer {
    semantic,
    for_heads: &for_head_collector.declarations,
    bindings: Vec::new(),
    seen_bindings: FxHashSet::default(),
    declarations: Vec::new(),
  };
  program.visit_with(&mut variable_analyzer);

  for index in 0..variable_analyzer.declarations.len() {
    let (previous, current) = variable_analyzer.declarations.split_at_mut(index);
    let current = &mut current[0];
    if current.in_for_head {
      continue;
    }
    let start = current.prefix.start as usize;
    let prefix = source.get(..start).unwrap_or_default();
    let follows_rewritten_declaration = previous
      .iter()
      .rev()
      .find(|declaration| !declaration.in_for_head && declaration.end <= current.prefix.start)
      .is_some_and(|declaration| {
        source
          .get(declaration.end as usize..start)
          .is_some_and(contains_only_trivia)
      });
    current.needs_asi_boundary = !contains_only_trivia(prefix)
      && !prefix.trim_end().ends_with(';')
      && !follows_rewritten_declaration;
  }

  let mut function_declarations = Vec::new();
  let mut class_declarations = Vec::new();
  if let Program::Module(module) = program {
    for item in &module.body {
      let ModuleItem::Stmt(statement) = item else {
        continue;
      };
      let Stmt::Decl(declaration) = statement.as_ref() else {
        continue;
      };
      match declaration.as_ref() {
        Decl::Fn(function) => function_declarations.push(function.span().into()),
        Decl::Class(class) => {
          let binding = Atom::from(class.ident.sym.as_str());
          if variable_analyzer.seen_bindings.insert(binding.clone()) {
            variable_analyzer.bindings.push(binding.clone());
          }
          class_declarations.push(InitializerClassDeclaration {
            start: class.span().real_lo(),
            end: class.span().real_hi(),
            binding,
          });
        }
        Decl::Var(_) | Decl::Using(_) => {}
      }
    }
  }

  ConcatenatedModuleInitializer {
    name: None,
    bindings: variable_analyzer.bindings,
    function_declarations,
    variable_declarations: variable_analyzer.declarations,
    class_declarations,
  }
}

fn generated_offset(source: &ReplaceSource, original: u32) -> u32 {
  let mut delta = 0i64;
  for replacement in source.replacements() {
    if replacement.end() > original {
      break;
    }
    delta += replacement.content().len() as i64
      - i64::from(replacement.end().saturating_sub(replacement.start()));
  }
  (i64::from(original) + delta)
    .try_into()
    .expect("generated module offset should be non-negative")
}

fn render_hoisted_functions(
  rendered: &ReplaceSource,
  ranges: &[DependencyRange],
) -> Option<ReplaceSource> {
  if ranges.is_empty() {
    return None;
  }

  let mut generated_ranges = ranges
    .iter()
    .map(|range| {
      (
        generated_offset(rendered, range.start),
        generated_offset(rendered, range.end),
      )
    })
    .collect::<Vec<_>>();
  generated_ranges.sort_unstable();

  let generated_len: u32 = rendered
    .source()
    .into_string_lossy()
    .len()
    .try_into()
    .expect("generated module source should fit in u32");
  let mut functions = ReplaceSource::new(rendered.clone());
  let mut cursor = 0;
  for (start, end) in generated_ranges {
    if cursor < start {
      functions.replace(cursor, start, String::new(), None);
    }
    cursor = end;
  }
  if cursor < generated_len {
    functions.replace(cursor, generated_len, String::new(), None);
  }
  Some(functions)
}

pub(super) fn render_initializer(
  info: &ConcatenatedModuleInfo,
  rendered: ReplaceSource,
  helper: &Atom,
  dependency_initializers: String,
  is_async: bool,
) -> ConcatSource {
  let initializer = info
    .initializer
    .as_ref()
    .expect("initializer render requires an initializer plan");
  let initializer_name = initializer
    .name
    .as_ref()
    .expect("initializer should be named during deconfliction");

  let mut source = ConcatSource::default();
  if !initializer.bindings.is_empty() {
    let bindings = initializer
      .bindings
      .iter()
      .map(|binding| info.get_internal_name(binding).unwrap_or(binding).as_str())
      .collect::<Vec<_>>()
      .join(", ");
    source.add(RawStringSource::from(format!("var {bindings};\n")));
  }
  if let Some(functions) = render_hoisted_functions(&rendered, &initializer.function_declarations) {
    source.add(functions);
    source.add(RawStringSource::from_static("\n"));
  }

  let mut body = ReplaceSource::new(rendered.clone());
  for function in &initializer.function_declarations {
    body.replace(
      generated_offset(&rendered, function.start),
      generated_offset(&rendered, function.end),
      String::new(),
      None,
    );
  }
  for declaration in &initializer.variable_declarations {
    let start = generated_offset(&rendered, declaration.prefix.start);
    let end = generated_offset(&rendered, declaration.prefix.end);
    body.replace(
      start,
      end,
      if declaration.in_for_head {
        String::new()
      } else {
        // A declaration establishes its own statement boundary. Preserve it
        // when the preceding generated statement relies on ASI; otherwise the
        // rewritten assignment could become a call on that statement's value.
        if declaration.needs_asi_boundary {
          ";(".to_string()
        } else {
          "(".to_string()
        }
      },
      None,
    );
    if !declaration.in_for_head {
      let end = generated_offset(&rendered, declaration.end);
      // The original declaration may rely on ASI. Once `const`/`let`/`var`
      // becomes a parenthesized assignment, the next parenthesized statement
      // would otherwise be parsed as a call on this assignment's value.
      body.insert(end, ");".to_string(), None);
    }
  }
  for declaration in &initializer.class_declarations {
    let binding = info
      .get_internal_name(&declaration.binding)
      .unwrap_or(&declaration.binding);
    body.insert(
      generated_offset(&rendered, declaration.start),
      format!("{binding} = "),
      None,
    );
    body.insert(
      generated_offset(&rendered, declaration.end),
      ";".to_string(),
      None,
    );
  }

  source.add(RawStringSource::from(format!(
    "var {initializer_name} = /*#__PURE__*/ {helper}({}() => {{\n{dependency_initializers}",
    if is_async { "async " } else { "" }
  )));
  source.add(body);
  source.add(RawStringSource::from_static("\n});\n"));
  source
}
