use std::{collections::BTreeMap, sync::Mutex};

use html_escape::encode_safe;
use rustc_hash::FxHashMap as HashMap;
#[cfg(test)]
use serde_json::{Value, json};

use crate::{context::*, eval::*, expr::*};

type JumpTable = HashMap<usize, usize>;
type JumpTables = (JumpTable, JumpTable);

// The executer that renders the template.
#[derive(Debug)]
pub struct Executer {
  insts: Vec<Action<Eval>>,

  // Tells where the instructor point to jump
  // - For, If, While: Tells where to jump if the condition is false.
  // - End, Break, Continue: Tells where to jump if hit.
  jump_table: HashMap<usize, usize>,

  // Mapping between index of "Break" and the corresponding For.
  break_table: HashMap<usize, usize>,
}

impl Executer {
  pub fn new(mut parser: Parser) -> Result<Self, String> {
    let mut insts = Vec::new();

    while let Some(expr) = parser.parse_tree.pop() {
      insts.push(convert_expr_to_eval(expr)?)
    }

    insts.reverse();

    let (jump_table, break_table) = Executer::compute_jump_table(&insts)?;
    Ok(Executer {
      jump_table,
      break_table,
      insts,
    })
  }

  fn compute_jump_table(insts: &[Action<Eval>]) -> Result<JumpTables, String> {
    let mut inst_index = 0;

    let mut jump_table = HashMap::default();
    let mut break_table = HashMap::default();

    let mut if_matching_ends = BTreeMap::new();

    // Every '{'.
    let mut opened = Vec::new();

    // Every '{' for loops (for, while). This will be used by break and continue.
    let mut loop_opened = Vec::new();

    // Mapping between location of break/continue to the corresponding loop.
    let mut break_and_continue_pos = HashMap::default();

    while inst_index < insts.len() {
      match &insts[inst_index] {
        Action::If(_) => opened.push(inst_index),
        Action::While(_) => {
          opened.push(inst_index);
          loop_opened.push(inst_index);
        }
        Action::For(_) => {
          opened.push(inst_index);
          loop_opened.push(inst_index);
        }
        Action::Else() => {
          if let Some(Action::If(_)) = insts.get(inst_index + 1) {
            inst_index += 1;
            continue;
          } else {
            opened.push(inst_index)
          }
        }
        Action::Do(eval) => {
          if let Some(keyword) = eval.get_keyword()
            && (keyword == Keyword::Break || keyword == Keyword::Continue)
          {
            if loop_opened.is_empty() {
              return Err("Cannot break/continue within non-loop context.".to_string());
            }

            break_and_continue_pos.insert(inst_index, *loop_opened.last().unwrap());
          }
        }
        Action::End() => {
          if opened.is_empty() {
            return Err(format!(
              "Unmatched end found at {inst_index:?} for {insts:?}"
            ));
          }

          let open_index = opened.pop().unwrap();
          match &insts[open_index] {
            Action::If(_) => {
              if_matching_ends.insert(open_index, inst_index + 1);
            }
            Action::Else() => {
              if_matching_ends.insert(open_index, inst_index + 1);
            }
            Action::For(_) => {
              jump_table.insert(open_index, inst_index + 1);
              jump_table.insert(inst_index, open_index);
              loop_opened.pop();
            }
            Action::While(_) => {
              jump_table.insert(open_index, inst_index + 1);
              jump_table.insert(inst_index, open_index);
              loop_opened.pop();
            }
            _ => {
              return Err(format!("Unknown action {insts:?} with closing parentheses"));
            }
          }
        }
        _ => {}
      }

      inst_index += 1;
    }

    if !opened.is_empty() {
      return Err(format!(
        "No closing bracket found {}",
        pretty_print_insts(insts)
      ));
    }

    // Handle break and continue.
    for (keyword_index, loop_index) in break_and_continue_pos {
      if let Action::Do(eval) = &insts[keyword_index] {
        let keyword = eval.get_keyword().unwrap();
        if keyword == Keyword::Continue {
          // For continue, then jump back to the start of the loop.
          jump_table.insert(keyword_index, loop_index);
        } else if keyword == Keyword::Break {
          // For break, then jump back to the end of the loop.
          jump_table.insert(keyword_index, jump_table[&loop_index]);
          break_table.insert(keyword_index, loop_index);
        }
      }
    }

    // Now scan again to handle if-else.
    while !if_matching_ends.is_empty() {
      let (if_start, if_block_end) = if_matching_ends.iter().next().unwrap();
      let if_start = *if_start;
      let mut if_block_end = *if_block_end;

      let mut if_else_block_starts = vec![if_start];

      // Find the index of all "if"s in same if-else block.
      while let Some(&Action::Else()) = insts.get(if_block_end) {
        // If this is the if-else statement
        if let Some(&Action::If(_)) = insts.get(if_block_end + 1) {
          if_else_block_starts.push(if_block_end + 1);
          if_block_end = if_matching_ends[&(if_block_end + 1)];
        } else {
          if_else_block_starts.push(if_block_end);

          // Otherwise this is just else statement.
          if_block_end = if_matching_ends[&if_block_end];
          break;
        }
      }

      // "End" of if statement should point to next of the End of the if-else block.
      // "If" of the if statement should point to next of the End.
      for if_start in if_else_block_starts {
        let if_end = if_matching_ends[&if_start];

        // This is the case when the condition is true.
        jump_table.insert(if_end - 1, if_block_end);

        // This is the case when the condition is false.
        jump_table.insert(if_start, if_end);

        if_matching_ends.remove(&if_start);
      }
    }

    Ok((jump_table, break_table))
  }

