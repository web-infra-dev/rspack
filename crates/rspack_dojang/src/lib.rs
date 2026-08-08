// Author Jaebum Lee (https://modoocode.com)
//
// Licensed under MIT license.

// The parser and evaluator validate their internal stacks before accessing them.
#![allow(clippy::unwrap_used)]

//! Dojang, a EJS inspired HTML template engine.
//!
//! Dojang is a html template engine, which aims to be an drop-in replacement for the EJS html
//! template engine. It follows the syntax of the EJS and follows the javascript syntax within the
//! template. Dojang also provides the strong integration with Rust.
//!
//! Dojang is pronounced as doe-jang, and it means a "stamp" in Korean (도장).
//!
//! # Quick Start
//!
//! Every template files are parsed once during the construction of the `Dojang` object. `Dojang`
//! owns and manages every template file provided. To provide the template files to `Dojang`, you
//! can either specify the directory of the template files using `load` or you can
//! manually provide the template as string using `add` method.
//!
//! ```
//! use rspack_dojang::dojang::Dojang;
//!
//! fn main() {
//!   let mut dj = Dojang::new();
//!
//!   // Load files as template under /template/files/path.
//!   dj.load("/template/files/path");
//!
//!   // Register "template_file.html" which have "<%= data %>" as a template.
//!   dj.add("template_file.html".to_string(), "<%= data %>".to_string());
//!
//!   // "hello" will be rendered.
//!   assert_eq!(dj.render("template_file.html", serde_json::json!({"data" : "hello"})).unwrap(), "hello");
//! }
//! ```
//!
//! Those template can be rendered by the either the name of the template file (when registered by
//! `load`) or the name provided (when reigstered by `add`).
//!
//! To render the template, use `render` method of Dojang. You should provide the name of the
//! template to render and the context to render. Context is simply a json data. Use serde_json to
//! either construct json data or parse the json string. You can even provide your custom struct if
//! you define `Serialize` trait of serde.
//!
//! # Use Custom Functions
//!
//! One powerful feature is that you can use Rust functions inside of the template file. For
//! example,
//!
//! ```
//! use rspack_dojang::dojang::Dojang;
//!
//! fn add(a: i64, b: i64) -> i64 {
//!   a + b
//! }
//!
//! fn main() {
//!   let mut dj = Dojang::new();
//!
//!   // Register our add function as "add_func". You can call add_func inside of the template now.
//!   dj.add_function_2("add_func".to_string(), add);
//!
//!   dj.add(
//!     "template_file.html".to_string(),
//!     "<%= add_func(1, 2) %>".to_string(),
//!   );
//!
//!   // "3" will be rendered.
//!   assert_eq!(
//!     dj.render("template_file.html", serde_json::json!({}))
//!       .unwrap(),
//!     "3"
//!   );
//! }
//! ```
//!
//! You have to choose add_function_* based on the number of parameters that the function has. We
//! provde add_function_1 to add_function_4.
//!
//! Also, the type of the function parameter must be non-reference, and should be one of i64, f64,
//! u64, String, bool.
//!
//! # Supported Javascript syntax.
//!
//! Most of the basic Javascript syntax is supported. Still, any complex javascript usage is not
//! recommended.
//!
//! * `if`, `else`, `for .. in`, `else` is supported.
//!
//! ```ejs
//! <% for i in vec1 { %>
//! <%   if i == 1 { %>
//!   <p>i is one</p>
//! <%   } else { <%>
//!   <p><%= i + 3 %></p>
//! <%   } %>
//! <% } %>
//! ```
//!
//! If `vec1` is provided as `[1,2,3]`, then the above will render
//!
//! ```html
//! <p>i is one</p>
//! <p>5</p>
//! <p>6</p>
//! ```
//!
//! * `break` and `continue` keywords are supported.
//! * Property accessing using `.` and `[..]` are supported
//! * Calling registered function is supported.
//! * Local variables can be introduced but it does not have a scope (every introduced local
//!   variables are global). It should be used with care. For example,
//!
//! ```ejs
//! <%# Sets the value of a as 3. %>
//! <%= a = 3 %>
//! <% a %><%# This will print 3 %>
//! ```
mod context;
mod default_functions;
pub mod dojang;
mod eval;
mod exec;
mod expr;
pub mod func_helper;
pub use crate::{
  context::{Context, FunctionContainer},
  dojang::Dojang,
  expr::Operand,
};
