# Code Style Guide

This document outlines the coding standards and conventions used in the Rspack project. Following these guidelines ensures consistency and makes the codebase easier to understand and maintain.

## General Principles

- **Consistency**: Follow existing patterns in the codebase
- **Clarity**: Write code that is easy to read and understand
- **Maintainability**: Consider future developers who will read and modify your code
- **Performance**: Be mindful of performance implications, especially in hot paths

## Rust Code Style

### Formatting

- Use `cargo fmt` to format Rust code
- Configuration is in `rustfmt.toml`
- **Indentation**: 2 spaces (not tabs)
- **Line length**: Prefer lines under 100 characters, but up to 120 is acceptable
- **Edition**: Rust 2024 edition

### Rust Naming Conventions

- **Types**: `PascalCase` (e.g., `BannerPlugin`, `Compilation`)
- **Functions**: `snake_case` (e.g., `process_assets`, `wrap_comment`)
- **Variables**: `snake_case` (e.g., `compilation`, `chunk`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `TRIALING_WHITESPACE`)
- **Modules**: `snake_case` (e.g., `rspack_plugin_banner`)

### Code Organization

#### Rust Imports

- Group imports by: std, external crates, internal crates
- Use `use` statements with granular imports
- Prefer explicit imports over wildcards
- Example:

```rust
use std::{
  fmt::{self, Debug},
  sync::LazyLock,
};

use regex::Regex;
use rspack_core::{
  Chunk, Compilation, CompilationProcessAssets, Filename, Logger, PathData, Plugin,
  rspack_sources::{BoxSource, ConcatSource, RawStringSource, SourceExt},
  to_comment,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
```

#### Module Structure

```rust
// 1. External imports
use std::...;

// 2. Internal imports
use rspack_core::...;

// 3. Type definitions
#[derive(Debug)]
pub struct MyPlugin {
  config: MyPluginOptions,
}

// 4. Constants
static MY_CONSTANT: LazyLock<Regex> = LazyLock::new(|| ...);

// 5. Helper functions
fn helper_function() -> bool {
  // ...
}

// 6. Plugin implementation
#[plugin]
#[derive(Debug)]
pub struct MyPlugin {
  config: MyPluginOptions,
}

impl MyPlugin {
  pub fn new(config: MyPluginOptions) -> Self {
    Self::new_inner(config)
  }
}

// 7. Hook implementations
#[plugin_hook(...)]
async fn my_hook(&self, compilation: &mut Compilation) -> Result<()> {
  // ...
}

// 8. Trait implementations
impl Plugin for MyPlugin {
  fn name(&self) -> &'static str {
    "rspack.MyPlugin"
  }

  fn apply(&self, ctx: &mut ApplyContext<'_>) -> Result<()> {
    // ...
  }
}
```

### Rust Error Handling

- Use `rspack_error::Result<T>` for fallible operations
- Use `?` operator for error propagation
- Provide context in error messages
- Use `Result::map_err` to add context when needed

```rust
use rspack_error::Result;

async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  let result = some_operation().await?;
  Ok(())
}
```

### Rust Async Code

- Prefer `async/await` over manual futures
- Use `BoxFuture` for trait object methods that return futures
- Avoid `block_on` in async contexts (use proper async patterns)

```rust
use futures::future::BoxFuture;

pub type MyAsyncFn = Box<dyn for<'a> Fn(MyCtx<'a>) -> BoxFuture<'a, Result<String>> + Sync + Send>;
```

### Static Initialization

- Use `LazyLock` for lazy static initialization
- Prefer `LazyLock` over `lazy_static` or `once_cell::sync::Lazy`

```rust
use std::sync::LazyLock;

static MY_REGEX: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(r"pattern").expect("invalid regexp")
});
```

### String Handling

- Use `cow_utils::CowUtils` for string operations to avoid unnecessary allocations
- Prefer `Cow<str>` over `String` when possible
- Use `cow_replace`, `cow_to_lowercase`, etc. instead of `str::replace`, `str::to_lowercase`

```rust
use cow_utils::CowUtils;

let result = str.cow_replace("old", "new");
```

### Clippy

- Run `cargo clippy` before committing
- Configuration is in `clippy.toml`
- Address all clippy warnings
- Some methods are disallowed (see `clippy.toml` for details)

### Documentation

- Use `///` for public API documentation
- Include examples for complex functions
- Document error conditions
- Use `#[doc(hidden)]` for internal APIs

```rust
/// Processes assets and adds banner comments.
///
/// # Arguments
///
/// * `compilation` - The compilation context
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error if processing fails.
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  // ...
}
```

## TypeScript/JavaScript Code Style

### TS/JS Formatting

- Use Prettier for formatting (configured in `biome.jsonc`)
- Use Biome for linting
- **Indentation**: Tabs (as configured in the project)
- **Semicolons**: Use semicolons
- **Quotes**: Use double quotes for strings

