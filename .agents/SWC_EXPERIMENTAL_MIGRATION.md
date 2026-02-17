# SWC Experimental Migration Guide

This skill provides guidelines and common patterns for migrating from `swc_core` (legacy) to `swc-experimental` (new flat AST storage).

## Core Concepts

In `swc-experimental`, AST nodes are typically lightweight identifiers (IDs) or small structs. The actual data is stored in a central `Ast` context. This means most property access and metadata retrieval require passing a reference to the `Ast` instance.

## Migration Patterns

### 1. Property Access (Pass the `ast` context)

Properties that were previously direct fields or parameterless methods now require the `&Ast` context.

**Legacy:**

```rust
let left = bin_expr.left;
let op = bin_expr.op;
let callee = call_expr.callee;
```

**Experimental:**

```rust
let left = bin_expr.left(ast);
let op = bin_expr.op(ast);
let callee = call_expr.callee(ast);
```

### 2. Spans and Spanned Trait

The `Spanned` trait now requires the AST context for most nodes.

**Legacy:**

```rust
let span = expr.span();
```

**Experimental:**

```rust
let span = expr.span(ast);
```

_Note: Some simple structs like `Ident` may have a direct `.span` field._

### 3. Strings and Literals

To get the actual string content from a literal or identifier, you must use the `Ast` storage.

**Legacy:**

```rust
let val = lit_str.value; // Atom or String
```

**Experimental:**

```rust
let val = ast.get_wtf8(lit_str.value(ast)); // Returns Utf8Ref
let string_val = val.to_string_lossy();
```

### 4. Ownership and Boxing

Since nodes are now lightweight IDs, they should be passed by value. `Box<T>` is rarely needed for AST structure.

**Legacy:**

```rust
fn scan_dependencies(expr: &Expr) { ... }
let boxed_expr = Box::new(expr);
```

**Experimental:**

```rust
fn scan_dependencies(expr: Expr) { ... } // Passed by value
// Box::new is usually removed when constructing nested structures
```

### 5. Common Method Replacements

| Legacy                      | Experimental                         |
| :-------------------------- | :----------------------------------- |
| `swc_core::ecma::ast`       | `swc_experimental_ecma_ast`          |
| `swc_core::common::Spanned` | `swc_experimental_ecma_ast::Spanned` |
| `.span()`                   | `.span(ast)`                         |
| `.left` / `.right`          | `.left(ast)` / `.right(ast)`         |
| `.obj` / `.prop`            | `.obj(ast)` / `.prop(ast)`           |
| `.callee`                   | `.callee(ast)`                       |
| `Box<Expr>`                 | `Expr`                               |
| `&Ident`                    | `Ident` (Pass by value)              |

### 6. Parser and Initialization

The parser API has changed to support the new AST storage.

**Legacy:**

```rust
let (mut ast, tokens) = javascript_compiler.parse_with_lexer(
  &source_string,
  parser_lexer,
  is_module,
  Some(comments),
  true,
)?;
```

**Experimental:**

```rust
let parse_lexer = Capturing::new(parser_lexer);
let parser = Parser::new_from(parse_lexer);
let mut ret = parser.parse_program()?;
let (root, ast, tokens) = (ret.root, ret.ast, ret.input.take());
```

_Note: The `ParserResult` contains both the `ast` (storage) and the `root` (the actual entry point node)._

## Example Refactoring

**Before:**

```rust
pub fn is_logic_op(op: BinaryOp) -> bool {
  matches!(op, BinaryOp::LogicalAnd | BinaryOp::LogicalOr)
}

pub fn expression_logic_operator(scanner: &mut JavascriptParser, expr: &BinExpr) -> Option<bool> {
  if is_logic_op(expr.op) {
    let param = scanner.evaluate_expression(&expr.left);
    // ...
    Some(true)
  } else {
    None
  }
}
```

**After:**

```rust
pub fn expression_logic_operator(scanner: &mut JavascriptParser, expr: BinExpr) -> Option<bool> {
  let ast = &scanner.ast;
  if is_logic_op(expr.op(ast)) {
    let param = scanner.evaluate_expression(expr.left(ast));
    // ...
    Some(true)
  } else {
    None
  }
}
```

## Tips

- Always check if a method expects `&ast` or `ast`.
- If you see a compilation error about `Box`, try removing it; nodes are likely IDs now.
- `Utf8Ref` is a common return type for string access; use `.to_string_lossy()` if you need a `String`.
