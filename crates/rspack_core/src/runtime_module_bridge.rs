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

use crate::{Compilation, RuntimeModule};

/// Whether a chunk has plugin-authored source and whether that source needs legacy names.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, rspack_hash::RspackHash)]
pub enum RuntimeModuleSourceKind {
  #[default]
  Native,
  Custom,
  NeedsBridge,
}

impl dyn RuntimeModule + '_ {
  pub(crate) async fn source_kind(
    &self,
    compilation: &Compilation,
  ) -> Result<RuntimeModuleSourceKind> {
    if let Some(source) = self.get_custom_source() {
      return classify_runtime_module_source(&source);
    }
    if let Some(source) = self.generated_source_for_bridge(compilation) {
      let source = source.await?;
      return classify_runtime_module_source(&source.source().into_string_lossy());
    }
    Ok(RuntimeModuleSourceKind::Native)
  }
}

/// Inspect extension code without rewriting it. Once a runtime object escapes, its keys
/// can be computed anywhere (including in another module); a list of member expressions
/// is not a sound description of the surface the plugin can observe.
fn classify_runtime_module_source(source: &str) -> Result<RuntimeModuleSourceKind> {
  let may_have_escaped_names = source.contains('\\');
  if !source.contains("__webpack_")
    && !source.contains("__rspack_context")
    && !may_have_escaped_names
  {
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
    .map_err(|e| error!("Cannot parse runtime module: {:?}", e.kind()))?;
    if let Some(e) = errors.first() {
      return Err(error!("Cannot parse runtime module: {:?}", e.kind()));
    }
    let unresolved = Mark::new();
    program.visit_mut_with(&mut resolver(unresolved, Mark::new(), false));
    let ctxt = SyntaxContext::empty().apply_mark(unresolved);
    let uses_runtime = [
      "__webpack_require__",
      "__rspack_context",
      "__webpack_modules__",
      "__webpack_module_cache__",
    ]
    .into_iter()
    .any(|name| {
      (may_have_escaped_names || source.contains(name))
        && IdentUsageFinder::find(&Ident::new(name.into(), DUMMY_SP, ctxt), &program)
    });
    Ok(if uses_runtime {
      RuntimeModuleSourceKind::NeedsBridge
    } else {
      RuntimeModuleSourceKind::Custom
    })
  })
}
