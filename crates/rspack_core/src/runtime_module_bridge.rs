use rspack_error::{Result, error};
use swc_core::{
  common::{DUMMY_SP, FileName, GLOBALS, Globals, Mark, SourceMap, SyntaxContext},
  ecma::{
    ast::Ident,
    parser::{EsSyntax, Syntax, parse_file_as_program},
    transforms::base::resolver,
    utils::IdentUsageFinder,
    visit::VisitMutWith,
  },
};

use crate::{RuntimeVariable, runtime_variable_name};

/// Whether a chunk has plugin-authored source and whether that source needs legacy names.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, rspack_hash::RspackHash)]
pub enum RuntimeModuleSourceKind {
  #[default]
  Native,
  Custom,
  NeedsBridge,
}

/// Inspect extension code without rewriting it. Once a runtime object escapes, its keys
/// can be computed anywhere (including in another module); a list of member expressions
/// is not a sound description of the surface the plugin can observe.
pub(crate) fn classify_runtime_module_source(source: &str) -> Result<RuntimeModuleSourceKind> {
  let runtime_names = [
    RuntimeVariable::Require,
    RuntimeVariable::Context,
    RuntimeVariable::Modules,
    RuntimeVariable::ModuleCache,
  ]
  .map(|variable| runtime_variable_name(&variable));
  if !source.contains('\\') && !runtime_names.iter().any(|&name| source.contains(name)) {
    return Ok(RuntimeModuleSourceKind::Custom);
  }
  GLOBALS.set(&Globals::new(), || {
    let cm = SourceMap::default();
    let file = cm.new_source_file(FileName::Anon.into(), source.to_owned());
    let mut errors = Vec::new();
    let mut program = parse_file_as_program(
      &file,
      Syntax::Es(EsSyntax {
        allow_return_outside_function: true,
        ..Default::default()
      }),
      Default::default(),
      None,
      &mut errors,
    )
    .map_err(|e| error!("Cannot parse runtime module: {}", e.kind().msg()))?;
    if let Some(e) = errors.first() {
      return Err(error!("Cannot parse runtime module: {}", e.kind().msg()));
    }
    let unresolved = Mark::new();
    program.visit_mut_with(&mut resolver(unresolved, Mark::new(), false));
    let ctxt = SyntaxContext::empty().apply_mark(unresolved);
    let uses_runtime = runtime_names
      .into_iter()
      .any(|name| IdentUsageFinder::find(&Ident::new(name.into(), DUMMY_SP, ctxt), &program));
    Ok(if uses_runtime {
      RuntimeModuleSourceKind::NeedsBridge
    } else {
      RuntimeModuleSourceKind::Custom
    })
  })
}