  pub fn render(
    &self,
    context: &mut Context,
    templates: &HashMap<String, (Executer, String)>,
    functions: &HashMap<String, FunctionContainer>,
    template: &str,
    includes: &mut Mutex<HashMap<String, String>>,
  ) -> Result<String, String> {
    let mut rendered = String::new();
    let mut for_index_counter = HashMap::default();

    // Contains the range value of the for-loop.
    let mut for_range_container = HashMap::default();

    let mut inst_index = 0;
    while inst_index < self.insts.len() {
      match &self.insts[inst_index] {
        Action::Show(show) => match show {
          Show::Html { start, end } => {
            rendered.push_str(&template[*start..*end]);
          }
          Show::ExprEscaped(eval) => {
            rendered.push_str(&encode_safe(&Executer::run_eval(
              context, templates, functions, eval, includes,
            )?));
          }
          Show::ExprUnescaped(eval) => {
            rendered.push_str(&Executer::run_eval(
              context, templates, functions, eval, includes,
            )?);
          }
        },
        Action::If(eval) | Action::While(eval) => {
          // Jump only if the condition is false. Otherwise just go to next instruction.
          if !eval.run(context, templates, functions, includes)?.is_true() {
            if let Some(next) = self.jump_table.get(&inst_index) {
              inst_index = *next;
              continue;
            } else {
              return Err(format!(
                "Jump of the if statement is not set: {:?} index : {}",
                self.insts, inst_index
              ));
            }
          }
        }
        Action::For(eval) => {
          let iter_index = match for_index_counter.get(&inst_index) {
            Some(index) => *index,
            None => 0usize,
          };

          // The for loop is first encountered. Compute the "range" part.
          if iter_index == 0 {
            for_range_container.insert(
              inst_index,
              eval.run_for_range(context, templates, functions, includes)?,
            );
          }

          for_index_counter.insert(inst_index, iter_index + 1);

          let range = &for_range_container[&inst_index];
          if !eval.run_for_loop(context, range, iter_index)? {
            if let Some(next) = self.jump_table.get(&inst_index) {
              // Reset the index counter.
              for_index_counter.insert(inst_index, 0);

              inst_index = *next;

              continue;
            } else {
              return Err(format!(
                "Jump of the if statement is not set: {:?} index : {}",
                self.insts, inst_index
              ));
            }
          }
        }
        Action::End() => {
          if let Some(next) = self.jump_table.get(&inst_index) {
            inst_index = *next;
            continue;
          }
        }
        Action::Do(eval) => match eval.get_keyword() {
          Some(Keyword::Break) => {
            // Reset the iter counter of the loop that we escape.
            for_index_counter.insert(self.break_table[&inst_index], 0);

            inst_index = self.jump_table[&inst_index];
            continue;
          }
          Some(Keyword::Continue) => {
            inst_index = self.jump_table[&inst_index];
            continue;
          }
          _ => {
            Executer::run_eval(context, templates, functions, eval, includes)?;
          }
        },
        Action::Else() => {}
      }

      inst_index += 1;
    }

    Ok(rendered)
  }

