use rspack_core::{
  create_javascript_visitor, runtime_globals, CodeGeneratable, CodeGeneratableContext,
  CodeGeneratableResult, Dependency, DependencyCategory, DependencyType, ErrorSpan, JsAstPath,
  ModuleDependency, ModuleDependencyExt, ModuleIdentifier,
};
use swc_core::{
  common::DUMMY_SP,
  ecma::{
    ast::*,
    atoms::{Atom, JsWord},
    utils::ExprFactory,
  },
};

#[derive(Debug, Eq, Clone)]
pub struct ModuleHotAcceptDependency {
  parent_module_identifier: Option<ModuleIdentifier>,
  request: JsWord,
  // user_request: String,
  category: &'static DependencyCategory,
  dependency_type: &'static DependencyType,

  span: Option<ErrorSpan>,
  #[allow(unused)]
  ast_path: JsAstPath,
}

// Do not edit this, as it is used to uniquely identify the dependency.
impl PartialEq for ModuleHotAcceptDependency {
  fn eq(&self, other: &Self) -> bool {
    self.parent_module_identifier == other.parent_module_identifier
      && self.request == other.request
      && self.category == other.category
      && self.dependency_type == other.dependency_type
  }
}

// Do not edit this, as it is used to uniquely identify the dependency.
impl std::hash::Hash for ModuleHotAcceptDependency {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.parent_module_identifier.hash(state);
    self.request.hash(state);
    self.category.hash(state);
    self.dependency_type.hash(state);
  }
}

impl ModuleHotAcceptDependency {
  pub fn new(request: JsWord, span: Option<ErrorSpan>, ast_path: JsAstPath) -> Self {
    Self {
      parent_module_identifier: None,
      request,
      category: &DependencyCategory::CommonJS,
      dependency_type: &DependencyType::ModuleHotAccept,
      span,
      ast_path,
    }
  }
}

impl Dependency for ModuleHotAcceptDependency {
  fn parent_module_identifier(&self) -> Option<&ModuleIdentifier> {
    self.parent_module_identifier.as_ref()
  }

  fn set_parent_module_identifier(&mut self, module_identifier: Option<ModuleIdentifier>) {
    self.parent_module_identifier = module_identifier;
  }

  fn category(&self) -> &DependencyCategory {
    self.category
  }

  fn dependency_type(&self) -> &DependencyType {
    self.dependency_type
  }
}

impl ModuleDependency for ModuleHotAcceptDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn span(&self) -> Option<&ErrorSpan> {
    self.span.as_ref()
  }
}

impl CodeGeneratable for ModuleHotAcceptDependency {
  fn generate(
    &self,
    code_generatable_context: &mut CodeGeneratableContext,
  ) -> rspack_error::Result<CodeGeneratableResult> {
    let CodeGeneratableContext { compilation, .. } = code_generatable_context;
    let mut code_gen = CodeGeneratableResult::default();

    let referenced_module = self.referencing_module_graph_module(&compilation.module_graph);

    if let Some(referenced_module) = referenced_module {
      let module_id = referenced_module.id(&compilation.chunk_graph).to_string();

      code_gen.visitors.push(
        create_javascript_visitor!(exact &self.ast_path, visit_mut_call_expr(n: &mut CallExpr) {
          let mut accpet_module_id: String = Default::default();

          if let Some(Lit::Str(str)) = n
            .args
            .get_mut(0)
            .and_then(|first_arg| first_arg.expr.as_mut_lit())
          {
            str.value = JsWord::from(&*module_id);
            str.raw = Some(Atom::from(format!("\"{module_id}\"")));
            accpet_module_id = module_id.to_string();
          }

          // TODO: add assign expr with module require
          // module.hot.accept without callback
          if !accpet_module_id.is_empty() && n.args.len() == 1 {
            n.args.push(
              FnExpr {
                ident: None,
                function: Box::new(Function {
                  span: DUMMY_SP,
                  decorators: Default::default(),
                  is_async: false,
                  is_generator: false,
                  params: vec![],
                  body: Some(BlockStmt {
                    span: DUMMY_SP,
                    stmts: vec![CallExpr {
                      span: DUMMY_SP,
                      callee: Ident::new(runtime_globals::REQUIRE.into(), DUMMY_SP).as_callee(),
                      args: vec![Lit::Str(accpet_module_id.into()).as_arg()],
                      type_args: None,
                    }
                    .into_stmt()],
                  }),
                  type_params: None,
                  return_type: None,
                }),
              }
              .as_arg(),
            );
          }
        }),
      );
    }

    Ok(code_gen)
  }
}
