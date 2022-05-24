use crate::prefix::{DefinePrefix, PatMetaInfo};
use rspack_core::ast::{self, Ident};
use rspack_swc::swc_ecma_visit::{Fold, FoldWith};
use std::collections::{HashMap, HashSet};

type DefineKey = String;
type DefineValue = String;

#[derive(Default, Debug, Clone)]
#[cfg_attr(test, derive(serde::Serialize, serde::Deserialize, PartialEq))]
struct DefineTreeNode {
  pub value: Option<DefineValue>,
  pub children: HashMap<DefineKey, DefineTreeNode>,
}

impl DefineTreeNode {
  // fn parse_value(cm: &Lrc<SourceMap>, value: &str) -> ast::Expr {
  //   let fm = cm.new_source_file(FileName::Anon, value.to_string());
  //   let lexer = Lexer::new(
  //     Default::default(),
  //     Default::default(),
  //     StringInput::from(&*fm),
  //     None,
  //   );

  //   match Parser::new_from(lexer).parse_expr() {
  //     Ok(expr) => *expr,
  //     Err(_) => panic!("{} had something wrong.", value),
  //   }
  // }

  // fn to_value(cm: &Lrc<SourceMap>, expr: &ast::Expr) -> String {
  //   let mut buf = vec![];
  //   let mut emitter = Emitter {
  //     cfg: Default::default(),
  //     cm: cm.clone(),
  //     comments: None,
  //     wr: Box::new(JsWriter::new(cm.clone(), "", &mut buf, None)),
  //   };

  //   if expr.emit_with(&mut emitter).is_err() {
  //     panic!("{:?} had something wrong.", expr);
  //   }
  //   String::from_utf8(buf).unwrap()
  // }

  fn insert_to_may_exist_node(&mut self, key: String, value: String) {
    let map = &mut self.children;
    match map.get_mut(&key) {
      Some(node) => {
        node.value.is_none().then(|| {
          node.value = Some(value);
        });
      }
      None => {
        map.insert(key, Self::create_new_node(value));
      }
    }
  }

  fn walk(mut node: &mut DefineTreeNode, mut key: &str, value: String) {
    if key.contains('.') {
      let splitted: Vec<&str> = key.split('.').collect();
      let len = splitted.len();
      #[allow(clippy::needless_range_loop)]
      for index in 0..len {
        let part = splitted[index];
        if index != len - 1 {
          let map = &mut node.children;
          if map.get(part).is_none() {
            let next = DefineTreeNode::create_empty_node();
            map.insert(part.to_string(), next);
          };
          node = map.get_mut(part).unwrap();
        } else {
          key = part
        }
      }
    };

    let key = key.to_string();
    // Do not constructor tire for `array` or `object`.
    // The work for get the closest value should put in `minify` phase.
    //
    // match expr {
    //   ast::Expr::Array(array) => {
    //     let elements = array.elems;
    //     let mut next = DefineTreeNode::create_new_node(value);
    //     elements
    //       .into_iter()
    //       .enumerate()
    //       .for_each(|(index, element)| match element {
    //         Some(ele) => Self::walk(
    //           &mut next,
    //           &index.to_string(),
    //           Self::to_value(cm, &ele.expr),
    //           cm,
    //           *ele.expr,
    //         ),
    //         None => Self::walk(
    //           &mut next,
    //           &index.to_string(),
    //           "null".to_string(),
    //           cm,
    //           ast::Expr::Lit(ast::Lit::Null(ast::Null {
    //             span: Default::default(),
    //           })),
    //         ),
    //       });
    //     node.children.entry(key).or_insert(next);
    //   }
    //   ast::Expr::Object(object) => {
    //     let props = object.props;
    //     let mut next = DefineTreeNode::create_new_node(value);
    //     props.into_iter().for_each(|prop| {
    //       if let ast::PropOrSpread::Prop(prop) = prop {
    //         match *prop {
    //           ast::Prop::Shorthand(ident) => {
    //             let sym = ident.sym.to_string();
    //             node.insert_to_may_exist_node(sym.clone(), sym);
    //           }
    //           ast::Prop::KeyValue(prop) => {
    //             let key = if let ast::PropName::Ident(ident) = prop.key {
    //               ident.sym.to_string()
    //             } else if let ast::PropName::Str(str) = prop.key {
    //               str.value.to_string()
    //             } else {
    //               panic!("{:?} is not a key type", prop.key);
    //             };
    //             Self::walk(
    //               &mut next,
    //               &key,
    //               Self::to_value(cm, &prop.value),
    //               cm,
    //               *prop.value,
    //             );
    //           }
    //           _ => {}
    //         }
    //       }
    //     });
    //     node.children.entry(key).or_insert(next);
    //   }
    //   // TODO: `function`, `unary` minify.
    //   _ => node.insert_to_may_exist_node(key, value),
    // }
    node.insert_to_may_exist_node(key, value);
  }

