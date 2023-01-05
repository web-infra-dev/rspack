use derivative::Derivative;

use swc_core::ecma::atoms::JsWord;
use swc_core::ecma::{ast::*, atoms::Atom};

use crate::{
  create_javascript_visitor,
  dependency::{
    CodeGeneratable, CodeGeneratableContext, CodeGeneratableResult, Dependency, DependencyCategory,
    ModuleDependency,
  },
  DependencyType, ErrorSpan, JsAstPath, ModuleIdentifier,
};

#[derive(Derivative)]
#[derivative(Debug, Hash, PartialEq, Eq, Clone)]
pub struct EsmImportDependency {
  parent_module_identifier: Option<ModuleIdentifier>,
  request: JsWord,
  // user_request: String,
  #[derivative(PartialEq = "ignore")]
  #[derivative(Hash = "ignore")]
  span: Option<ErrorSpan>,

  #[derivative(PartialEq = "ignore")]
  #[derivative(Hash = "ignore")]
  ast_path: JsAstPath,
}

impl EsmImportDependency {
  pub fn new(request: JsWord, span: Option<ErrorSpan>, ast_path: JsAstPath) -> Self {
    Self {
      parent_module_identifier: None,
      request,
      // user_request,
      span,
      ast_path,
    }
  }
}

impl Dependency for EsmImportDependency {
  fn parent_module_identifier(&self) -> Option<&ModuleIdentifier> {
    self.parent_module_identifier.as_ref()
  }

  fn set_parent_module_identifier(&mut self, module_identifier: Option<ModuleIdentifier>) {
    self.parent_module_identifier = module_identifier;
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::Import
  }
}

impl ModuleDependency for EsmImportDependency {
  fn request(&self) -> &str {
    &*self.request
  }

  fn user_request(&self) -> &str {
    &*self.request
  }

  fn span(&self) -> Option<&ErrorSpan> {
    self.span.as_ref()
  }
}

static SWC_HELPERS_REG: once_cell::sync::Lazy<regex::Regex> =
  once_cell::sync::Lazy::new(|| regex::Regex::new(r"@swc/helpers/lib/(\w*)\.js$").expect("TODO:"));

impl CodeGeneratable for EsmImportDependency {
  fn generate<'s: 'cx, 'cx>(
    &'s self,
    code_generatable_context: &'cx CodeGeneratableContext,
  ) -> CodeGeneratableResult<'s> {
    let CodeGeneratableContext {
      compilation,
      module,
    } = code_generatable_context;

    let mut code_gen = CodeGeneratableResult::default();

    let v = create_javascript_visitor!(&self.ast_path, visit_mut_call_expr(n: &mut CallExpr) {
        if let Callee::Expr(box Expr::Ident(_ident)) = &mut n.callee {
          if let Some(ExprOrSpread {
            spread: None,
            expr: box Expr::Lit(Lit::Str(str)),
          }) = n.args.first_mut()
          {
            // swc will automatically replace @swc/helpers/src/xx.mjs with @swc/helpers/lib/xx.js when it transform code to commonjs
            // so we need replace it to original specifier to find module
            // this is a temporary solution
            let specifier = match SWC_HELPERS_REG.captures(&str.value) {
              Some(cap) => match cap.get(1) {
                Some(cap) => format!(r#"@swc/helpers/src/{}.mjs"#, cap.as_str()),
                None => str.value.to_string(),
              },
              None => str.value.to_string(),
            };

            let target_mgm = compilation.module_graph
              .module_graph_module_by_identifier(&module.identifier())
              .and_then(|mgm| {
                mgm.dependencies.iter().find_map(|dep| {
                  if dep.request() == self.request() && dep.dependency_type() == self.dependency_type() {
                    compilation
                      .module_graph
                      .module_by_dependency(dep)
                  } else {
                    None
                  }
                })
              }).expect("Failed to get module graph module");

            let module_id = target_mgm.id(&compilation.chunk_graph);
            str.value = JsWord::from(module_id);
            str.raw = Some(Atom::from(format!("\"{}\"", module_id)));
          };
      }
    });

    code_gen.visitors.push(v);

    code_gen
  }
}
