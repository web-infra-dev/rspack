use crate::prt;
use colored::Colorize;
use std::{env, fmt::Display, rc::Rc};

pub fn clear_console() {
  // clear terminal
  print!("\x1B[2J");
}

pub fn is_mute() -> bool {
  env::var("MUTE").is_ok()
}

pub enum Feedback {
  Accept,
  Cancel,
}

pub fn user_prompt<T: Display>(msg: T) -> bool {
  prt!("{}", format!("{}\nYou can\n", msg).green());
  prt!("{}", "Type 'a' to accept\nType 'c' to cancel\n".green());

  loop {
    prt!(">");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let input = input.trim();
    if input == "a" {
      break true;
    } else if input == "c" {
      break false;
    } else {
      prt!("Invalid input, please type 'a' or 'c'\n");
    }
  }
}