  fn create_new_node(value: DefineValue) -> DefineTreeNode {
    DefineTreeNode {
      value: Some(value),
      children: HashMap::new(),
    }
  }

  fn create_empty_node() -> DefineTreeNode {
    DefineTreeNode {
      value: None,
      children: HashMap::new(),
    }
  }

  pub fn build(define: &HashMap<DefineKey, DefineValue>) -> DefineTreeNode {
    if define.is_empty() {
      DefineTreeNode::default()
    } else {
      let mut root = DefineTreeNode::create_empty_node();
      for (key, value) in define {
        Self::walk(&mut root, key, value.to_string())
      }
      root
    }
  }
}

#[test]
fn test_define_tree() {
  use serde_json::json;

  fn compare(define: Vec<(&str, &str)>, json: serde_json::Value) {
    let define = HashMap::from_iter(
      define
        .into_iter()
        .map(|(key, value)| (key.to_string(), value.to_string())),
    );
    let tree = DefineTreeNode::build(&define);
    assert_eq!(tree, serde_json::from_value(json).unwrap());
  }

  compare(
    vec![
      ("a", "aaa"),
      ("a.b.c", "bbb"),
      ("b", "bbb"),
      ("b.a.c", "aaa"),
      ("c", "ccc"),
    ],
    json!({
      "value": null,
      "children": {
        "a": {
          "value": "aaa",
          "children": {
            "b": {
              "value": null,
              "children": {
                "c": {
                  "value": "bbb",
                  "children": {},
                }
              }
            }
          },
        },
        "b": {
          "value": "bbb",
          "children": {
            "a": {
              "value": null,
              "children": {
                "c": {
                "value": "aaa",
                "children": {},
                }
              }
            }
          },
        },
        "c": {
          "value": "ccc",
          "children": {},
        }
      }
    }),
  );

  compare(
    vec![
      ("num", "1"),
      ("num_add", "1+1"),
      ("bool", "false"),
      ("null", "null"),
      ("nan", "NAN"),
      ("undefined", "undefined"),
      ("string", "\"str\""),
    ],
    json!({
      "value": null,
      "children": {
        "nan": {
          "value": "NAN",
          "children": {}
        },
        "num": {
          "value": "1",
          "children": {}
        },
        "num_add": {
          "value": "1+1",
          "children": {}
        },
        "bool": {
          "value": "false",
          "children": {}
        },
        "null": {
          "value": "null",
          "children": {}
        },
        "undefined": {
          "value": "undefined",
          "children": {}
        },
        "string": {
          "value": "\"str\"",
          "children": {},
        },
      }
    }),
  );

  compare(
    vec![("array", "[1,[2]]"), ("empty_array", "[]")],
    json!({
      "value": null,
      "children": {
        "empty_array": {
          "value": "[]",
          "children": {},
        },
        "array": {
          "value": "[1,[2]]",
          "children": {}
        },
      }
    }),
  );

  compare(
    vec![
      ("object", "{\"a\":1,\"b\":{\"c\":2,\"d\":3}}"),
      ("empty_object", "{}"),
    ],
    json!({
      "value": null,
      "children": {
        "empty_object": {
          "value": "{}",
          "children": {},
        },
        "object": {
          "value": "{\"a\":1,\"b\":{\"c\":2,\"d\":3}}",
          "children": {},
        },
      }
    }),
  );
}

#[derive(Debug, Clone)]
pub struct DefineTransform {
  defintions: DefineTreeNode,
  can_not_rename: HashSet<PatMetaInfo>,
}