### TS/JS Naming Conventions

- **Types/Interfaces**: `PascalCase` (e.g., `RspackOptions`, `Compiler`)
- **Classes**: `PascalCase` (e.g., `Compiler`, `Compilation`)
- **Functions**: `camelCase` (e.g., `createCompiler`, `processAssets`)
- **Variables**: `camelCase` (e.g., `compiler`, `options`)
- **Constants**: `SCREAMING_SNAKE_CASE` or `camelCase` (depending on context)
- **Files**: `camelCase.ts` or `PascalCase.ts` (match the main export)

### TS/JS Code Organization

#### File Header

Include copyright notice for code derived from webpack:

```typescript
/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */
```

#### TS/JS Imports

- Group imports: external packages, internal modules
- Use explicit imports
- Organize imports logically

```typescript
import util from "node:util";
import type { Callback } from "@rspack/lite-tapable";
import { Compiler } from "./Compiler";
import {
  applyRspackOptionsBaseDefaults,
  applyRspackOptionsDefaults,
  getNormalizedRspackOptions,
  type RspackOptions,
  type RspackPluginFunction
} from "./config";
```

#### Type Definitions

- Use `type` for type aliases
- Use `interface` for object shapes
- Prefer `type` over `interface` for unions and intersections
- Export types explicitly with `export type`

```typescript
export interface RspackOptions {
  entry?: EntryNormalized;
  output?: OutputNormalized;
}

export type RspackPluginFunction = (compiler: Compiler) => void;
```

### TS/JS Error Handling

- Use `throw new Error()` for errors
- Provide descriptive error messages
- Include context in error messages

```typescript
if (isNil(options.context)) {
  throw new Error("options.context is required");
}
```

### TS/JS Async Code

- Use `async/await` over promises
- Handle errors with try/catch
- Use `Promise.all` for parallel operations

```typescript
async function processAssets(compilation: Compilation): Promise<void> {
  try {
    await someAsyncOperation();
  } catch (error) {
    throw new Error(`Failed to process assets: ${error}`);
  }
}
```

### Comments

- Use `//` for single-line comments
- Use `/* */` for multi-line comments
- Use JSDoc comments for public APIs
- Explain "why" not "what"

```typescript
/**
 * Creates a new compiler instance with the given options.
 *
 * @param options - The rspack configuration options
 * @returns A new Compiler instance
 */
function createCompiler(options: RspackOptions): Compiler {
  // ...
}
```

## Testing

### Rust Tests

- Place tests in the same file with `#[cfg(test)]` module
- Use descriptive test names: `test_<what>_<condition>_<expected_result>`
- Use `assert_eq!`, `assert!`, etc.
- Test both success and error cases

```rust
#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_wrap_comment_single_line() {
    let result = wrap_comment("test");
    assert_eq!(result, "/*! test */");
  }
}
```

### JavaScript/TypeScript Tests

- Use descriptive test names
- Group related tests with `describe`
- Use `it` or `test` for individual test cases
- Clean up after tests

```typescript
describe("Compiler", () => {
  it("should create a compiler with valid options", () => {
    const compiler = createCompiler({ context: "/path" });
    expect(compiler).toBeInstanceOf(Compiler);
  });
});
```

## File Organization

### Rust Crates

```text
crates/rspack_plugin_xxx/
├── src/
│   ├── lib.rs          # Main module, exports
│   └── mod.rs          # Additional modules (if needed)
└── Cargo.toml          # Dependencies
```

### TypeScript Packages

```text
packages/xxx/
├── src/
│   ├── index.ts        # Main entry point
│   ├── Compiler.ts     # Individual modules
│   └── types.ts        # Type definitions
├── dist/               # Compiled output
├── package.json
└── tsconfig.json
```

## Performance Considerations

- Avoid unnecessary allocations in hot paths
- Use `Cow<str>` for string operations when possible
- Prefer `&str` over `String` when ownership isn't needed
- Use appropriate data structures (HashMap vs Vec)
- Profile before optimizing

## Common Patterns to Avoid

### Rust

- ❌ Don't use `str::to_lowercase` - use `CowUtils::cow_to_lowercase`
- ❌ Don't use `block_on` in async contexts
- ❌ Don't use `std::mem::forget` with `future::scope`
- ❌ Don't use `unwrap()` in production code (use `?` or proper error handling)

### TypeScript

- ❌ Don't use `any` type (use `unknown` or proper types)
- ❌ Don't ignore TypeScript errors
- ❌ Don't use `@ts-ignore` without good reason

## Tools

- **Rust formatting**: `cargo fmt`
- **Rust linting**: `cargo clippy`
- **TypeScript formatting**: Prettier (via Biome)
- **TypeScript linting**: Biome
- **Type checking**: TypeScript compiler

## Resources

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [TypeScript Handbook](https://www.typescriptlang.org/docs/handbook/intro.html)
- [Biome Documentation](https://biomejs.dev/)
