# Deep Dive: SWC Parsing & Dependency Scanning Pipeline

**Files**:
- `crates/rspack_plugin_javascript/src/parser_and_generator/mod.rs` — Parse entry point
- `crates/rspack_plugin_javascript/src/visitors/dependency/parser/mod.rs` — JavascriptParser, walk_program
- `crates/rspack_plugin_javascript/src/visitors/dependency/parser/walk.rs` — Main AST walk (1700+ lines)
- `crates/rspack_plugin_javascript/src/visitors/dependency/parser/walk_pre.rs` — Pre-walk
- `crates/rspack_plugin_javascript/src/visitors/dependency/parser/walk_block_pre.rs` — Block pre-walk
- `crates/rspack_plugin_javascript/src/visitors/dependency/parser/walk_module_pre.rs` — Module pre-walk

---

## The 7-Pass Pipeline Per Module

For each JavaScript module, the following pipeline runs:

### Phase A: SWC Lexer + Parser (in `parse` method)

```
Source String → Lexer (tokenize) → Parser (AST) → Tokens
```

**Cost**: Proportional to source size. SWC is highly optimized but still:
- Allocates AST nodes on heap (Box<Expr>, Box<Stmt>, etc.)
- Processes every character in the source
- Creates a comments map
- Creates a token list (used for semicolon detection)

### Phase B: Three SWC Transforms

```rust
ast.transform(|program, context| {
    program.visit_mut_with(&mut paren_remover(Some(&comments)));      // Pass 1
    program.visit_mut_with(&mut resolver(unresolved_mark, top_level_mark, false)); // Pass 2
    program.visit_with(&mut semicolon::InsertedSemicolons { ... });   // Pass 3
});
```

**Pass 1 — `paren_remover`**: Removes unnecessary parentheses from the AST. Walks every expression node.

**Pass 2 — `resolver`**: Resolves variable scopes by applying SWC marks. This is a full traversal that:
- Maintains a scope stack
- Assigns `SyntaxContext` to identifiers
- Handles hoisting, closures, block scoping

**Pass 3 — `InsertedSemicolons`**: Walks the token list to detect ASI (Automatic Semicolon Insertion) positions. Used later for code generation.

### Phase C: Four-Pass Dependency Scanning

```rust
// In walk_program():
self.module_pre_walk_module_items(&m.body);    // Pass 4: Module pre-walk
self.pre_walk_module_items(&m.body);           // Pass 5: Pre-walk (var hoisting)
self.block_pre_walk_module_items(&m.body);     // Pass 6: Block pre-walk (let/const)
self.walk_module_items(&m.body);               // Pass 7: Full walk (dependency discovery)
```

**Pass 4 — `module_pre_walk`**: Processes import/export declarations at the module level. Creates ESM import/export dependencies.

**Pass 5 — `pre_walk`**: Hoists `var` declarations. Defines variables before the main walk processes expressions.

**Pass 6 — `block_pre_walk`**: Processes `let`, `const`, and `class` declarations. These are block-scoped so processed separately from `var`.

**Pass 7 — `walk`**: The main full AST traversal. For every expression, statement, and declaration:
- Calls plugin hooks (35 plugins × dynamic dispatch)
- Evaluates expressions for constant folding
- Creates dependencies for require(), import(), import.meta, etc.
- Tracks scope information

---

## Total: 7 Passes Over the AST

For a 1000-line JavaScript file with ~2000 AST nodes:
- **Passes 1-3**: ~2000 × 3 = 6,000 node visits (SWC transforms)
- **Passes 4-7**: ~2000 × 4 = 8,000 node visits (dependency scanning)
- **Total**: ~14,000 node visits per module

At 10K modules: **140 million node visits**

---

## Plugin Drive Overhead in Pass 7

Every node visit in the main walk goes through the plugin system:

```rust
fn walk_statement(&mut self, statement: Statement) {
    self.enter_statement(&statement, |parser, _| {
        parser.plugin_drive.clone().statement(parser, statement).unwrap_or_default()
    }, |parser, _| match statement { ... });
}
```

The `enter_statement` pattern:
```rust
fn enter_statement<T, B, W>(&mut self, statement: &T, before: B, walk: W)
where B: FnOnce(&mut Self, &T) -> bool,
      W: FnOnce(&mut Self, &T) {
    self.statement_path.push(...);
    if !before(self, statement) {
        walk(self, statement);
    }
    self.statement_path.pop();
}
```

For each statement:
1. Push to statement_path (Vec push)
2. Clone plugin drive (Rc::clone — atomic increment)
3. Iterate all 35 plugins for `statement` hook
4. If no plugin handled it, walk the statement
5. Pop from statement_path

For each expression (inside walk):
```rust
pub(crate) fn walk_expression(&mut self, expr: &Expr) {
    match expr {
        Expr::Array(expr) => self.walk_array_expression(expr),
        Expr::Arrow(expr) => self.walk_arrow_function_expression(expr),
        Expr::Assign(expr) => self.walk_assignment_expression(expr),
        // ... 30+ match arms
    }
}
```

**Key cost**: The plugin dispatch for expression hooks is the most frequent:
```rust
// Called for EVERY expression
pub fn expression(&self, parser: &mut JavascriptParser, expr: &Expr) -> Option<bool> {
    for plugin in &self.plugins {
        if let Some(result) = plugin.expression(parser, expr) {
            return Some(result);
        }
    }
    None
}
```

