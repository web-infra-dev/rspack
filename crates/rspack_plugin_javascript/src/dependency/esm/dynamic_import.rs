use rspack_core::{
  create_javascript_visitor, runtime_globals, CodeGeneratable, CodeGeneratableContext,
  CodeGeneratableDeclMappings, CodeGeneratableResult, Dependency, DependencyCategory, DependencyId,
  DependencyType, ErrorSpan, JsAstPath, ModuleDependency, ModuleDependencyExt, ModuleIdentifier,
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
pub struct EsmDynamicImportDependency {
  id: Option<DependencyId>,
  parent_module_identifier: Option<ModuleIdentifier>,
  request: JsWord,
  category: &'static DependencyCategory,
  dependency_type: &'static DependencyType,
  span: Option<ErrorSpan>,

  #[allow(unused)]
  ast_path: JsAstPath,
}

// Do not edit this, as it is used to uniquely identify the dependency.
impl PartialEq for EsmDynamicImportDependency {
  fn eq(&self, other: &Self) -> bool {
    self.parent_module_identifier == other.parent_module_identifier
      && self.request == other.request
      && self.category == other.category
      && self.dependency_type == other.dependency_type
  }
}

// Do not edit this, as it is used to uniquely identify the dependency.
impl std::hash::Hash for EsmDynamicImportDependency {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.parent_module_identifier.hash(state);
    self.request.hash(state);
    self.category.hash(state);
    self.dependency_type.hash(state);
  }
}

impl EsmDynamicImportDependency {
  pub fn new(request: JsWord, span: Option<ErrorSpan>, ast_path: JsAstPath) -> Self {
    Self {
      parent_module_identifier: None,
      request,
      category: &DependencyCategory::Esm,
      dependency_type: &DependencyType::DynamicImport,
      span,
      ast_path,
      id: None,
    }
  }
}

impl Dependency for EsmDynamicImportDependency {
  fn id(&self) -> Option<DependencyId> {
    self.id
  }
  fn set_id(&mut self, id: Option<DependencyId>) {
    self.id = id;
  }
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

impl ModuleDependency for EsmDynamicImportDependency {
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

impl CodeGeneratable for EsmDynamicImportDependency {
  fn generate(
    &self,
    code_generatable_context: &mut CodeGeneratableContext,
  ) -> rspack_error::Result<CodeGeneratableResult> {
    let CodeGeneratableContext {
      compilation,
      runtime_requirements,
      ..
    } = code_generatable_context;
    let mut code_gen = CodeGeneratableResult::default();
    let mut decl_mappings = CodeGeneratableDeclMappings::default();

    if let Some(dependency_id) = self.id() {
      if let Some(referenced_module) = compilation
        .module_graph
        .module_graph_module_by_dependency_id(&dependency_id)
      {
        let module_id = referenced_module.id(&compilation.chunk_graph).to_string();

        {
          let (id, val) = self.decl_mapping(&compilation.module_graph, module_id.clone());
          decl_mappings.insert(id, val);
        }

        // Add interop require to runtime requirements, as dynamic imports have been transformed so `inject_runtime_helper` will not be able to detect this.
        runtime_requirements.insert(runtime_globals::INTEROP_REQUIRE);
        runtime_requirements.insert(runtime_globals::LOAD_CHUNK_WITH_MODULE);

        code_gen.visitors.push(
          create_javascript_visitor!(exact &self.ast_path, visit_mut_call_expr(n: &mut CallExpr) {
            if let Some(import) = n.args.get_mut(0) {
              if import.spread.is_none() && let Expr::Lit(Lit::Str(_)) = import.expr.as_mut() {
                  let call_expr = CallExpr {
                        span: DUMMY_SP,
                        callee: Ident::new(runtime_globals::LOAD_CHUNK_WITH_MODULE.into(), DUMMY_SP).as_callee(),
                        args: vec![Expr::Lit(Lit::Str(Atom::from(&*module_id).into())).as_arg()],
                        type_args: None,
                      };
                  n.callee = MemberExpr {
                    span: DUMMY_SP,
                    obj: Box::new(Expr::Call(CallExpr {
                      span: DUMMY_SP,
                      callee: MemberExpr {
                        span: DUMMY_SP,
                        obj: Box::new(Expr::Call(call_expr)),
                        prop: MemberProp::Ident(Ident::new("then".into(), DUMMY_SP)),
                      }
                      .as_callee(),
                      args: vec![CallExpr {
                        span: DUMMY_SP,
                        callee: MemberExpr {
                          span: DUMMY_SP,
                          obj: Box::new(Expr::Ident(Ident::new(
                            runtime_globals::REQUIRE.into(),
                            DUMMY_SP,
                          ))),
                          prop: MemberProp::Ident(Ident::new("bind".into(), DUMMY_SP)),
                        }
                        .as_callee(),
                        args: vec![
                          Ident::new(runtime_globals::REQUIRE.into(), DUMMY_SP).as_arg(),
                          Lit::Str(Atom::from(&*module_id).into()).as_arg(),
                        ],
                        type_args: None,
                      }
                      .as_arg()],
                      type_args: None,
                    })),
                    prop: MemberProp::Ident(Ident::new("then".into(), DUMMY_SP)),
                  }
                  .as_callee();
                  n.args = vec![MemberExpr {
                    span: DUMMY_SP,
                    obj: Box::new(Expr::Ident(Ident::new(
                      runtime_globals::REQUIRE.into(),
                      DUMMY_SP,
                    ))),
                    prop: MemberProp::Ident(Ident::new(
                      runtime_globals::INTEROP_REQUIRE.into(),
                      DUMMY_SP,
                    )),
                  }
                  .as_arg()];
                };
            }
          }),
        );
      }
    }

    Ok(code_gen.with_decl_mappings(decl_mappings))
  }
}
