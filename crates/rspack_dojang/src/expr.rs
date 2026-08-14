use serde_json::Value;

use crate::dojang::DojangOptions;

#[derive(PartialEq, Debug, Clone)]
pub enum Op {
  Not,          // !
  Or,           // ||
  And,          // &&
  ParenOpen,    // (
  ParenClose,   // )
  BracketOpen,  // [
  BracketClose, // ]
  Equal,        // ==
  NotEqual,     // !=
  Less,         // <
  LessEq,       // <=
  Greater,      // >
  GreaterEq,    // >=,
  Assign,       // =
  Plus,         // +
  Minus,        // -
  Multiply,     // *
  Divide,       // /
  Comma,        // ,
  Dot,          // .
  Operand(Operand),
}

#[derive(PartialEq, Debug, Clone)]
pub enum Keyword {
  In,
  Continue,
  Break,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Operand {
  Value(Value),
  Array(Vec<Operand>),
  Object(Object),
  Keyword(Keyword),
  Function(String),
}

// Name that will be found in the execution context.
#[derive(PartialEq, Debug, Clone)]
pub struct Object {
  pub name: String,
}

#[derive(PartialEq, Debug)]
pub struct Tokens {
  pub ops: Vec<Op>,
}

#[derive(PartialEq, Debug)]
pub enum Action<T> {
  Show(Show<T>),
  If(T),    // If condition
  While(T), // Loop condition
  For(T),   // For condition
  Else(),   // Else
  End(),    // Closing }
  Do(T),
}

impl Action<Tokens> {
  pub fn add_op(&mut self, op: Op) {
    match self {
      Action::Show(show) => match show {
        Show::ExprEscaped(e) => e.ops.push(op),
        Show::ExprUnescaped(e) => e.ops.push(op),
        _ => panic!("Cannot add op to Show; op {op:?}"),
      },
      Action::If(expr) => {
        expr.ops.push(op);
      }
      Action::While(expr) => {
        expr.ops.push(op);
      }
      Action::For(expr) => {
        expr.ops.push(op);
      }
      Action::Do(expr) => {
        expr.ops.push(op);
      }
      _ => panic!("Cannot add op to {self:?}; op {op:?}"),
    }
  }
}

#[derive(PartialEq, Debug)]
pub enum Show<T> {
  Html { start: usize, end: usize }, // [start, end) of the original template.
  ExprEscaped(T),
  ExprUnescaped(T),
}

#[derive(PartialEq, Debug)]
pub struct Parser {
  pub parse_tree: Vec<Action<Tokens>>,
}

impl Parser {
  pub fn parse(template: &str) -> Result<Self, String> {
    Parser::parse_with_options(template, DojangOptions::default())
  }
  pub fn parse_with_options(mut template: &str, options: DojangOptions) -> Result<Self, String> {
    let mut parse_tree = Vec::new();

    let mut index_offset = 0;
    while !template.is_empty() {
      match template.find("<%") {
        Some(tag_start) => {
          let before_tag = &template[..tag_start];
          if !before_tag.is_empty() {
            parse_tree.push(Action::Show(Show::Html {
              start: index_offset,
              end: index_offset + tag_start,
            }));
          }

          let after_tag = &template[tag_start + 2..];
          index_offset += tag_start + 2;

          match after_tag.find("%>") {
            Some(tag_end) => {
              //let escape = options.escape.as_str();
              let escape = options.escape.as_str();
              let unescape = options.unescape.as_str();

              let tag_to_parse = match &template[tag_start + 2..tag_start + 3] {
                str if str == escape => {
                  parse_tree.push(Action::Show(Show::ExprEscaped(Tokens { ops: Vec::new() })));
                  &after_tag[1..tag_end]
                }
                str if str == unescape => {
                  parse_tree.push(Action::Show(Show::ExprUnescaped(Tokens {
                    ops: Vec::new(),
                  })));
                  &after_tag[1..tag_end]
                }
                "#" => {
                  // Ignore the comment.
                  template = &after_tag[tag_end + 2..];
                  index_offset += tag_end + 2;
                  continue;
                }
                "%" => {
                  // Show '%'.
                  parse_tree.push(Action::Show(Show::Html {
                    start: tag_start + 2,
                    end: tag_start + 3,
                  }));

                  template = &after_tag[tag_end + 2..];
                  index_offset += tag_end + 2;
                  continue;
                }
                _ => {
                  parse_tree.push(Action::Do(Tokens { ops: Vec::new() }));
                  &after_tag[..tag_end]
                }
              };

              Parser::parse_tag(tag_to_parse, &mut parse_tree)?;
              template = &after_tag[tag_end + 2..];
              index_offset += tag_end + 2;
            }
            _ => {
              return Err("Unmatched %> tag found".to_string());
            }
          }
        }
        _ => {
          parse_tree.push(Action::Show(Show::Html {
            start: index_offset,
            end: index_offset + template.len(),
          }));
          break;
        }
      }
    }

    Ok(Parser { parse_tree })
  }