enum MemberStats<'a> {
  BothIdent(Vec<(String, &'a DefineTreeNode)>),
  ObjBothIdent(Vec<(String, &'a DefineTreeNode)>),
  Impurity,
}

impl DefineTransform {
  pub(crate) fn new(defintions: &HashMap<String, String>, prefix: DefinePrefix) -> Self {
    let defintions_tree = DefineTreeNode::build(defintions);
    DefineTransform {
      defintions: defintions_tree,
      can_not_rename: prefix.can_not_rename,
    }
  }

  /// Check `Ident` could be renamed.
  /// If could, then return the renamed String, else return `None`.
  fn ident_can_rename(&self, ident: &Ident) -> Option<String> {
    let name = ident.sym.to_string();
    let ctxt = ident.span.ctxt;
    let info = PatMetaInfo { name, ctxt };
    if self.can_not_rename.contains(&info) {
      None
    } else {
      Some(info.name)
    }
  }

  fn get_member_stats<'a>(
    &'a self,
    member: &'a ast::MemberExpr,
    is_first: bool,
    node: &'a DefineTreeNode,
  ) -> MemberStats<'a> {
    let mut list = vec![];
    if let ast::Expr::Ident(ident) = member.obj.as_ref() {
      let sym = if !is_first {
        ident.sym.to_string()
      } else if let Some(sym) = self.ident_can_rename(ident) {
        sym
      } else {
        return MemberStats::Impurity;
      };
      if let Some(next) = node.children.get(&sym) {
        list.push((sym, next));
      } else {
        return MemberStats::Impurity;
      }
    } else if let ast::Expr::Member(next_member) = member.obj.as_ref() {
      match self.get_member_stats(next_member, false, node) {
        MemberStats::BothIdent(sub) => list.extend(sub),
        _ => return MemberStats::Impurity,
      }
    }

    if let ast::MemberProp::Ident(ident) = &member.prop {
      let sym = ident.sym.to_string();
      let expected_some = list
        .last()
        .map(|last| last.1)
        .and_then(|node| node.children.get(&sym))
        .map(|n| {
          list.push((sym, n));
        });
      if expected_some.is_none() {
        return MemberStats::ObjBothIdent(list);
      }
    } else {
      return MemberStats::ObjBothIdent(list);
    }
    MemberStats::BothIdent(list)
  }

  /// Example: the ast of `a.b.c` is:
  ///
  /// ```text
  ///              __
  ///      (obj) /   \  (prop)
  ///           a     c
  ///            \
  ///             b
  /// ```
  ///
  /// Here, the nodes are matched by additional time consumption,
  /// because it is difficult to find the corresponding node and modify it in linear time.
  /// (maybe it can be optimized by more appropriate data structure)
  fn deal_with_member(&mut self, member: ast::MemberExpr) -> ast::Expr {
    match self.get_member_stats(&member, true, &self.defintions) {
      MemberStats::BothIdent(list) => list
        .last()
        .and_then(|(_, node)| node.value.clone())
        .map(|renamed| {
          ast::Expr::Ident(ast::Ident {
            sym: Self::to_code(renamed).into(),
            optional: false,
            span: member.span,
          })
        })
        .unwrap_or_else(|| ast::Expr::Member(member)),
      MemberStats::ObjBothIdent(list) => {
        // We can continue to optimize in the following way:
        // 1. `a[1]` to `defined_value`, (But I think this process can put into `minify` phase,
        // so I remove trie for object or array)
        // 2. `a[1 + 1]` to `a[2]`, (I noticed that `webpack` had only dealled whithin `Ident`)
        let renamed = list.last().and_then(|(_, node)| node.value.clone());
        let obj = if let Some(renamed) = renamed {
          Box::new(ast::Expr::Ident(ast::Ident {
            sym: Self::to_code(renamed).into(),
            optional: false,
            span: member.span,
          }))
        } else {
          member.obj
        };
        ast::Expr::Member(ast::MemberExpr {
          obj,
          prop: member.prop.fold_children_with(self),
          span: member.span,
        })
      }
      MemberStats::Impurity => {
        let obj = if let ast::Expr::Member(next_member) = *member.obj {
          Box::new(self.deal_with_member(next_member))
        } else {
          member.obj
        };
        ast::Expr::Member(ast::MemberExpr {
          obj,
          prop: member.prop.fold_children_with(self),
          span: member.span,
        })
      }
    }
  }

  fn to_code(sym: String) -> String {
    if sym.starts_with('{') && sym.ends_with('}') {
      format!("({})", sym)
    } else {
      sym
    }
  }

  fn deal_with_ident(&self, ident: ast::Ident) -> ast::Ident {
    let sym = self
      .ident_can_rename(&ident)
      .and_then(|sym| self.defintions.children.get(&sym))
      .and_then(|node| node.value.clone())
      .unwrap_or_else(|| ident.sym.to_string());

    ast::Ident {
      sym: Self::to_code(sym).into(),
      ..ident
    }
  }
}

impl Fold for DefineTransform {
  fn fold_expr(&mut self, expr: ast::Expr) -> ast::Expr {
    match expr {
      ast::Expr::Ident(ident) => ast::Expr::Ident(self.deal_with_ident(ident)),
      ast::Expr::Member(member) => self.deal_with_member(member),
      ast::Expr::Assign(assign) => {
        let left = match assign.left {
          ast::PatOrExpr::Expr(expr) => ast::PatOrExpr::Expr(expr.fold_with(self)),
          ast::PatOrExpr::Pat(pat) => ast::PatOrExpr::Pat(match *pat {
            ast::Pat::Ident(binding) => Box::new(ast::Pat::Ident(ast::BindingIdent {
              id: self.deal_with_ident(binding.id),
              ..binding
            })),
            _ => pat.fold_with(self),
          }),
        };
        let right = assign.right.fold_with(self);
        ast::Expr::Assign(ast::AssignExpr {
          left,
          right,
          ..assign
        })
      }
      // TODO: ident in other statements.
      _ => expr.fold_children_with(self),
    }
  }
}
