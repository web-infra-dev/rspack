use crate::mapping::NAME_MAPPING;
use rspack_core::ast::{
  Expr, Ident, JSXAttrOrSpread, JSXAttrValue, JSXElement, JSXText, Lit, Program,
};
use rspack_swc::swc_ecma_visit::FoldWith;
use rspack_swc::{swc, swc_common, swc_ecma_ast, swc_ecma_visit, swc_plugin};

use heck::ToLowerCamelCase;
use rspack_core::{get_swc_compiler, syntax_by_ext};
use swc::config::SourceMapsConfig;
use swc::ecmascript::ast::EsVersion;

use swc::config::IsModule;
use swc_common::{FileName, DUMMY_SP};
use swc_ecma_ast::JSXAttrName;
use swc_ecma_visit::{as_folder, VisitMut, VisitMutWith};
use swc_plugin::{plugin_transform, TransformPluginProgramMetadata};
pub fn transform(source_code: String) -> String {
  let compiler = get_swc_compiler();
  let syntax = syntax_by_ext("jsx");
  let fm = compiler
    .cm
    .new_source_file(FileName::Custom("svg.jsx".to_string()), source_code);
  let program = swc::try_with_handler(compiler.cm.clone(), Default::default(), |handler| {
    compiler.parse_js(
      fm,
      handler,
      EsVersion::Es2022,
      syntax,
      IsModule::Bool(true),
      None,
    )
  })
  .unwrap();
  let metadata = swc_plugin::TransformPluginProgramMetadata {
    comments: None,
    source_map: swc_plugin::source_map::PluginSourceMapProxy,
    plugin_config: "".to_string(),
    transform_context: "".to_string(),
  };
  let program = process(program, metadata);
  compiler
    .print(
      &program,
      None,
      None,
      false,
      EsVersion::Es2020,
      SourceMapsConfig::Bool(false),
      &Default::default(),
      None,
      false,
      None,
      false,
      false,
    )
    .unwrap()
    .code
}

struct SvgrReplacer;
impl VisitMut for SvgrReplacer {
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
  return "{{".to_string() + &item + "}}";
}