  fn parse_tag(template: &str, parse_tree: &mut Vec<Action<Tokens>>) -> Result<(), String> {
    let mut current = 0;
    let mut token_begin = 0;

    while current < template.len() {
      let current_char = template.chars().nth(current).unwrap();

      // Check string literal.
      if current_char == '"' || current_char == '\'' {
        match Parser::handle_string_literal(template, current, parse_tree) {
          Ok(end_of_literal) => {
            current = end_of_literal;
            token_begin = current;
            continue;
          }
          Err(e) => return Err(e),
        }
      }

      // Handle Operators.
      if template[current..current + 1]
        .find(
          &[
            '&', '|', '(', ')', '!', '=', '<', '>', '+', '-', '*', '/', ',', '[', ']', '.',
          ][..],
        )
        .is_some()
      {
        Parser::handle_operand(&template[token_begin..current], parse_tree);

        match Parser::handle_operator(template, current, parse_tree.last_mut().unwrap()) {
          Ok(end_of_op) => {
            current = end_of_op;
            token_begin = current;
            continue;
          }
          Err(e) => return Err(e),
        }
      }

      if current_char == ' ' {
        let token = &template[token_begin..current];
        Parser::handle_token(token, parse_tree);

        token_begin = current + 1;
      } else if current_char == '{' {
        Parser::handle_token(&template[token_begin..current], parse_tree);
        parse_tree.push(Action::Do(Tokens { ops: Vec::new() }));
        token_begin = current + 1;
      } else if current_char == '}' {
        Parser::handle_token(&template[token_begin..current], parse_tree);
        parse_tree.push(Action::End());
        token_begin = current + 1;
      } else if current_char == ';' {
        Parser::handle_token(&template[token_begin..current], parse_tree);
        match parse_tree.pop().unwrap() {
          Action::Do(expr) => {
            parse_tree.push(Action::Do(expr));
            parse_tree.push(Action::Do(Tokens { ops: Vec::new() }));
          }
          Action::Show(show) => match show {
            Show::Html { start: _, end: _ } => {
              panic!("Show should not contain html");
            }
            Show::ExprUnescaped(expr) => {
              // For <%= ... %> tags, only the last expression will be shown.
              // In other words, <%= a; b %> is identical to <%= b %>
              parse_tree.push(Action::Do(expr));
              parse_tree.push(Action::Show(Show::ExprUnescaped(Tokens {
                ops: Vec::new(),
              })))
            }
            Show::ExprEscaped(expr) => {
              parse_tree.push(Action::Do(expr));
              parse_tree.push(Action::Show(Show::ExprEscaped(Tokens { ops: Vec::new() })))
            }
          },
          _ => return Err(format!("Invalid position of ; at {template}")),
        }
        token_begin = current + 1;
      }

      current += 1;
    }

    Parser::handle_token(&template[token_begin..template.len()], parse_tree);
    Ok(())
  }

