mod css_syntax;
mod dependencies;
mod dependency_types;
mod lexer;

pub use dependencies::{DashedIdentCollector, LexDependencies, ModeData};
pub use dependency_types::{
  Dependency, DependencyContext, DependencyIndex, DependencyListRange, ImportAttributes, Mode,
  Range, UrlRangeKind, ValueAtRuleImportItem, Warning, WarningKind,
};
pub use lexer::{Lexer, LexerVisitor, Pos, Token, TokenFlags, TokenKind, TokenWithTrivia, Trivia};

pub trait HandleWarning<'s> {
  fn handle_warning(&mut self, warning: Warning<'s>);
}

impl<'s, F: FnMut(Warning<'s>)> HandleWarning<'s> for F {
  fn handle_warning(&mut self, warning: Warning<'s>) {
    self(warning);
  }
}

pub fn lex_dependencies<'s>(
  input: &'s str,
  mode: Mode,
  mut handle_dependency: impl FnMut(&Dependency<'s>),
  handle_warning: impl HandleWarning<'s>,
) -> DependencyContext<'s> {
  let mut lexer = Lexer::new(input, DashedIdentCollector::default());
  let mut visitor = LexDependencies::new(handle_warning, mode);
  visitor.lex_streaming(&mut lexer);
  let dependency_context = visitor.into_dependency_context();
  for dependency in dependency_context.iter() {
    handle_dependency(dependency);
  }
  dependency_context
}

pub fn collect_dependencies(input: &str, mode: Mode) -> (DependencyContext<'_>, Vec<Warning<'_>>) {
  let mut warnings = Vec::with_capacity(estimate_warning_capacity(input.len(), mode));
  let mut lexer = Lexer::new(input, DashedIdentCollector::default());
  let mut visitor = LexDependencies::new(|v| warnings.push(v), mode);
  visitor.lex_streaming(&mut lexer);
  (visitor.into_dependency_context(), warnings)
}

fn estimate_warning_capacity(input_len: usize, mode: Mode) -> usize {
  let warning_rate = match mode {
    Mode::Pure => (1, 256),
    _ => (1, 512),
  };

  estimate_capacity(input_len, warning_rate.0, warning_rate.1, 2, 512)
}

#[inline]
fn estimate_capacity(
  input_len: usize,
  numerator: usize,
  denominator: usize,
  minimum: usize,
  maximum: usize,
) -> usize {
  input_len
    .saturating_mul(numerator)
    .checked_div(denominator)
    .unwrap_or(0)
    .clamp(minimum, maximum)
}
