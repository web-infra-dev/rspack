use std::any::Any;
use std::fmt::{Debug, Formatter};

use swc_core::common::collections::AHashMap as HashMap;
use swc_core::common::Span;
use swc_core::ecma::ast::*;
use swc_core::ecma::visit::{VisitMut, VisitMutWith};

pub struct AstSpanRepairer {
  pub(crate) span_map: HashMap<String, Span>,
  pub(crate) span_map0: HashMap<Span, Span>,
}

impl AstSpanRepairer {
  pub(crate) fn get_span(&mut self, ptr: *const dyn Any) -> Option<Span> {
    let mut buf = String::new();
    let mut formatter = Formatter::new(&mut buf);
    ptr.fmt(&mut formatter).unwrap();

    if let Some(new_span) = self.span_map.get(&buf) {
      return Some(*new_span);
    }
    None
  }
}

impl VisitMut for AstSpanRepairer {
  #[inline]
  fn visit_mut_span(&mut self, span: &mut Span) {
    if let Some(new_span) = self.span_map0.get(span) {
      *span = *new_span;
    }
  }

  #[inline]
  fn visit_mut_array_lit(&mut self, node: &mut ArrayLit) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };

    <ArrayLit as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_array_pat(&mut self, node: &mut ArrayPat) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <ArrayPat as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_arrow_expr(&mut self, node: &mut ArrowExpr) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <ArrowExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_assign_expr(&mut self, node: &mut AssignExpr) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <AssignExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_assign_pat(&mut self, node: &mut AssignPat) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <AssignPat as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_assign_pat_prop(&mut self, node: &mut AssignPatProp) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <AssignPatProp as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_assign_prop(&mut self, node: &mut AssignProp) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <AssignProp as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_auto_accessor(&mut self, node: &mut AutoAccessor) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <AutoAccessor as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_await_expr(&mut self, node: &mut AwaitExpr) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <AwaitExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_big_int(&mut self, node: &mut BigInt) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <BigInt as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_bin_expr(&mut self, node: &mut BinExpr) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <BinExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_binding_ident(&mut self, node: &mut BindingIdent) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <BindingIdent as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_block_stmt(&mut self, node: &mut BlockStmt) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <BlockStmt as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_bool(&mut self, node: &mut Bool) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <Bool as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_break_stmt(&mut self, node: &mut BreakStmt) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <BreakStmt as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_call_expr(&mut self, node: &mut CallExpr) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <CallExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_catch_clause(&mut self, node: &mut CatchClause) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <CatchClause as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_class(&mut self, node: &mut Class) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <Class as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_class_method(&mut self, node: &mut ClassMethod) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <ClassMethod as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_class_prop(&mut self, node: &mut ClassProp) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <ClassProp as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_computed_prop_name(&mut self, node: &mut ComputedPropName) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <ComputedPropName as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_cond_expr(&mut self, node: &mut CondExpr) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <CondExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_constructor(&mut self, node: &mut Constructor) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <Constructor as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_continue_stmt(&mut self, node: &mut ContinueStmt) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <ContinueStmt as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_debugger_stmt(&mut self, node: &mut DebuggerStmt) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <DebuggerStmt as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_decorator(&mut self, node: &mut Decorator) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <Decorator as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_decorators(&mut self, nodes: &mut Vec<Decorator>) {
    for node in nodes.iter_mut() {
      if let Some(span) = self.get_span(node) {
        node.span = span;
      };
    }
    <Vec<Decorator> as VisitMutWith<Self>>::visit_mut_children_with(nodes, self)
  }

  #[inline]
  fn visit_mut_do_while_stmt(&mut self, node: &mut DoWhileStmt) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <DoWhileStmt as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_empty_stmt(&mut self, node: &mut EmptyStmt) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <EmptyStmt as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_export_all(&mut self, node: &mut ExportAll) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <ExportAll as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_export_decl(&mut self, node: &mut ExportDecl) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <ExportDecl as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_export_default_decl(&mut self, node: &mut ExportDefaultDecl) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <ExportDefaultDecl as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_export_default_expr(&mut self, node: &mut ExportDefaultExpr) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <ExportDefaultExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_export_named_specifier(&mut self, node: &mut ExportNamedSpecifier) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <ExportNamedSpecifier as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_export_namespace_specifier(&mut self, node: &mut ExportNamespaceSpecifier) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <ExportNamespaceSpecifier as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_expr_stmt(&mut self, node: &mut ExprStmt) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <ExprStmt as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_for_in_stmt(&mut self, node: &mut ForInStmt) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <ForInStmt as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_for_of_stmt(&mut self, node: &mut ForOfStmt) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <ForOfStmt as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_for_stmt(&mut self, node: &mut ForStmt) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <ForStmt as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_function(&mut self, node: &mut Function) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <Function as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_getter_prop(&mut self, node: &mut GetterProp) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <GetterProp as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ident(&mut self, node: &mut Ident) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <Ident as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ident_name(&mut self, node: &mut IdentName) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <IdentName as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_if_stmt(&mut self, node: &mut IfStmt) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <IfStmt as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_import(&mut self, node: &mut Import) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <Import as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_import_decl(&mut self, node: &mut ImportDecl) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <ImportDecl as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_import_default_specifier(&mut self, node: &mut ImportDefaultSpecifier) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <ImportDefaultSpecifier as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_import_named_specifier(&mut self, node: &mut ImportNamedSpecifier) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <ImportNamedSpecifier as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_import_star_as_specifier(&mut self, node: &mut ImportStarAsSpecifier) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <ImportStarAsSpecifier as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_import_with(&mut self, node: &mut ImportWith) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <ImportWith as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_invalid(&mut self, node: &mut Invalid) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <Invalid as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_jsx_attr(&mut self, node: &mut JSXAttr) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <JSXAttr as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_jsx_closing_element(&mut self, node: &mut JSXClosingElement) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <JSXClosingElement as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_jsx_closing_fragment(&mut self, node: &mut JSXClosingFragment) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <JSXClosingFragment as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_jsx_element(&mut self, node: &mut JSXElement) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <JSXElement as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_jsx_empty_expr(&mut self, node: &mut JSXEmptyExpr) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <JSXEmptyExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_jsx_expr_container(&mut self, node: &mut JSXExprContainer) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <JSXExprContainer as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_jsx_fragment(&mut self, node: &mut JSXFragment) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <JSXFragment as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_jsx_member_expr(&mut self, node: &mut JSXMemberExpr) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <JSXMemberExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_jsx_namespaced_name(&mut self, node: &mut JSXNamespacedName) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <JSXNamespacedName as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_jsx_opening_element(&mut self, node: &mut JSXOpeningElement) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <JSXOpeningElement as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_jsx_opening_fragment(&mut self, node: &mut JSXOpeningFragment) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <JSXOpeningFragment as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_jsx_spread_child(&mut self, node: &mut JSXSpreadChild) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <JSXSpreadChild as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_jsx_text(&mut self, node: &mut JSXText) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <JSXText as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_labeled_stmt(&mut self, node: &mut LabeledStmt) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <LabeledStmt as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_member_expr(&mut self, node: &mut MemberExpr) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <MemberExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_meta_prop_expr(&mut self, node: &mut MetaPropExpr) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <MetaPropExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_module(&mut self, node: &mut Module) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <Module as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_named_export(&mut self, node: &mut NamedExport) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <NamedExport as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_new_expr(&mut self, node: &mut NewExpr) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <NewExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_null(&mut self, node: &mut Null) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <Null as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_number(&mut self, node: &mut Number) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <Number as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_object_lit(&mut self, node: &mut ObjectLit) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <ObjectLit as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_object_pat(&mut self, node: &mut ObjectPat) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <ObjectPat as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_opt_block_stmt(&mut self, node: &mut Option<BlockStmt>) {
    if let Some(node) = node {
      if let Some(span) = self.get_span(node) {
        node.span = span;
      };
    }
    <Option<BlockStmt> as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_opt_call(&mut self, node: &mut OptCall) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <OptCall as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_opt_catch_clause(&mut self, node: &mut Option<CatchClause>) {
    if let Some(node) = node {
      if let Some(span) = self.get_span(node) {
        node.span = span;
      };
    }
    <Option<CatchClause> as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_opt_chain_expr(&mut self, node: &mut OptChainExpr) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <OptChainExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_opt_ident(&mut self, node: &mut Option<Ident>) {
    if let Some(node) = node {
      if let Some(span) = self.get_span(node) {
        node.span = span;
      };
    }
    <Option<Ident> as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_opt_jsx_closing_element(&mut self, node: &mut Option<JSXClosingElement>) {
    if let Some(node) = node {
      if let Some(span) = self.get_span(node) {
        node.span = span;
      };
    }
    <Option<JSXClosingElement> as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_opt_object_lit(&mut self, node: &mut Option<Box<ObjectLit>>) {
    if let Some(node) = node {
      if let Some(span) = self.get_span(node) {
        node.span = span;
      };
    }
    <Option<Box<ObjectLit>> as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_opt_str(&mut self, node: &mut Option<Box<Str>>) {
    if let Some(node) = node {
      if let Some(span) = self.get_span(node) {
        node.span = span;
      };
    }
    <Option<Box<Str>> as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_opt_ts_type_ann(&mut self, node: &mut Option<Box<TsTypeAnn>>) {
    if let Some(node) = node {
      if let Some(span) = self.get_span(node) {
        node.span = span;
      };
    }

    <Option<Box<TsTypeAnn>> as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_opt_ts_type_param_decl(&mut self, node: &mut Option<Box<TsTypeParamDecl>>) {
    if let Some(node) = node {
      if let Some(span) = self.get_span(node) {
        node.span = span;
      };
    }

    <Option<Box<TsTypeParamDecl>> as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_opt_ts_type_param_instantiation(
    &mut self,
    node: &mut Option<Box<TsTypeParamInstantiation>>,
  ) {
    if let Some(node) = node {
      if let Some(span) = self.get_span(node) {
        node.span = span;
      };
    }

    <Option<Box<TsTypeParamInstantiation>> as VisitMutWith<Self>>::visit_mut_children_with(
      node, self,
    )
  }

  #[inline]
  fn visit_mut_param(&mut self, node: &mut Param) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <Param as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_params(&mut self, nodes: &mut Vec<Param>) {
    for node in nodes.iter_mut() {
      if let Some(span) = self.get_span(node) {
        node.span = span;
      };
    }

    <Vec<Param> as VisitMutWith<Self>>::visit_mut_children_with(nodes, self)
  }

  #[inline]
  fn visit_mut_paren_expr(&mut self, node: &mut ParenExpr) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <ParenExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_private_method(&mut self, node: &mut PrivateMethod) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <PrivateMethod as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_private_name(&mut self, node: &mut PrivateName) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <PrivateName as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_private_prop(&mut self, node: &mut PrivateProp) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <PrivateProp as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_regex(&mut self, node: &mut Regex) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <Regex as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_rest_pat(&mut self, node: &mut RestPat) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <RestPat as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_return_stmt(&mut self, node: &mut ReturnStmt) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <ReturnStmt as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_script(&mut self, node: &mut Script) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <Script as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_seq_expr(&mut self, node: &mut SeqExpr) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <SeqExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_setter_prop(&mut self, node: &mut SetterProp) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <SetterProp as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_static_block(&mut self, node: &mut StaticBlock) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <StaticBlock as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_str(&mut self, node: &mut Str) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <Str as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_super(&mut self, node: &mut Super) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <Super as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_super_prop_expr(&mut self, node: &mut SuperPropExpr) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <SuperPropExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_switch_case(&mut self, node: &mut SwitchCase) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <SwitchCase as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_switch_cases(&mut self, nodes: &mut Vec<SwitchCase>) {
    for node in nodes.iter_mut() {
      if let Some(span) = self.get_span(node) {
        node.span = span;
      };
    }

    <Vec<SwitchCase> as VisitMutWith<Self>>::visit_mut_children_with(nodes, self)
  }

  #[inline]
  fn visit_mut_switch_stmt(&mut self, node: &mut SwitchStmt) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <SwitchStmt as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_tagged_tpl(&mut self, node: &mut TaggedTpl) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TaggedTpl as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_this_expr(&mut self, node: &mut ThisExpr) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <ThisExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_throw_stmt(&mut self, node: &mut ThrowStmt) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <ThrowStmt as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_tpl(&mut self, node: &mut Tpl) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <Tpl as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_tpl_element(&mut self, node: &mut TplElement) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TplElement as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_tpl_elements(&mut self, nodes: &mut Vec<TplElement>) {
    for node in nodes.iter_mut() {
      if let Some(span) = self.get_span(node) {
        node.span = span;
      };
    }

    <Vec<TplElement> as VisitMutWith<Self>>::visit_mut_children_with(nodes, self)
  }

  #[inline]
  fn visit_mut_try_stmt(&mut self, node: &mut TryStmt) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TryStmt as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_array_type(&mut self, node: &mut TsArrayType) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsArrayType as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_as_expr(&mut self, node: &mut TsAsExpr) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsAsExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_call_signature_decl(&mut self, node: &mut TsCallSignatureDecl) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsCallSignatureDecl as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_conditional_type(&mut self, node: &mut TsConditionalType) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsConditionalType as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_const_assertion(&mut self, node: &mut TsConstAssertion) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsConstAssertion as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_construct_signature_decl(&mut self, node: &mut TsConstructSignatureDecl) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsConstructSignatureDecl as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_constructor_type(&mut self, node: &mut TsConstructorType) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsConstructorType as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_enum_decl(&mut self, node: &mut TsEnumDecl) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsEnumDecl as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_enum_member(&mut self, node: &mut TsEnumMember) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsEnumMember as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_export_assignment(&mut self, node: &mut TsExportAssignment) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsExportAssignment as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_expr_with_type_args(&mut self, node: &mut TsExprWithTypeArgs) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsExprWithTypeArgs as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_expr_with_type_argss(&mut self, nodes: &mut Vec<TsExprWithTypeArgs>) {
    for node in nodes.iter_mut() {
      if let Some(span) = self.get_span(node) {
        node.span = span;
      };
    }
    <Vec<TsExprWithTypeArgs> as VisitMutWith<Self>>::visit_mut_children_with(nodes, self)
  }

  #[inline]
  fn visit_mut_ts_external_module_ref(&mut self, node: &mut TsExternalModuleRef) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsExternalModuleRef as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_fn_type(&mut self, node: &mut TsFnType) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsFnType as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_getter_signature(&mut self, node: &mut TsGetterSignature) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsGetterSignature as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_import_equals_decl(&mut self, node: &mut TsImportEqualsDecl) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsImportEqualsDecl as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_import_type(&mut self, node: &mut TsImportType) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsImportType as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_index_signature(&mut self, node: &mut TsIndexSignature) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsIndexSignature as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_indexed_access_type(&mut self, node: &mut TsIndexedAccessType) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsIndexedAccessType as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_infer_type(&mut self, node: &mut TsInferType) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsInferType as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_instantiation(&mut self, node: &mut TsInstantiation) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsInstantiation as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_interface_body(&mut self, node: &mut TsInterfaceBody) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsInterfaceBody as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_interface_decl(&mut self, node: &mut TsInterfaceDecl) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsInterfaceDecl as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_intersection_type(&mut self, node: &mut TsIntersectionType) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsIntersectionType as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_keyword_type(&mut self, node: &mut TsKeywordType) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsKeywordType as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_lit_type(&mut self, node: &mut TsLitType) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsLitType as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_mapped_type(&mut self, node: &mut TsMappedType) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsMappedType as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_method_signature(&mut self, node: &mut TsMethodSignature) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsMethodSignature as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_module_block(&mut self, node: &mut TsModuleBlock) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsModuleBlock as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_module_decl(&mut self, node: &mut TsModuleDecl) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsModuleDecl as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_namespace_decl(&mut self, node: &mut TsNamespaceDecl) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsNamespaceDecl as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_namespace_export_decl(&mut self, node: &mut TsNamespaceExportDecl) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsNamespaceExportDecl as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_non_null_expr(&mut self, node: &mut TsNonNullExpr) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsNonNullExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_optional_type(&mut self, node: &mut TsOptionalType) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsOptionalType as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_param_prop(&mut self, node: &mut TsParamProp) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsParamProp as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_parenthesized_type(&mut self, node: &mut TsParenthesizedType) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsParenthesizedType as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_property_signature(&mut self, node: &mut TsPropertySignature) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsPropertySignature as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_qualified_name(&mut self, node: &mut TsQualifiedName) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsQualifiedName as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_rest_type(&mut self, node: &mut TsRestType) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsRestType as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_satisfies_expr(&mut self, node: &mut TsSatisfiesExpr) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsSatisfiesExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_setter_signature(&mut self, node: &mut TsSetterSignature) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsSetterSignature as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_this_type(&mut self, node: &mut TsThisType) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsThisType as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_tpl_lit_type(&mut self, node: &mut TsTplLitType) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsTplLitType as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_tuple_element(&mut self, node: &mut TsTupleElement) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsTupleElement as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_tuple_elements(&mut self, nodes: &mut Vec<TsTupleElement>) {
    for node in nodes.iter_mut() {
      if let Some(span) = self.get_span(node) {
        node.span = span;
      };
    }

    <Vec<TsTupleElement> as VisitMutWith<Self>>::visit_mut_children_with(nodes, self)
  }

  #[inline]
  fn visit_mut_ts_tuple_type(&mut self, node: &mut TsTupleType) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsTupleType as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_type_alias_decl(&mut self, node: &mut TsTypeAliasDecl) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsTypeAliasDecl as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_type_ann(&mut self, node: &mut TsTypeAnn) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsTypeAnn as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_type_assertion(&mut self, node: &mut TsTypeAssertion) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsTypeAssertion as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_type_lit(&mut self, node: &mut TsTypeLit) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsTypeLit as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_type_operator(&mut self, node: &mut TsTypeOperator) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsTypeOperator as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_type_param(&mut self, node: &mut TsTypeParam) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsTypeParam as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_type_param_decl(&mut self, node: &mut TsTypeParamDecl) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsTypeParamDecl as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_type_param_instantiation(&mut self, node: &mut TsTypeParamInstantiation) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsTypeParamInstantiation as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_type_params(&mut self, nodes: &mut Vec<TsTypeParam>) {
    for node in nodes.iter_mut() {
      if let Some(span) = self.get_span(node) {
        node.span = span;
      };
    }

    <Vec<TsTypeParam> as VisitMutWith<Self>>::visit_mut_children_with(nodes, self)
  }

  #[inline]
  fn visit_mut_ts_type_predicate(&mut self, node: &mut TsTypePredicate) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsTypePredicate as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_type_query(&mut self, node: &mut TsTypeQuery) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsTypeQuery as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_type_ref(&mut self, node: &mut TsTypeRef) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsTypeRef as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_ts_union_type(&mut self, node: &mut TsUnionType) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <TsUnionType as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_unary_expr(&mut self, node: &mut UnaryExpr) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <UnaryExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_update_expr(&mut self, node: &mut UpdateExpr) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <UpdateExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_using_decl(&mut self, node: &mut UsingDecl) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <UsingDecl as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_var_decl(&mut self, node: &mut VarDecl) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <VarDecl as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_var_declarator(&mut self, node: &mut VarDeclarator) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <VarDeclarator as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_while_stmt(&mut self, node: &mut WhileStmt) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <WhileStmt as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_with_stmt(&mut self, node: &mut WithStmt) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <WithStmt as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }

  #[inline]
  fn visit_mut_yield_expr(&mut self, node: &mut YieldExpr) {
    if let Some(span) = self.get_span(node) {
      node.span = span;
    };
    <YieldExpr as VisitMutWith<Self>>::visit_mut_children_with(node, self)
  }
}