  // Returns the next of the end of the string literal.
  fn handle_string_literal(
    template: &str,
    start: usize,
    parse_tree: &mut [Action<Tokens>],
  ) -> Result<usize, String> {
    let starting_quote = template.chars().nth(start).unwrap();
    assert!(starting_quote == '"' || starting_quote == '\'');
    let string_literal_end;

    // Read until closing " is found.
    let mut find_end_quote_start = start + 1;
    loop {
      match template[find_end_quote_start..].find(starting_quote) {
        Some(end_quote_pos) => {
          if end_quote_pos == 0
            || template[find_end_quote_start..]
              .chars()
              .nth(end_quote_pos - 1)
              .unwrap()
              != '\\'
          {
            string_literal_end = find_end_quote_start + end_quote_pos;
            break;
          } else {
            find_end_quote_start += end_quote_pos + 1;
          }
        }
        _ => {
          return Err(format!(
            "template '{template}' does not have terminated {starting_quote} for string."
          ));
        }
      }
    }

    parse_tree
      .last_mut()
      .unwrap()
      .add_op(Op::Operand(Operand::Value(Value::from(
        template[start + 1..string_literal_end].to_string(),
      ))));

    Ok(string_literal_end + 1)
  }

  fn handle_operator(
    template: &str,
    start: usize,
    action: &mut Action<Tokens>,
  ) -> Result<usize, String> {
    // Check for the binary operators.
    if template.len() >= start + 2
      && let Some(op) = check_2_char_op(&template[start..start + 2])
    {
      action.add_op(op);
      return Ok(start + 2);
    }

    // Check for the unary operators.
    match template.chars().nth(start).unwrap() {
      '!' => action.add_op(Op::Not),
      '(' => action.add_op(Op::ParenOpen),
      ')' => action.add_op(Op::ParenClose),
      '[' => action.add_op(Op::BracketOpen),
      ']' => action.add_op(Op::BracketClose),
      '<' => action.add_op(Op::Less),
      '>' => action.add_op(Op::Greater),
      '=' => action.add_op(Op::Assign),
      '+' => action.add_op(Op::Plus),
      '-' => action.add_op(Op::Minus),
      '*' => action.add_op(Op::Multiply),
      '/' => action.add_op(Op::Divide),
      ',' => action.add_op(Op::Comma),
      '.' => action.add_op(Op::Dot),
      c => {
        return Err(format!("Unknown operator at '{template}', unknown : {c}"));
      }
    }

    Ok(start + 1)
  }

  fn handle_token(token: &str, parse_tree: &mut Vec<Action<Tokens>>) {
    if token == "if" {
      parse_tree.push(Action::If(Tokens { ops: Vec::new() }));
    } else if token == "while" {
      parse_tree.push(Action::While(Tokens { ops: Vec::new() }));
    } else if token == "for" {
      parse_tree.push(Action::For(Tokens { ops: Vec::new() }));
    } else if token == "else" {
      parse_tree.push(Action::Else());
    } else {
      Parser::handle_operand(token, parse_tree);
    }
  }