  // Runs Eval if it is not empty.
  fn run_eval(
    context: &mut Context,
    templates: &HashMap<String, (Executer, String)>,
    functions: &HashMap<String, FunctionContainer>,
    eval: &Eval,
    includes: &mut Mutex<HashMap<String, String>>,
  ) -> Result<String, String> {
    if eval.is_empty() {
      return Ok(String::new());
    }

    Ok(eval.run(context, templates, functions, includes)?.to_str())
  }
}

fn pretty_print_insts(insts: &[Action<Eval>]) -> String {
  let mut result = String::new();

  let mut padding = String::new();
  for action in insts {
    match action {
      Action::If(_) | Action::While(_) | Action::For(_) => {
        result.push_str(&format!("{padding}{action:?} \n"));
        padding.push(' ');
      }
      Action::End() => {
        padding.pop();
        result.push_str(&format!("{padding}{action:?} \n"));
      }
      action => {
        result.push_str(&format!("{padding}{action:?} \n"));
      }
    }
  }

  result
}

fn convert_expr_to_eval(action: Action<Tokens>) -> Result<Action<Eval>, String> {
  match action {
    Action::Show(show) => match show {
      Show::Html { start, end } => Ok(Action::Show(Show::Html { start, end })),
      Show::ExprEscaped(expr) => Ok(Action::Show(Show::ExprEscaped(Eval::new(expr)?))),
      Show::ExprUnescaped(expr) => Ok(Action::Show(Show::ExprUnescaped(Eval::new(expr)?))),
    },
    Action::If(expr) => Ok(Action::If(Eval::new(expr)?)),
    Action::While(expr) => Ok(Action::While(Eval::new(expr)?)),
    Action::Do(expr) => Ok(Action::Do(Eval::new(expr)?)),
    Action::For(expr) => Ok(Action::For(Eval::new(expr)?)),
    Action::Else() => Ok(Action::Else()),
    Action::End() => Ok(Action::End()),
  }
}

