use std::collections::{HashMap, HashSet};

use crossbeam::channel::Sender;
use linked_hash_map::LinkedHashMap;
use swc_atoms::JsWord;
use swc_ecma_ast::{CallExpr, Callee, ExportAll, Expr, ImportDecl, Lit, NamedExport};
use swc_ecma_visit::{noop_visit_type, Visit, VisitWith};

use crate::{
    graph_container::Msg,
    js_module::{DynImportDesc, JsModule, RelationInfo},
};

pub struct DependencyScanner<'a> {
    pub tx: &'a Sender<Msg>,
    pub js_module: &'a JsModule,
    pub dependencies: LinkedHashMap<JsWord, ()>,
    pub dynamic_dependencies: HashSet<DynImportDesc>,
    // pub re_exported: HashSet<JsWord>,
    // pub dyn_imported: HashSet<JsWord>,
    pub import_infos: HashMap<JsWord, RelationInfo>,
    // pub local_exports: HashMap<JsWord, ExportDesc>,
    // pub re_exports: HashMap<JsWord, ReExportDesc>,
    pub re_export_infos: HashMap<JsWord, RelationInfo>,
    // pub export_all_sources: HashSet<(JsWord, usize)>,
}

impl<'a> DependencyScanner<'a> {
    pub fn new(tx: &'a Sender<Msg>, js_module: &'a JsModule) -> Self {
        Self {
            tx,
            js_module,
            import_infos: Default::default(),
            re_export_infos: Default::default(),
            dynamic_dependencies: Default::default(),
            dependencies: Default::default(),
        }
    }
}

impl<'a> Visit for DependencyScanner<'a> {
    noop_visit_type!();
    fn visit_import_decl(&mut self, import_decl: &ImportDecl) {
        let imported_path = &import_decl.src.value;
        self.dependencies.insert(imported_path.clone(), ());
    }

    // fn visit_export_decl(&mut self, n: &ExportDecl) {

    // }

    fn visit_named_export(&mut self, node: &NamedExport) {
        if let Some(re_exported) = &node.src {
            self.dependencies.insert(re_exported.value.clone(), ());
        }
    }

    // fn visit_export_default_decl(&mut self, n: &ExportDefaultDecl) {  }

    // fn visit_export_default_expr(&mut self, n: &ExportDefaultExpr) {  }

    fn visit_export_all(&mut self, node: &ExportAll) {
        self.dependencies.insert(node.src.value.clone(), ());
    }

    fn visit_call_expr(&mut self, node: &CallExpr) {
        // println!("import node: {:#?}", node);
        if let Callee::Import(_) = node.callee {
            if let Some(dyn_imported) = node.args.get(0) {
                if dyn_imported.spread.is_none() {
                    if let Expr::Lit(Lit::Str(imported)) = dyn_imported.expr.as_ref() {
                        self.dynamic_dependencies.insert(DynImportDesc {
                            argument: imported.value.clone(),
                        });
                    }
                }
            }
        } else {
            node.visit_children_with(self);
        }
    }
}

// impl DependencyScanner {
//   pub fn add_import(&mut self, module_decl: &mut ModuleDecl) {
//     if let ModuleDecl::Import(import_decl) = module_decl {
//       let source = &import_decl.src.value;
//       let import_info = self.import_infos.entry(source.clone()).or_insert_with(|| {
//         let rel = RelationInfo::new(source.clone(), self.cur_relation_order);
//         self.cur_relation_order += 1;
//         rel
//       });

//       // We separate each specifier to support later tree-shaking.
//       import_decl.specifiers.iter_mut().for_each(|specifier| {
//         let used;
//         let original;
//         let mark;
//         match specifier {
//           // import foo from './foo'
//           swc_ecma_ast::ImportSpecifier::Default(n) => {
//             used = n.local.sym.clone();
//             original = "default".into();
//             mark = n.local.span.ctxt.as_mark();
//           }
//           // import { foo } from './foo'
//           // import { foo as foo2 } from './foo'
//           swc_ecma_ast::ImportSpecifier::Named(n) => {
//             used = n.local.sym.clone();
//             original = n
//               .imported // => foo2 in `import { foo as foo2 } from './foo'`
//               .as_ref()
//               .map_or(used.clone(), |module_export_name| {
//                 if let ModuleExportName::Ident(ident) = module_export_name {
//                   ident.sym.clone()
//                 } else {
//                   panic!("")
//                 }
//               });
//             mark = n.local.span.ctxt.as_mark();
//           }
//           // import * as foo from './foo'
//           swc_ecma_ast::ImportSpecifier::Namespace(n) => {
//             used = n.local.sym.clone();
//             original = "*".into();
//             mark = n.local.span.ctxt.as_mark();
//           }
//         }
//         import_info.names.insert(Specifier {
//           original,
//           used,
//           mark,
//         });
//       });
//     }
//   }