At ~2000 expressions per module × 35 plugins = **70,000 virtual dispatches per module**.
At 10K modules: **700 million virtual dispatches**.

---

## Optimization Opportunities

### 1. Merge Passes 1-3 (SWC transforms)

The three SWC transforms are independent visitors. They could be combined into a single traversal:

```rust
// Current: 3 separate passes
program.visit_mut_with(&mut paren_remover(&comments));
program.visit_mut_with(&mut resolver(unresolved_mark, top_level_mark, false));
program.visit_with(&mut InsertedSemicolons { ... });

// Proposed: single combined pass
struct CombinedTransform<'a> {
    paren_remover: ParenRemover<'a>,
    resolver: Resolver,
    semicolons: InsertedSemicolons<'a>,
}

impl VisitMut for CombinedTransform<'_> {
    fn visit_mut_expr(&mut self, e: &mut Expr) {
        self.paren_remover.visit_mut_expr(e);
        self.resolver.visit_mut_expr(e);
        // semicolons only needs token info, doesn't visit AST
    }
}
```

**Challenge**: `paren_remover` modifies the AST structure which `resolver` depends on. Need to verify ordering constraints.

**Savings**: ~2000 node visits × 2 eliminated passes = 4000 fewer visits per module.

### 2. Merge Passes 4-6 (pre-walks)

The three pre-walk passes process different kinds of declarations:

```
module_pre_walk: import/export declarations
pre_walk: var declarations (hoisted)
block_pre_walk: let/const/class declarations (block-scoped)
```

These could be combined into a single pass that handles all declaration types:

```rust
fn unified_pre_walk_module_items(&mut self, statements: &[ModuleItem]) {
    for statement in statements {
        match statement {
            ModuleItem::ModuleDecl(decl) => self.pre_walk_module_decl(decl),
            ModuleItem::Stmt(stmt) => {
                self.pre_walk_var_declarations(stmt);
                self.pre_walk_block_declarations(stmt);
            }
        }
    }
}
```

**Challenge**: The ordering of these pre-walks matters for correct scoping. `var` hoisting must happen before `let`/`const` processing.

**Savings**: ~2000 × 2 = 4000 fewer node visits per module.

### 3. Plugin Interest Bitmasks

Pre-compute which plugins implement each hook:

```rust
struct PluginDriveCache {
    has_statement_plugins: bool,           // Any plugin implements statement()?
    has_expression_plugins: bool,          // Any plugin implements expression()?
    statement_plugin_indices: Vec<usize>,  // Which plugins implement statement()
    expression_plugin_indices: Vec<usize>, // Which plugins implement expression()
    // etc.
}
```

Skip the plugin iteration entirely when no plugin cares:

```rust
fn expression(&self, parser: &mut JavascriptParser, expr: &Expr) -> Option<bool> {
    if !self.cache.has_expression_plugins {
        return None;  // FAST PATH: skip all 35 plugins
    }
    for &idx in &self.cache.expression_plugin_indices {
        if let Some(result) = self.plugins[idx].expression(parser, expr) {
            return Some(result);
        }
    }
    None
}
```

**Savings**: At typical configurations, most hooks have 1-3 interested plugins out of 35. This eliminates 90%+ of virtual dispatch calls.

### 4. Fast-Path for Import-Only Modules

Many modules in a React app are simple "barrel" files:
```js
export { default as Button } from './Button';
export { default as Input } from './Input';
// ...
```

These only have import/export at the top level. The full walk (pass 7) is unnecessary — all dependencies are discovered in the module pre-walk (pass 4).

**Detection**: After pass 4, if all statements are import/export declarations and there are no expressions, skip passes 5-7.

**Savings**: Eliminates the most expensive pass for the simplest modules.

### 5. Lazy Token Collection

Tokens are collected during parsing but only used for semicolon detection:

```rust
let (mut ast, tokens) = javascript_compiler.parse_with_lexer(
    &source_string, parser_lexer, ..., true  // with_tokens = true
);
```

If semicolons could be detected during parsing (by the lexer), the token collection could be eliminated.

**Savings**: Reduces memory allocation (token list for 1000-line file is significant).

---

## Profiling Data Interpretation

From the 1000-module benchmark:
- `build module graph`: 748ms
- This includes: file I/O (~100ms) + SWC parse+transform (~300ms) + dependency scan (~200ms) + task loop overhead (~150ms)

If we save 30% on SWC transforms (merging passes) and 20% on dependency scanning (plugin interest + fast-path):
- SWC: 300ms × 0.7 = 210ms (save 90ms)
- Scan: 200ms × 0.8 = 160ms (save 40ms)
- **Total savings: ~130ms at 1000 modules, projected ~1.3s at 10K modules**

---

## Summary

| Optimization | Per-Module Saving | At 10K Modules | Effort |
|-------------|-------------------|----------------|--------|
| Merge SWC passes 1-3 | ~30% of transform | ~1s | Medium |
| Merge pre-walks 4-6 | ~15% of scan | ~0.3s | Medium |
| Plugin interest bitmasks | ~20% of scan | ~0.5s | Medium |
| Fast-path for import-only modules | 50-80% for barrels | Varies | Low |
| Lazy token collection | ~5% of parse | ~0.2s | High |