  fn handle_operand(mut operand: &str, parse_tree: &mut Vec<Action<Tokens>>) {
    if operand.is_empty() {
      return;
    }

    operand = operand.trim();

    if let Action::End() = parse_tree.last().unwrap() {
      parse_tree.push(Action::Do(Tokens { ops: Vec::new() }));
    }

    let action = parse_tree.last_mut().unwrap();

    let first_char = operand.chars().nth(0).unwrap();
    if first_char.is_ascii_digit() {
      if operand.contains('.') {
        action.add_op(Op::Operand(Operand::Value(Value::from(
          operand.parse().unwrap_or(0.0),
        ))));
      } else {
        action.add_op(Op::Operand(Operand::Value(Value::from(
          operand.parse().unwrap_or(0),
        ))));
      }
    } else {
      match operand {
        "in" => {
          action.add_op(Op::Operand(Operand::Keyword(Keyword::In)));
        }
        "break" => {
          action.add_op(Op::Operand(Operand::Keyword(Keyword::Break)));
        }
        "continue" => {
          action.add_op(Op::Operand(Operand::Keyword(Keyword::Continue)));
        }
        _ => action.add_op(Op::Operand(Operand::Object(Object {
          name: operand.to_string(),
        }))),
      }
    }
  }
}

fn check_2_char_op(s: &str) -> Option<Op> {
  if s.len() != 2 {
    return None;
  }

  if s == "&&" {
    return Some(Op::And);
  } else if s == "||" {
    return Some(Op::Or);
  } else if s == "==" {
    return Some(Op::Equal);
  } else if s == "!=" {
    return Some(Op::NotEqual);
  } else if s == "<=" {
    return Some(Op::LessEq);
  } else if s == ">=" {
    return Some(Op::GreaterEq);
  }

  None
}

// Returns the operator priority; Lower number == Higher priority.
// Used the precedence table at
//   https://en.cppreference.com/w/c/language/operator_precedence.
pub fn operator_priority(op: &Op) -> u32 {
  match op {
    Op::Not => 2,
    Op::Multiply => 3,
    Op::Divide => 3,
    Op::Plus => 4,
    Op::Minus => 4,
    Op::Greater => 6,
    Op::GreaterEq => 6,
    Op::Less => 6,
    Op::LessEq => 6,
    Op::Equal => 7,
    Op::NotEqual => 7,
    Op::And => 11,
    Op::Or => 12,
    Op::Assign => 14,
    Op::BracketOpen => 100,
    Op::BracketClose => 100,
    Op::ParenOpen => 100,
    Op::ParenClose => 100,
    _ => panic!("Oops {op:?}"),
  }
}

pub fn operator_num_operands(op: &Op) -> usize {
  match op {
    Op::Not => 1,
    _ => 2,
  }
}

pub fn get_nth_element_from_operand(operand: &Operand, index: usize) -> Option<Value> {
  match operand {
    Operand::Value(value) => {
      if value.is_array() {
        let arr = value.as_array().unwrap();
        if index >= arr.len() {
          return None;
        }

        return Some(arr[index].clone());
      } else if value.is_string() {
        let s = value.as_str().unwrap();
        if index >= s.len() {
          return None;
        }

        return Some(Value::from(s.chars().nth(index).unwrap().to_string()));
      }

      return None;
    }
    Operand::Array(arr) => {
      if index >= arr.len() {
        return None;
      }

      match &arr[index] {
        Operand::Value(value) => {
          return Some(value.clone());
        }
        _ => {
          return None;
        }
      }
    }
    _ => {}
  }

  None
}

pub fn convert_value_to_operand(value: Value) -> Operand {
  match value {
    Value::Object(obj) => Operand::Value(Value::Object(obj)),
    Value::Array(arr) => {
      let mut vec = Vec::new();
      for elem in arr {
        vec.push(Operand::Value(elem));
      }
      Operand::Array(vec)
    }
    v => Operand::Value(v),
  }
}

pub fn convert_operand_to_value(operand: Operand) -> Value {
  match operand {
    Operand::Value(v) => v,
    Operand::Array(arr) => {
      let mut vec = Vec::new();
      for elem in arr {
        vec.push(convert_operand_to_value(elem));
      }
      Value::from(vec)
    }
    _ => {
      panic!("Unable to convert object to value.")
    }
  }
}

#[test]
fn parse_simple_expr_with_binary() {
  let result = Parser::parse("<% some == 3 %>");

  let expected_expr = Parser {
    parse_tree: vec![Action::Do(Tokens {
      ops: vec![
        Op::Operand(Operand::Object(Object {
          name: "some".to_string(),
        })),
        Op::Equal,
        Op::Operand(Operand::Value(Value::from(3))),
      ],
    })],
  };
  assert_eq!(result.unwrap(), expected_expr);
}

#[test]
fn parse_simple_expr_with_unary() {
  let result = Parser::parse("<% !some_value %>");

  let expected_expr = Parser {
    parse_tree: vec![Action::Do(Tokens {
      ops: vec![
        Op::Not,
        Op::Operand(Operand::Object(Object {
          name: "some_value".to_string(),
        })),
      ],
    })],
  };

  assert_eq!(result.unwrap(), expected_expr);
}

#[test]
fn parse_complex_binary_and_unary() {
  let result = Parser::parse("<% (a + b * c - e / d) * f %>");

  let expected_expr = Parser {
    parse_tree: vec![Action::Do(Tokens {
      ops: vec![
        Op::ParenOpen,
        Op::Operand(Operand::Object(Object {
          name: "a".to_string(),
        })),
        Op::Plus,
        Op::Operand(Operand::Object(Object {
          name: "b".to_string(),
        })),
        Op::Multiply,
        Op::Operand(Operand::Object(Object {
          name: "c".to_string(),
        })),
        Op::Minus,
        Op::Operand(Operand::Object(Object {
          name: "e".to_string(),
        })),
        Op::Divide,
        Op::Operand(Operand::Object(Object {
          name: "d".to_string(),
        })),
        Op::ParenClose,
        Op::Multiply,
        Op::Operand(Operand::Object(Object {
          name: "f".to_string(),
        })),
      ],
    })],
  };

  assert_eq!(result.unwrap(), expected_expr);
}

#[test]
fn parse_simple_literal() {
  let result = Parser::parse(r#"<% some_value == "abc" %>"#);
  let expected_expr = Parser {
    parse_tree: vec![Action::Do(Tokens {
      ops: vec![
        Op::Operand(Operand::Object(Object {
          name: "some_value".to_string(),
        })),
        Op::Equal,
        Op::Operand(Operand::Value(Value::from("abc".to_string()))),
      ],
    })],
  };

  assert_eq!(result.unwrap(), expected_expr);
}

#[test]
fn parse_simple_literal_single_quote() {
  let result = Parser::parse(r#"<% some_value == 'a"bc' %>"#);
  let expected_expr = Parser {
    parse_tree: vec![Action::Do(Tokens {
      ops: vec![
        Op::Operand(Operand::Object(Object {
          name: "some_value".to_string(),
        })),
        Op::Equal,
        Op::Operand(Operand::Value(Value::from("a\"bc".to_string()))),
      ],
    })],
  };

  assert_eq!(result.unwrap(), expected_expr);
}
#[test]
fn parse_literal_with_escape() {
  let result = Parser::parse(r#"<% some_value == "a\"bc" abc %>"#);
  let expected_expr = Parser {
    parse_tree: vec![Action::Do(Tokens {
      ops: vec![
        Op::Operand(Operand::Object(Object {
          name: "some_value".to_string(),
        })),
        Op::Equal,
        Op::Operand(Operand::Value(Value::from("a\\\"bc".to_string()))),
        Op::Operand(Operand::Object(Object {
          name: "abc".to_string(),
        })),
      ],
    })],
  };

  assert_eq!(result.unwrap(), expected_expr);
}

#[test]
fn parse_complex_expr() {
  let result = Parser::parse("<% !some_value && ((var1 != var2) || some <= val) %>");
  let expected_expr = Parser {
    parse_tree: vec![Action::Do(Tokens {
      ops: vec![
        Op::Not,
        Op::Operand(Operand::Object(Object {
          name: "some_value".to_string(),
        })),
        Op::And,
        Op::ParenOpen,
        Op::ParenOpen,
        Op::Operand(Operand::Object(Object {
          name: "var1".to_string(),
        })),
        Op::NotEqual,
        Op::Operand(Operand::Object(Object {
          name: "var2".to_string(),
        })),
        Op::ParenClose,
        Op::Or,
        Op::Operand(Operand::Object(Object {
          name: "some".to_string(),
        })),
        Op::LessEq,
        Op::Operand(Operand::Object(Object {
          name: "val".to_string(),
        })),
        Op::ParenClose,
      ],
    })],
  };

  assert_eq!(result.unwrap(), expected_expr);
}

#[test]
fn parse_complex_expr2() {
  let result = Parser::parse(r"<% var1 >= var2 && var2 <= var3 || !var3  %>");
  let expected_expr = Parser {
    parse_tree: vec![Action::Do(Tokens {
      ops: vec![
        Op::Operand(Operand::Object(Object {
          name: "var1".to_string(),
        })),
        Op::GreaterEq,
        Op::Operand(Operand::Object(Object {
          name: "var2".to_string(),
        })),
        Op::And,
        Op::Operand(Operand::Object(Object {
          name: "var2".to_string(),
        })),
        Op::LessEq,
        Op::Operand(Operand::Object(Object {
          name: "var3".to_string(),
        })),
        Op::Or,
        Op::Not,
        Op::Operand(Operand::Object(Object {
          name: "var3".to_string(),
        })),
      ],
    })],
  };

  assert_eq!(result.unwrap(), expected_expr);
}

#[test]
fn parse_if_statement() {
  let result = Parser::parse(r#"<% if var1 >= var2 && var3 < "}" { "hello" } %>"#);
  let expected_expr = Parser {
    parse_tree: vec![
      Action::Do(Tokens { ops: vec![] }),
      Action::If(Tokens {
        ops: vec![
          Op::Operand(Operand::Object(Object {
            name: "var1".to_string(),
          })),
          Op::GreaterEq,
          Op::Operand(Operand::Object(Object {
            name: "var2".to_string(),
          })),
          Op::And,
          Op::Operand(Operand::Object(Object {
            name: "var3".to_string(),
          })),
          Op::Less,
          Op::Operand(Operand::Value(Value::from("}".to_string()))),
        ],
      }),
      Action::Do(Tokens {
        ops: vec![Op::Operand(Operand::Value(Value::from(
          "hello".to_string(),
        )))],
      }),
      Action::End(),
    ],
  };

  assert_eq!(result.unwrap(), expected_expr);
}

#[test]
fn parse_multiple_if_statement() {
  let result = Parser::parse(r#"<% if var1>=var2{if var2>var3{} if !var1 {}} %>"#);
  let expected_expr = Parser {
    parse_tree: vec![
      Action::Do(Tokens { ops: vec![] }),
      Action::If(Tokens {
        ops: vec![
          Op::Operand(Operand::Object(Object {
            name: "var1".to_string(),
          })),
          Op::GreaterEq,
          Op::Operand(Operand::Object(Object {
            name: "var2".to_string(),
          })),
        ],
      }),
      Action::Do(Tokens { ops: vec![] }),
      Action::If(Tokens {
        ops: vec![
          Op::Operand(Operand::Object(Object {
            name: "var2".to_string(),
          })),
          Op::Greater,
          Op::Operand(Operand::Object(Object {
            name: "var3".to_string(),
          })),
        ],
      }),
      Action::Do(Tokens { ops: vec![] }),
      Action::End(),
      Action::If(Tokens {
        ops: vec![
          Op::Not,
          Op::Operand(Operand::Object(Object {
            name: "var1".to_string(),
          })),
        ],
      }),
      Action::Do(Tokens { ops: vec![] }),
      Action::End(),
      Action::End(),
    ],
  };

  assert_eq!(result.unwrap(), expected_expr);
}

#[test]
fn parse_if_else_statement() {
  let result = Parser::parse(r#"<% if var1>=var2{ "b" } else if var1==3{"a"}else{"c"}%>"#);
  let expected_expr = Parser {
    parse_tree: vec![
      Action::Do(Tokens { ops: vec![] }),
      Action::If(Tokens {
        ops: vec![
          Op::Operand(Operand::Object(Object {
            name: "var1".to_string(),
          })),
          Op::GreaterEq,
          Op::Operand(Operand::Object(Object {
            name: "var2".to_string(),
          })),
        ],
      }),
      Action::Do(Tokens {
        ops: vec![Op::Operand(Operand::Value(Value::from("b".to_string())))],
      }),
      Action::End(),
      Action::Else(),
      Action::If(Tokens {
        ops: vec![
          Op::Operand(Operand::Object(Object {
            name: "var1".to_string(),
          })),
          Op::Equal,
          Op::Operand(Operand::Value(Value::from(3))),
        ],
      }),
      Action::Do(Tokens {
        ops: vec![Op::Operand(Operand::Value(Value::from("a".to_string())))],
      }),
      Action::End(),
      Action::Else(),
      Action::Do(Tokens {
        ops: vec![Op::Operand(Operand::Value(Value::from("c".to_string())))],
      }),
      Action::End(),
    ],
  };

  assert_eq!(result.unwrap(), expected_expr);
}

#[test]
fn parse_while_statement() {
  let result = Parser::parse(r#"<% while var1 >= var2{ "hello"} %>"#);
  let expected_expr = Parser {
    parse_tree: vec![
      Action::Do(Tokens { ops: vec![] }),
      Action::While(Tokens {
        ops: vec![
          Op::Operand(Operand::Object(Object {
            name: "var1".to_string(),
          })),
          Op::GreaterEq,
          Op::Operand(Operand::Object(Object {
            name: "var2".to_string(),
          })),
        ],
      }),
      Action::Do(Tokens {
        ops: vec![Op::Operand(Operand::Value(Value::from(
          "hello".to_string(),
        )))],
      }),
      Action::End(),
    ],
  };

  assert_eq!(result.unwrap(), expected_expr);
}

#[test]
fn parse_for_statement() {
  let result = Parser::parse(r#"<% for a in vec{ "hello"} %>"#);
  let expected_expr = Parser {
    parse_tree: vec![
      Action::Do(Tokens { ops: vec![] }),
      Action::For(Tokens {
        ops: vec![
          Op::Operand(Operand::Object(Object {
            name: "a".to_string(),
          })),
          Op::Operand(Operand::Keyword(Keyword::In)),
          Op::Operand(Operand::Object(Object {
            name: "vec".to_string(),
          })),
        ],
      }),
      Action::Do(Tokens {
        ops: vec![Op::Operand(Operand::Value(Value::from(
          "hello".to_string(),
        )))],
      }),
      Action::End(),
    ],
  };

  assert_eq!(result.unwrap(), expected_expr);
}

#[test]
fn parse_function() {
  let result = Parser::parse(r#"<% func(1+1, a, b) + x %>"#);
  let expected_expr = Parser {
    parse_tree: vec![Action::Do(Tokens {
      ops: vec![
        Op::Operand(Operand::Object(Object {
          name: "func".to_string(),
        })),
        Op::ParenOpen,
        Op::Operand(Operand::Value(Value::from(1))),
        Op::Plus,
        Op::Operand(Operand::Value(Value::from(1))),
        Op::Comma,
        Op::Operand(Operand::Object(Object {
          name: "a".to_string(),
        })),
        Op::Comma,
        Op::Operand(Operand::Object(Object {
          name: "b".to_string(),
        })),
        Op::ParenClose,
        Op::Plus,
        Op::Operand(Operand::Object(Object {
          name: "x".to_string(),
        })),
      ],
    })],
  };

  assert_eq!(result.unwrap(), expected_expr);
}

#[test]
fn parse_bracket() {
  let result = Parser::parse(r#"<% func(a[1][2]) + x %>"#);
  let expected_expr = Parser {
    parse_tree: vec![Action::Do(Tokens {
      ops: vec![
        Op::Operand(Operand::Object(Object {
          name: "func".to_string(),
        })),
        Op::ParenOpen,
        Op::Operand(Operand::Object(Object {
          name: "a".to_string(),
        })),
        Op::BracketOpen,
        Op::Operand(Operand::Value(Value::from(1))),
        Op::BracketClose,
        Op::BracketOpen,
        Op::Operand(Operand::Value(Value::from(2))),
        Op::BracketClose,
        Op::ParenClose,
        Op::Plus,
        Op::Operand(Operand::Object(Object {
          name: "x".to_string(),
        })),
      ],
    })],
  };

  assert_eq!(result.unwrap(), expected_expr);
}

#[test]
fn parse_property_accessor() {
  let result = Parser::parse(r#"<% 1 + a.b[123].c %>"#);
  let expected_expr = Parser {
    parse_tree: vec![Action::Do(Tokens {
      ops: vec![
        Op::Operand(Operand::Value(Value::from(1))),
        Op::Plus,
        Op::Operand(Operand::Object(Object {
          name: "a".to_string(),
        })),
        Op::Dot,
        Op::Operand(Operand::Object(Object {
          name: "b".to_string(),
        })),
        Op::BracketOpen,
        Op::Operand(Operand::Value(Value::from(123))),
        Op::BracketClose,
        Op::Dot,
        Op::Operand(Operand::Object(Object {
          name: "c".to_string(),
        })),
      ],
    })],
  };

  assert_eq!(result.unwrap(), expected_expr);
}

#[test]
fn parse_tags_simple() {
  let template = r#"<html><% if a { %>some tag<% } %><body></body></html>"#;
  let result = Parser::parse(template);
  let expected_expr = Parser {
    parse_tree: vec![
      Action::Show(Show::Html { start: 0, end: 6 }),
      Action::Do(Tokens { ops: vec![] }),
      Action::If(Tokens {
        ops: vec![Op::Operand(Operand::Object(Object {
          name: "a".to_string(),
        }))],
      }),
      Action::Do(Tokens { ops: vec![] }),
      Action::Show(Show::Html { start: 18, end: 26 }),
      Action::Do(Tokens { ops: vec![] }),
      Action::End(),
      Action::Show(Show::Html { start: 33, end: 53 }),
    ],
  };

  assert_eq!(result.unwrap(), expected_expr);
  assert_eq!(&template[18..26], "some tag");
  assert_eq!(&template[33..53], "<body></body></html>");
}

#[test]
fn parse_tags_escaped() {
  let result = Parser::parse(r#"<html><% if a { %><%= data %><% } %>"#);
  let expected_expr = Parser {
    parse_tree: vec![
      Action::Show(Show::Html { start: 0, end: 6 }),
      Action::Do(Tokens { ops: vec![] }),
      Action::If(Tokens {
        ops: vec![Op::Operand(Operand::Object(Object {
          name: "a".to_string(),
        }))],
      }),
      Action::Do(Tokens { ops: vec![] }),
      Action::Show(Show::ExprEscaped(Tokens {
        ops: vec![Op::Operand(Operand::Object(Object {
          name: "data".to_string(),
        }))],
      })),
      Action::Do(Tokens { ops: vec![] }),
      Action::End(),
    ],
  };

  assert_eq!(result.unwrap(), expected_expr);
}

#[test]
fn parse_multiple_expressions() {
  let result = Parser::parse(r#"<html><% a; b; c %><%= a; b %><%- a; b  %>"#);
  let expected_expr = Parser {
    parse_tree: vec![
      Action::Show(Show::Html { start: 0, end: 6 }),
      Action::Do(Tokens {
        ops: vec![Op::Operand(Operand::Object(Object {
          name: "a".to_string(),
        }))],
      }),
      Action::Do(Tokens {
        ops: vec![Op::Operand(Operand::Object(Object {
          name: "b".to_string(),
        }))],
      }),
      Action::Do(Tokens {
        ops: vec![Op::Operand(Operand::Object(Object {
          name: "c".to_string(),
        }))],
      }),
      Action::Do(Tokens {
        ops: vec![Op::Operand(Operand::Object(Object {
          name: "a".to_string(),
        }))],
      }),
      Action::Show(Show::ExprEscaped(Tokens {
        ops: vec![Op::Operand(Operand::Object(Object {
          name: "b".to_string(),
        }))],
      })),
      Action::Do(Tokens {
        ops: vec![Op::Operand(Operand::Object(Object {
          name: "a".to_string(),
        }))],
      }),
      Action::Show(Show::ExprUnescaped(Tokens {
        ops: vec![Op::Operand(Operand::Object(Object {
          name: "b".to_string(),
        }))],
      })),
    ],
  };

  assert_eq!(result.unwrap(), expected_expr);
}

#[test]
fn parse_ignore_comment() {
  let result = Parser::parse(r#"<html><%# some comment %></html>"#);
  let expected_expr = Parser {
    parse_tree: vec![
      Action::Show(Show::Html { start: 0, end: 6 }),
      Action::Show(Show::Html { start: 25, end: 32 }),
    ],
  };

  assert_eq!(result.unwrap(), expected_expr);
}

#[test]
fn parse_percent_tag() {
  let result = Parser::parse(r#"<html><%% %></html>"#);
  let expected_expr = Parser {
    parse_tree: vec![
      Action::Show(Show::Html { start: 0, end: 6 }),
      Action::Show(Show::Html { start: 8, end: 9 }),
      Action::Show(Show::Html { start: 12, end: 19 }),
    ],
  };

  assert_eq!(result.unwrap(), expected_expr);
}