//   pub fn add_dynamic_import(&mut self, call_exp: &CallExpr) {
//     if let Callee::Import(_import) = &call_exp.callee {
//       // FIXME: should warn about pattern like `import(...a)`
//       if let Some(exp) = call_exp
//         .args
//         .get(0)
//         .map(|exp_or_spread| &exp_or_spread.expr)
//       {
//         if let Expr::Lit(Lit::Str(first_param)) = exp.as_ref() {
//           self.dynamic_imports.insert(DynImportDesc {
//             argument: first_param.value.clone(),
//             id: None,
//           });
//         } else {
//           panic!("unkown dynamic import params")
//         }
//       }
//     }
//   }

//   pub fn add_export(&mut self, module_decl: &ModuleDecl) -> Result<(), anyhow::Error> {
//     match module_decl {
//       ModuleDecl::ExportDefaultDecl(node) => {
//         let identifier = match &node.decl {
//           DefaultDecl::Class(node) => node.ident.as_ref().map(|id| id.sym.clone()),
//           DefaultDecl::Fn(node) => node.ident.as_ref().map(|id| id.sym.clone()),
//           _ => None,
//         };
//         let mark = self
//           .symbol_box
//           .lock()
//           .map_err(|_| RolldownError::Lock)?
//           .new_mark();
//         self.statement_infos[self.cur_stmt_index].export_mark = Some(mark);
//         self.local_exports.insert(
//           "default".into(),
//           ExportDesc {
//             identifier,
//             local_name: "default".into(),
//             mark,
//           },
//         );
//       }
//       ModuleDecl::ExportDefaultExpr(node) => {
//         // export default foo;
//         let identifier: Option<JsWord> = match node.expr.as_ref() {
//           Expr::Ident(id) => Some(id.sym.clone()),
//           _ => None,
//         };
//         let mark = self
//           .symbol_box
//           .lock()
//           .map_err(|_| RolldownError::Lock)?
//           .new_mark();
//         self.statement_infos[self.cur_stmt_index].export_mark = Some(mark);
//         self.local_exports.insert(
//           "default".into(),
//           ExportDesc {
//             identifier,
//             local_name: "default".into(),
//             mark,
//           },
//         );
//       }
//       ModuleDecl::ExportNamed(node) => {
//         node.specifiers.iter().try_for_each(|specifier| {
//           match specifier {
//             ExportSpecifier::Named(s) => {
//               if let Some(source_node) = &node.src {
//                 let source = source_node.value.clone();
//                 let re_export_info =
//                   self
//                     .re_export_infos
//                     .entry(source.clone())
//                     .or_insert_with(|| {
//                       let rel = RelationInfo {
//                         source: source.clone(),
//                         names: Default::default(),
//                         order: self.cur_relation_order,
//                       };
//                       self.cur_relation_order += 1;
//                       rel
//                     });
//                 // export { name } from './other'
//                 let source = source_node.value.clone();
//                 let name = s
//                   .exported
//                   .as_ref()
//                   .map_or(get_sym_from_module_export(&s.orig), |id| {
//                     get_sym_from_module_export(id)
//                   });
//                 let re_export_mark = self
//                   .symbol_box
//                   .lock()
//                   .map_err(|_| RolldownError::Lock)?
//                   .new_mark();
//                 re_export_info.names.insert(Specifier {
//                   original: get_sym_from_module_export(&s.orig),
//                   used: name.clone(),
//                   mark: re_export_mark,
//                 });
//                 self.statement_infos[self.cur_stmt_index].export_mark = Some(re_export_mark);
//                 self.re_exports.insert(
//                   name.clone(),
//                   ReExportDesc {
//                     local_name: get_sym_from_module_export(&s.orig),
//                     source,
//                     original: name,
//                     mark: re_export_mark,
//                   },
//                 );
//               } else {
//                 // export { foo, bar, baz }
//                 log::debug!("export var {:#?}", s);
//                 let local_name = get_sym_from_module_export(&s.orig);
//                 let exported_name: JsWord = s
//                   .exported
//                   .as_ref()
//                   .map_or(get_sym_from_module_export(&s.orig), |id| {
//                     get_sym_from_module_export(id)
//                   });

