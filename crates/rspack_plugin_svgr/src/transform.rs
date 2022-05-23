use crate::mapping::NAME_MAPPING;
use heck::ToLowerCamelCase;
use rspack_core::ast::{
  Expr, Ident, JSXAttrOrSpread, JSXAttrValue, JSXElement, JSXText, Lit, Module, Program,
};
use rspack_swc::{
  swc_common, swc_ecma_ast,
  swc_ecma_visit::{self, FoldWith},
  swc_plugin,
};
use swc_common::DUMMY_SP;
use swc_ecma_ast::JSXAttrName;
use swc_ecma_visit::{as_folder, VisitMut, VisitMutWith};
use swc_plugin::{plugin_transform, TransformPluginProgramMetadata};

pub struct SvgrReplacer;
impl VisitMut for SvgrReplacer {
  fn visit_mut_module(&mut self, module: &mut Module) {
    module.visit_mut_children_with(self);
  }

  fn visit_mut_jsx_element(&mut self, elem: &mut JSXElement) {
    elem.visit_mut_children_with(self);
    for attr in elem.opening.attrs.iter_mut() {
      if let JSXAttrOrSpread::JSXAttr(attr) = attr {
        if let JSXAttrName::JSXNamespacedName(ref space_name) = attr.name {
          let name = space_name.name.sym.to_string();
          let ns = space_name.ns.sym.to_string();
          let full_name = ns + ":" + &name;
          if let Some(new_name) = NAME_MAPPING.get(&full_name as &str) {
            let new_id = Ident::new((*new_name).into(), DUMMY_SP);
            attr.name = JSXAttrName::Ident(new_id);
          }
        } else if let JSXAttrName::Ident(ref id) = attr.name {
          let name = id.sym.to_string();
          if let Some(new_name) = NAME_MAPPING.get(&name as &str) {
            let new_id = Ident::new((*new_name).into(), DUMMY_SP);
            attr.name = JSXAttrName::Ident(new_id);
          }

          if name == "style" {
            if let JSXAttrValue::Lit(Lit::Str(ref value)) = attr.value.as_ref().unwrap() {
              let v = value.value.to_string();
              attr.value = Some(JSXAttrValue::Lit(Lit::JSXText(JSXText {
                raw: "".into(),
                span: DUMMY_SP,
                value: format_css(v).into(),
              })));
            }
          }
        }
      }
    }
  }
  fn visit_mut_expr(&mut self, expr: &mut Expr) {
    expr.visit_mut_children_with(self);
    if let Expr::JSXElement(elem) = expr {
      self.visit_mut_jsx_element(elem);
    }
  }
}

#[plugin_transform]
pub fn process(program: Program, _metadata: TransformPluginProgramMetadata) -> Program {
  program.fold_with(&mut as_folder(SvgrReplacer))
}

pub fn format_css(css: String) -> String {
  let item = css
    .split(";")
    .map(|s| {
      let v = s.split(":").collect::<Vec<&str>>();
      if v.len() != 2 {
        return (*s).to_string();
      }
      let prefix = ToLowerCamelCase::to_lower_camel_case(v[0]);
      let postfix = v[1];
      return prefix.to_string() + ": `" + postfix + "`";
    })
    .collect::<Vec<String>>()
    .join(",");
  return "{".to_string() + &item + "}";
}
