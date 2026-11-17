use rspack_error::{Result, error};
use swc_core::{
  common::{FileName, GLOBALS, Globals, Mark, SourceMap, SyntaxContext},
  ecma::{
    ast::Ident,
    parser::{EsSyntax, Syntax, parse_file_as_program},
    transforms::base::resolver,
    visit::{Visit, VisitMutWith, VisitWith},
  },
};

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
pub fn classify_runtime_module_source(source: &str) -> Result<RuntimeModuleSourceKind> {
  if !source.contains("__webpack_")
    && !source.contains("__rspack_context")
    && !source.contains('\\')
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
    let mut visitor = RuntimeReferences {
      unresolved: SyntaxContext::empty().apply_mark(unresolved),
      found: false,
    };
    program.visit_with(&mut visitor);
    Ok(if visitor.found {
      RuntimeModuleSourceKind::NeedsBridge
    } else {
      RuntimeModuleSourceKind::Custom
    })
  })
}

struct RuntimeReferences {
  unresolved: SyntaxContext,
  found: bool,
}
impl Visit for RuntimeReferences {
  fn visit_ident(&mut self, ident: &Ident) {
    self.found |= ident.ctxt == self.unresolved
      && matches!(
        ident.sym.as_ref(),
        "__webpack_require__"
          | "__rspack_context"
          | "__webpack_modules__"
          | "__webpack_module_cache__"
      );
  }
}
