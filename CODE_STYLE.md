# Code Style Guide

Coding standards and conventions for the Rspack project.

## General Principles

- **Consistency**: Follow existing patterns
- **Clarity**: Write readable code
- **Maintainability**: Consider future developers
- **Performance**: Be mindful of hot paths

## Rust Code Style

### Formatting

- Use `cargo fmt` (config in `rustfmt.toml`)
- **Indentation**: 2 spaces
- **Line length**: Prefer < 100 chars, up to 120 acceptable
- **Edition**: Rust 2024

### Naming Conventions

- **Types**: `PascalCase` (e.g., `BannerPlugin`, `Compilation`)
- **Functions**: `snake_case` (e.g., `process_assets`)
- **Variables**: `snake_case` (e.g., `compilation`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `MY_CONSTANT`)
- **Modules**: `snake_case` (e.g., `rspack_plugin_banner`)

### Code Organization

**Imports:**

```rust
use std::{fmt::Debug, sync::LazyLock};
use regex::Regex;
use rspack_core::{Compilation, Plugin};
use rspack_error::Result;
```

**Module Structure:**

1. External imports
2. Internal imports
3. Type definitions
4. Constants
5. Helper functions
6. Plugin implementation
7. Hook implementations
8. Trait implementations

### Error Handling

- Use `rspack_error::Result<T>` for fallible operations
- Use `?` operator for error propagation
- Provide context in error messages

```rust
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
 let result = some_operation().await?;
 Ok(())
}
```

### Async Code

- Prefer `async/await` over manual futures
- Use `BoxFuture` for trait object methods
- Avoid `block_on` in async contexts

### Static Initialization

- Use `LazyLock` for lazy static initialization

```rust
use std::sync::LazyLock;
static MY_REGEX: LazyLock<Regex> = LazyLock::new(|| {
 Regex::new(r"pattern").expect("invalid regexp")
});
```

### String Handling

- Use `cow_utils::CowUtils` to avoid unnecessary allocations
- Prefer `Cow<str>` over `String` when possible

```rust
use cow_utils::CowUtils;
let result = str.cow_replace("old", "new");
```

### Clippy

- Run `cargo clippy` before committing
- Address all warnings
- Configuration in `clippy.toml`

### Documentation

- Use `///` for public API documentation
- Include examples for complex functions
- Document error conditions
- Use `#[doc(hidden)]` for internal APIs

## TypeScript/JavaScript Code Style

### TS/JS Formatting

- Use Prettier (configured in `biome.jsonc`)
- Use Biome for linting
- **Indentation**: Tabs
- **Semicolons**: Use semicolons
- **Quotes**: Double quotes

### TS/JS Naming Conventions

- **Types/Interfaces**: `PascalCase` (e.g., `RspackOptions`)
- **Classes**: `PascalCase` (e.g., `Compiler`)
- **Functions**: `camelCase` (e.g., `createCompiler`)
- **Variables**: `camelCase` (e.g., `compiler`)
- **Constants**: `SCREAMING_SNAKE_CASE` or `camelCase`
- **Files**: `camelCase.ts` or `PascalCase.ts` (match main export)

### TS/JS Code Organization

**File Header** (for webpack-derived code):

```typescript
/**
 * Modified from https://github.com/webpack/webpack/blob/4b4ca3b/lib
 * MIT Licensed
 */
```

**Imports:**

```typescript
import util from "node:util";
import type { Callback } from "@rspack/lite-tapable";
import { Compiler } from "./Compiler";
```

**Type Definitions:**

- Use `type` for type aliases
- Use `interface` for object shapes
- Prefer `type` for unions/intersections
- Export types explicitly with `export type`

### TS/JS Error Handling

- Use `throw new Error()` for errors
- Provide descriptive messages
- Include context

```typescript
if (isNil(options.context)) {
	throw new Error("options.context is required");
}
```

### TS/JS Async Code

- Use `async/await` over promises
- Handle errors with try/catch
- Use `Promise.all` for parallel operations

### Comments

- Use `//` for single-line
- Use `/* */` for multi-line
- Use JSDoc for public APIs
- Explain "why" not "what"

## Testing

### Rust Testing

- Place in same file with `#[cfg(test)]` module
- Use descriptive names: `test_<what>_<condition>_<expected>`
- Use `assert_eq!`, `assert!`, etc.

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

### JavaScript/TypeScript Testing

- Use descriptive names
- Group with `describe`
- Use `it` or `test` for cases
- Clean up after tests

## File Organization

### Rust Crates

```text
crates/rspack_plugin_xxx/
├── src/
│   ├── lib.rs
│   └── mod.rs
└── Cargo.toml
```

### TypeScript Packages

```text
packages/xxx/
├── src/
│   ├── index.ts
│   └── Compiler.ts
├── dist/
├── package.json
└── tsconfig.json
```

## Performance Considerations

- Avoid unnecessary allocations in hot paths
- Use `Cow<str>` when possible
- Prefer `&str` over `String` when ownership isn't needed
- Use appropriate data structures
- Profile before optimizing

## Common Patterns to Avoid

### Rust

- ❌ Don't use `str::to_lowercase` - use `CowUtils::cow_to_lowercase`
- ❌ Don't use `block_on` in async contexts
- ❌ Don't use `unwrap()` in production code

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