//                 let mark = self.symbol_box.lock().unwrap().new_mark();
//                 self.statement_infos[self.cur_stmt_index].export_mark = Some(mark);
//                 self.local_exports.insert(
//                   exported_name,
//                   ExportDesc {
//                     identifier: None,
//                     local_name,
//                     mark,
//                   },
//                 );
//               };
//             }
//             ExportSpecifier::Namespace(s) => {
//               let source = node.src.as_ref().map(|str| str.value.clone()).unwrap();
//               let re_export_info =
//                 self
//                   .re_export_infos
//                   .entry(source.clone())
//                   .or_insert_with(|| {
//                     let rel = RelationInfo {
//                       source: source.clone(),
//                       names: Default::default(),
//                       order: self.cur_relation_order,
//                     };
//                     self.cur_relation_order += 1;
//                     rel
//                   });
//               let re_export_mark = self
//                 .symbol_box
//                 .lock()
//                 .map_err(|_| RolldownError::Lock)?
//                 .new_mark();

//               re_export_info.names.insert(Specifier {
//                 original: "*".into(),
//                 used: get_sym_from_module_export(&s.name),
//                 mark: re_export_mark,
//               });
//               // export * as name from './other'
//               let name = get_sym_from_module_export(&s.name);
//               self.statement_infos[self.cur_stmt_index].export_mark = Some(re_export_mark);
//               self.re_exports.insert(
//                 name.clone(),
//                 ReExportDesc {
//                   local_name: "*".into(),
//                   source,
//                   original: name,
//                   mark: re_export_mark,
//                 },
//               );
//             }
//             ExportSpecifier::Default(_) => {
//               // export v from 'mod';
//               // Rollup doesn't support it.
//             }
//           };
//           Ok::<(), RolldownError>(())
//         })?;
//       }
//       ModuleDecl::ExportDecl(node) => {
//         match &node.decl {
//           Decl::Class(node) => {
//             // export class Foo {}
//             let local_name = node.ident.sym.clone();
//             let mark = self
//               .symbol_box
//               .lock()
//               .map_err(|_| RolldownError::Lock)?
//               .new_mark();
//             self.statement_infos[self.cur_stmt_index].export_mark = Some(mark);
//             self.local_exports.insert(
//               local_name.clone(),
//               ExportDesc {
//                 identifier: None,
//                 local_name,
//                 mark,
//               },
//             );
//           }
//           Decl::Fn(node) => {
//             // export function foo () {}
//             let local_name = node.ident.sym.clone();
//             let mark = self
//               .symbol_box
//               .lock()
//               .map_err(|_| RolldownError::Lock)?
//               .new_mark();
//             self.statement_infos[self.cur_stmt_index].export_mark = Some(mark);
//             self.local_exports.insert(
//               local_name.clone(),
//               ExportDesc {
//                 identifier: None,
//                 local_name,
//                 mark,
//               },
//             );
//           }
//           Decl::Var(node) => {
//             // export var { foo, bar } = ...
//             // export var foo = 1, bar = 2;
//             node.decls.iter().try_for_each(|decl| {
//               collect_js_word_of_pat(&decl.name)
//                 .into_iter()
//                 .try_for_each(|local_name| {
//                   let mark = self
//                     .symbol_box
//                     .lock()
//                     .map_err(|_| RolldownError::Lock)?
//                     .new_mark();
//                   self.statement_infos[self.cur_stmt_index].export_mark = Some(mark);
//                   self.local_exports.insert(
//                     local_name.clone(),
//                     ExportDesc {
//                       identifier: None,
//                       local_name,
//                       mark,
//                     },
//                   );
//                   Ok::<(), RolldownError>(())
//                 })
//             })?;
//           }
//           _ => {}
//         }
//       }
//       ModuleDecl::ExportAll(node) => {
//         // export * from './other'
//         self
//           .export_all_sources
//           .insert((node.src.value.clone(), self.cur_relation_order));
//         self.cur_relation_order += 1;
//       }
//       _ => {}
//     }
//     Ok(())
//   }
// }