#[test]
fn test_if_jump_table() {
  let executer = Executer::new(
    Parser::parse(r#"<% if var1>=var2{ "b" } else if var1==3{"a"}else{"c"}%>"#).unwrap(),
  )
  .unwrap();

  assert_eq!(
    executer.jump_table,
    [(1, 4), (3, 11), (5, 8), (7, 11), (8, 11), (10, 11)]
      .iter()
      .copied()
      .collect()
  )
}

#[test]
fn test_multiple_ifs() {
  let executer =
    Executer::new(Parser::parse(r#"<% if a >= b {} else if c >= a {} if a > b {} %>"#).unwrap())
      .unwrap();

  assert_eq!(
    executer.jump_table,
    [(1, 4), (3, 8), (5, 8), (7, 8), (8, 11), (10, 11)]
      .iter()
      .copied()
      .collect()
  )
}

#[test]
fn test_nested_ifs() {
  let executer = Executer::new(
    Parser::parse(r#"<% if a { if a {} else if b { if a {}} else {} if a {} } else { if a {}} %>"#)
      .unwrap(),
  )
  .unwrap();

  assert_eq!(
    executer.jump_table,
    [
      (1, 20),
      (3, 6),
      (5, 16),
      (7, 13),
      (9, 12),
      (11, 12),
      (12, 16),
      (13, 16),
      (15, 16),
      (16, 19),
      (18, 19),
      (19, 26),
      (20, 26),
      (22, 25),
      (24, 25),
      (25, 26)
    ]
    .iter()
    .copied()
    .collect()
  )
}

#[test]
fn test_while_and_if() {
  let executer =
    Executer::new(Parser::parse(r#"<% while a < 3 { if a < 2 {} else {} } %>"#).unwrap()).unwrap();

  assert_eq!(
    executer.jump_table,
    [(1, 10), (3, 6), (5, 9), (6, 9), (8, 9), (9, 1)]
      .iter()
      .copied()
      .collect()
  )
}

#[test]
fn test_for_and_if() {
  let executer =
    Executer::new(Parser::parse(r#"<% for a in v { if a < 2 {} else {} } %>"#).unwrap()).unwrap();

  assert_eq!(
    executer.jump_table,
    [(1, 10), (3, 6), (5, 9), (6, 9), (8, 9), (9, 1)]
      .iter()
      .copied()
      .collect()
  )
}

#[test]
fn test_break_and_continue_jump_table() {
  let executer = Executer::new(
    Parser::parse(r#"<% for a in v { if a < 2 { break; } else { continue; } } %>"#).unwrap(),
  )
  .unwrap();

  assert_eq!(
    executer.jump_table,
    [
      (1, 12),
      (3, 7),
      (4, 12),
      (6, 11),
      (7, 11),
      (8, 1),
      (10, 11),
      (11, 1)
    ]
    .iter()
    .copied()
    .collect()
  )
}

#[test]
fn test_nested_continue_jump_table() {
  let template = r#"<% for a in v { for b in v { if a == b { continue; } %><%= b %><% } if a == 2 { continue } } %>"#;
  let executer = Executer::new(Parser::parse(template).unwrap()).unwrap();

  assert_eq!(
    executer.jump_table,
    [
      (1, 16),
      (3, 12),
      (5, 9),
      (6, 3),
      (8, 9),
      (11, 3),
      (12, 15),
      (13, 1),
      (14, 15),
      (15, 1)
    ]
    .iter()
    .copied()
    .collect()
  )
}
#[test]
fn test_arithmetic_exec() {
  let context_json = r#"{"a": 1, "b":2, "c": 3, "d" : 2, "e" : 6, "f" : 2}"#;
  let mut includes = Mutex::new(HashMap::default());

  {
    let context_value: Value = serde_json::from_str(context_json).unwrap();
    let mut context = Context::new(context_value);

    // b = a = (1 + 2 * 3 - 6 / 2) * 2  == 8
    let template = r"<% b = a = (a + b * c - e / d) * f;e=a+b; if e == 16 { g = 5; h = 6} %>g=<%= g %> h=<%= h %>";
    let executer = Executer::new(Parser::parse(template).unwrap()).unwrap();
    let result = executer
      .render(
        &mut context,
        &HashMap::default(),
        &HashMap::default(),
        template,
        &mut includes,
      )
      .unwrap();

    assert_eq!(result, "g=5 h=6".to_string());
  }
}

#[test]
fn test_while_exec() {
  let template =
    r#"<% a = 0; while a < 3 { if a == 1 { %>One <% } else { %>a = <%= a %> <% } a = a + 1 } %>"#;
  let mut includes = Mutex::new(HashMap::default());
  let executer = Executer::new(Parser::parse(template).unwrap()).unwrap();

  let mut context = Context::new(Value::from(""));
  let result = executer
    .render(
      &mut context,
      &HashMap::default(),
      &HashMap::default(),
      template,
      &mut includes,
    )
    .unwrap();

  assert_eq!(result, "a = 0 One a = 2 ".to_string());
}

#[test]
fn test_for_in_statement() {
  let template = r#"<% for x in arr { %><%= x %> <% } %>"#;
  let mut includes = Mutex::new(HashMap::default());
  let executer = Executer::new(Parser::parse(template).unwrap()).unwrap();

  let context_json = r#"{"arr" : [1,2,3,4,5]}"#;
  let context_value: Value = serde_json::from_str(context_json).unwrap();
  let mut context = Context::new(context_value);

  let result = executer
    .render(
      &mut context,
      &HashMap::default(),
      &HashMap::default(),
      template,
      &mut includes,
    )
    .unwrap();

  assert_eq!(result, "1 2 3 4 5 ".to_string());
}

#[test]
fn test_for_in_statement_with_index() {
  let template = r#"<% for x in arr { %><%= index %> <%= x %>,<% } %>"#;
  let mut includes = Mutex::new(HashMap::default());
  let executer = Executer::new(Parser::parse(template).unwrap()).unwrap();

  let context_json = r#"{"arr" : [1,2,3,4,5]}"#;
  let context_value: Value = serde_json::from_str(context_json).unwrap();
  let mut context = Context::new(context_value);

  let result = executer
    .render(
      &mut context,
      &HashMap::default(),
      &HashMap::default(),
      template,
      &mut includes,
    )
    .unwrap();

  assert_eq!(result, "0 1,1 2,2 3,3 4,4 5,".to_string());
}

#[test]
fn test_nested_for_in_statement() {
  let template = r#"<% for x in arr { for y in arr2 { %><%= x %>*<%= y %>=<%= x * y %>,<% } } %>"#;
  let mut includes = Mutex::new(HashMap::default());
  let executer = Executer::new(Parser::parse(template).unwrap()).unwrap();

  let context_json = r#"{"arr" : [1,2,3,4,5], "arr2" : [1,2]}"#;
  let context_value: Value = serde_json::from_str(context_json).unwrap();
  let mut context = Context::new(context_value);

  let result = executer
    .render(
      &mut context,
      &HashMap::default(),
      &HashMap::default(),
      template,
      &mut includes,
    )
    .unwrap();
  assert_eq!(
    result,
    "1*1=1,1*2=2,2*1=2,2*2=4,3*1=3,3*2=6,4*1=4,4*2=8,5*1=5,5*2=10,".to_string()
  );
}

#[test]
fn test_break() {
  let template = r#"<% for a in v { if a > b { break; } %><%= a %> <% } %>"#;
  let mut includes = Mutex::new(HashMap::default());
  let executer = Executer::new(Parser::parse(template).unwrap()).unwrap();

  let context_json = r#"{"b" : 3, "v" : [1,2,3,4,5]}"#;
  let context_value: Value = serde_json::from_str(context_json).unwrap();
  let mut context = Context::new(context_value);

  let result = executer
    .render(
      &mut context,
      &HashMap::default(),
      &HashMap::default(),
      template,
      &mut includes,
    )
    .unwrap();
  assert_eq!(result, "1 2 3 ".to_string());
}

#[test]
fn test_nested_break() {
  let template =
    r#"<% for a in v { for b in v { if a * b > 10 { break; } %><%= a * b %> <% } } %>"#;
  let mut includes = Mutex::new(HashMap::default());
  let executer = Executer::new(Parser::parse(template).unwrap()).unwrap();

  let context_json = r#"{"v" : [1,2,3,4,5]}"#;
  let context_value: Value = serde_json::from_str(context_json).unwrap();
  let mut context = Context::new(context_value);

  let result = executer
    .render(
      &mut context,
      &HashMap::default(),
      &HashMap::default(),
      template,
      &mut includes,
    )
    .unwrap();
  assert_eq!(result, "1 2 3 4 5 2 4 6 8 10 3 6 9 4 8 5 10 ".to_string());
}

#[test]
fn test_continue() {
  let template = r#"<% for a in v { if a == b { continue; } %><%= a %> <% } %>"#;
  let mut includes = Mutex::new(HashMap::default());
  let executer = Executer::new(Parser::parse(template).unwrap()).unwrap();

  let context_json = r#"{"b" : 3, "v" : [1,2,3,4,5]}"#;
  let context_value: Value = serde_json::from_str(context_json).unwrap();
  let mut context = Context::new(context_value);

  let result = executer
    .render(
      &mut context,
      &HashMap::default(),
      &HashMap::default(),
      template,
      &mut includes,
    )
    .unwrap();
  assert_eq!(result, "1 2 4 5 ".to_string());
}

#[test]
fn test_nested_continue() {
  let template = r#"<% for a in v { for b in v { if a == b { continue; } %><%= b %><% } if a == 2 { continue } } %>"#;
  let mut includes = Mutex::new(HashMap::default());
  let executer = Executer::new(Parser::parse(template).unwrap()).unwrap();

  let context_json = r#"{"v" : [1,2,3,4,5]}"#;
  let context_value: Value = serde_json::from_str(context_json).unwrap();
  let mut context = Context::new(context_value);

  let result = executer
    .render(
      &mut context,
      &HashMap::default(),
      &HashMap::default(),
      template,
      &mut includes,
    )
    .unwrap();
  assert_eq!(result, "23451345124512351234".to_string());
}

#[test]
fn test_function() {
  let template = r#"<%= func(a, b) %>"#;
  let mut includes = Mutex::new(HashMap::default());
  let executer = Executer::new(Parser::parse(template).unwrap()).unwrap();

  let context_json = r#"{"a" : 10, "b" : 20}"#;
  let context_value: Value = serde_json::from_str(context_json).unwrap();
  let mut context = Context::new(context_value);

  let mut functions = HashMap::default();
  functions.insert(
    "func".to_string(),
    FunctionContainer::F2(Box::new(|a: Operand, b: Operand| -> Operand {
      let a = match a {
        Operand::Value(v) => v.as_i64().unwrap(),
        _ => 0,
      };

      let b = match b {
        Operand::Value(v) => v.as_i64().unwrap(),
        _ => 0,
      };

      Operand::Value(Value::from(a + b))
    })),
  );

  let result = executer
    .render(
      &mut context,
      &HashMap::default(),
      &functions,
      template,
      &mut includes,
    )
    .unwrap();
  assert_eq!(result, "30".to_string());
}

#[test]
fn test_function_complex() {
  // (12 + 10 * 20 - 2 + 1 + 3) * 2
  let template = r#"<%= func(func(2, a), func(1+func2(a, b, 2), 3)) * 2 %>"#;
  let mut includes = Mutex::new(HashMap::default());
  let executer = Executer::new(Parser::parse(template).unwrap()).unwrap();

  let context_json = r#"{"a" : 10, "b" : 20}"#;
  let context_value: Value = serde_json::from_str(context_json).unwrap();
  let mut context = Context::new(context_value);

  let mut functions = HashMap::default();
  functions.insert(
    "func".to_string(),
    FunctionContainer::F2(Box::new(|a: Operand, b: Operand| -> Operand {
      let a = match a {
        Operand::Value(v) => v.as_i64().unwrap(),
        _ => 0,
      };

      let b = match b {
        Operand::Value(v) => v.as_i64().unwrap(),
        _ => 0,
      };

      Operand::Value(Value::from(a + b))
    })),
  );

  functions.insert(
    "func2".to_string(),
    FunctionContainer::F3(Box::new(|a: Operand, b: Operand, c: Operand| -> Operand {
      let a = match a {
        Operand::Value(v) => v.as_i64().unwrap(),
        _ => 0,
      };

      let b = match b {
        Operand::Value(v) => v.as_i64().unwrap(),
        _ => 0,
      };

      let c = match c {
        Operand::Value(v) => v.as_i64().unwrap(),
        _ => 0,
      };

      Operand::Value(Value::from(a * b - c))
    })),
  );

  let result = executer
    .render(
      &mut context,
      &HashMap::default(),
      &functions,
      template,
      &mut includes,
    )
    .unwrap();
  assert_eq!(result, "428".to_string());
}

#[test]
fn test_function_with_statements() {
  let template =
    r#"<%= while func(a, b) < 10 { a = a + 1; %>(<%= a %>, <%= b %>) <% b = b + 1 } %>"#;
  let mut includes = Mutex::new(HashMap::default());
  let executer = Executer::new(Parser::parse(template).unwrap()).unwrap();

  let context_json = r#"{"a" : 1, "b" : 2}"#;
  let context_value: Value = serde_json::from_str(context_json).unwrap();
  let mut context = Context::new(context_value);

  let mut functions = HashMap::default();
  functions.insert(
    "func".to_string(),
    FunctionContainer::F2(Box::new(|a: Operand, b: Operand| -> Operand {
      let a = match a {
        Operand::Value(v) => v.as_i64().unwrap(),
        _ => 0,
      };

      let b = match b {
        Operand::Value(v) => v.as_i64().unwrap(),
        _ => 0,
      };

      Operand::Value(Value::from(a + b))
    })),
  );

  let result = executer
    .render(
      &mut context,
      &HashMap::default(),
      &functions,
      template,
      &mut includes,
    )
    .unwrap();
  assert_eq!(result, "(2, 2) (3, 3) (4, 4) (5, 5) ".to_string());
}

#[test]
fn test_function_taking_object() {
  let template = r#"<%- func(a) %>"#;
  let mut includes = Mutex::new(HashMap::default());
  let executer = Executer::new(Parser::parse(template).unwrap()).unwrap();

  let context_value: Value = json!({"a" : {"b" : 1, "c" : 2}});
  let mut context = Context::new(context_value);

  let mut functions = HashMap::default();
  functions.insert(
    "func".to_string(),
    FunctionContainer::F1(Box::new(|a: Operand| -> Operand {
      let a = match a {
        Operand::Value(v) => v,
        _ => panic!(""),
      };

      Operand::Value(Value::String(a.to_string()))
    })),
  );

  let result = executer
    .render(
      &mut context,
      &HashMap::default(),
      &functions,
      template,
      &mut includes,
    )
    .unwrap();

  assert_eq!(result, r#"{"b":1,"c":2}"#.to_string());
}

#[test]
fn test_predefined_include_function() {
  let template = r#"<%- include(a) %>"#;
  let mut includes = Mutex::new(HashMap::default());
  let executer = Executer::new(Parser::parse(template).unwrap()).unwrap();

  let context_json = r#"{"a" : "./tests/test_include.html"}"#;
  let context_value: Value = serde_json::from_str(context_json).unwrap();
  let mut context = Context::new(context_value);

  let functions = HashMap::default();
  let result = executer
    .render(
      &mut context,
      &HashMap::default(),
      &functions,
      template,
      &mut includes,
    )
    .unwrap();

  assert_eq!(result, "<html>test include</html>\n".to_string());
}

#[test]
fn test_predefined_include_template_function() {
  let template = r#"<%- include_template(a) %>"#;
  let mut includes = Mutex::new(HashMap::default());
  let executer = Executer::new(Parser::parse(template).unwrap()).unwrap();

  let context_json = r#"{"a" : "some_template", "title" : "TITLE"}"#;
  let context_value: Value = serde_json::from_str(context_json).unwrap();
  let mut context = Context::new(context_value);

  let functions = HashMap::default();
  let mut templates = HashMap::default();

  // Register "some_template". This will be loaded from include_template function.
  let some_template = "<html><%= title %></html>".to_string();
  templates.insert(
    "some_template".to_string(),
    (
      Executer::new(Parser::parse(&some_template).unwrap()).unwrap(),
      some_template,
    ),
  );

  let result = executer
    .render(
      &mut context,
      &templates,
      &functions,
      template,
      &mut includes,
    )
    .unwrap();

  assert_eq!(result, "<html>TITLE</html>".to_string());
}

#[test]
fn test_accessor() {
  let template = r#"<%= 10000 + a[x][y + "c"][z] - 3%>"#;
  let mut includes = Mutex::new(HashMap::default());
  let executer = Executer::new(Parser::parse(template).unwrap()).unwrap();
  let context_json = r#"{"a" : { "b" : { "cc" : { "1" : 1234}}}, "x" : "b", "y" : "c", "z" : 1}"#;

  let context_value: Value = serde_json::from_str(context_json).unwrap();
  let mut context = Context::new(context_value);

  let result = executer
    .render(
      &mut context,
      &HashMap::default(),
      &HashMap::default(),
      template,
      &mut includes,
    )
    .unwrap();
  assert_eq!(result, "11231".to_string());
}

#[test]
fn test_accessor_and_function() {
  let template = r#"<%= a[x][func2(y, "c")+"d"][func(z)] %>"#;
  let mut includes = Mutex::new(HashMap::default());
  let executer = Executer::new(Parser::parse(template).unwrap()).unwrap();

  let context_json = r#"{"a" : { "b" : { "ccd" : { "2" : 1234}}}, "x" : "b", "y" : "c", "z" : 1}"#;
  let context_value: Value = serde_json::from_str(context_json).unwrap();
  let mut context = Context::new(context_value);

  let mut functions = HashMap::default();
  functions.insert(
    "func2".to_string(),
    FunctionContainer::F2(Box::new(|a: Operand, b: Operand| -> Operand {
      let mut a = match a {
        Operand::Value(v) => v.as_str().unwrap().to_string(),
        _ => String::new(),
      };

      let b = match b {
        Operand::Value(v) => v.as_str().unwrap().to_string(),
        _ => String::new(),
      };

      a.push_str(&b);
      Operand::Value(Value::from(a))
    })),
  );

  functions.insert(
    "func".to_string(),
    FunctionContainer::F1(Box::new(|a: Operand| -> Operand {
      let a = match a {
        Operand::Value(v) => v.as_i64().unwrap(),
        _ => 0,
      };

      Operand::Value(Value::from(a + 1))
    })),
  );

  let result = executer
    .render(
      &mut context,
      &HashMap::default(),
      &functions,
      template,
      &mut includes,
    )
    .unwrap();
  assert_eq!(result, "1234".to_string());
}

#[test]
fn test_accessor_with_dot() {
  let template = r#"<%= 10000 + a[x].cc[z] - 3%>"#;
  let mut includes = Mutex::new(HashMap::default());
  let executer = Executer::new(Parser::parse(template).unwrap()).unwrap();
  let context_json = r#"{"a" : { "b" : { "cc" : { "1" : 1234}}}, "x" : "b", "y" : "c", "z" : 1}"#;

  let context_value: Value = serde_json::from_str(context_json).unwrap();
  let mut context = Context::new(context_value);

  let result = executer
    .render(
      &mut context,
      &HashMap::default(),
      &HashMap::default(),
      template,
      &mut includes,
    )
    .unwrap();
  assert_eq!(result, "11231".to_string());
}
