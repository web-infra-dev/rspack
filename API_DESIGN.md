# API Design Principles

This document outlines the API design principles and guidelines for Rspack, focusing on webpack compatibility, versioning, and maintainability.

## Core Principles

### 1. Webpack Compatibility First

Rspack prioritizes compatibility with webpack's API to enable seamless migration and ecosystem reuse.

**Compatibility Goals:**

- **API Compatibility**: Match webpack's public API surface
- **Plugin Compatibility**: Support webpack plugins without modification (when possible)
- **Loader Compatibility**: Support webpack loaders without modification
- **Configuration Compatibility**: Accept webpack configuration format

**Compatibility Levels:**

- **Full Compatibility**: Feature works identically to webpack
- **Partial Compatibility**: Feature works with some differences or limitations
- **Not Compatible**: Feature is not supported (documented with alternatives)

### 2. Performance Over Compatibility (When Necessary)

When compatibility conflicts with performance, performance takes precedence, but alternatives are provided.

**Examples:**

- Native Rust implementations over JavaScript equivalents
- Optimized algorithms over exact webpack behavior
- Parallel processing over sequential execution

### 3. Type Safety

APIs should be type-safe and provide good TypeScript support.

**Guidelines:**

- Use TypeScript for all public APIs
- Provide comprehensive type definitions
- Use discriminated unions for options
- Avoid `any` types in public APIs

### 4. Developer Experience

APIs should be intuitive, well-documented, and provide helpful error messages.

**Guidelines:**

- Clear, descriptive names
- Comprehensive documentation with examples
- Helpful error messages with suggestions
- Good IDE support (autocomplete, type hints)

## API Design Patterns

### Configuration API

**Pattern:**

```typescript
interface RspackOptions {
	entry?: EntryNormalized;
	output?: OutputNormalized;
	module?: ModuleOptions;
	plugins?: RspackPluginInstance[];
	// ...
}
```

**Principles:**

- Use optional properties for all configuration
- Provide sensible defaults
- Use nested objects for related options
- Support both object and array formats where appropriate

### Plugin API

**Pattern:**

```typescript
interface Plugin {
	apply(compiler: Compiler): void;
	name?: string;
}
```

**Principles:**

- Simple, single-method interface
- Optional name for identification
- Access to compiler/compilation through hooks
- Support both class and function plugins

### Loader API

**Pattern:**

```typescript
function loader(
	this: LoaderContext,
	content: string | Buffer,
	sourceMap?: SourceMap
): string | Buffer | void;
```

**Principles:**

- Simple function signature
- Context provided via `this`
- Support async loaders
- Chainable (output becomes next loader's input)

### Hook API

**Pattern:**

```typescript
compiler.hooks.done.tap("PluginName", stats => {
	// Handle completion
});
```

**Principles:**

- Consistent naming (camelCase)
- Clear hook names describing when they're called
- Type-safe hook signatures
- Support sync and async hooks

## Versioning Strategy

### Semantic Versioning

Rspack follows [Semantic Versioning](https://semver.org/):

- **MAJOR**: Breaking changes (incompatible API changes)
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

### Breaking Changes

Breaking changes are only introduced in major versions and are carefully planned.

**Types of Breaking Changes:**

- Removing or renaming public APIs
- Changing function signatures
- Changing default behavior
- Removing configuration options

**Migration Path:**

- Deprecation warnings before removal
- Migration guides for major versions
- Compatibility layers when possible

### Deprecation Policy

**Deprecation Process:**

1. Mark API as deprecated with `@deprecated` tag
2. Add deprecation notice in documentation
3. Log deprecation warning at runtime
4. Remove in next major version

**Deprecation Timeline:**

- At least one minor version with deprecation warning
- Removal in next major version
- Migration guide provided

## Backward Compatibility

### Configuration Compatibility

**Principles:**

- Old configuration formats continue to work
- New options are additive (optional)
- Deprecated options show warnings but still work
- Migration tools provided for major changes

**Example:**

```typescript
// Old format (still supported)
entry: "./src/index.js";

// New format (preferred)
entry: {
	main: "./src/index.js";
}
```

### Plugin Compatibility

**Compatibility Levels:**

- **Full Compatibility**: Plugin works without modification
- **Compatibility Layer**: Plugin works with adapter
- **Not Compatible**: Plugin needs rewrite (documented)

**Compatibility Checks:**

- Test against popular webpack plugins
- Document compatibility status
- Provide alternatives for incompatible plugins

### API Compatibility

**Breaking Change Examples:**

- Changing return types
- Removing methods
- Changing method signatures
- Changing behavior

**Non-Breaking Changes:**

- Adding new methods
- Adding optional parameters
- Adding new configuration options
- Performance improvements

## Error Handling

### Error Types

**Build Errors:**

- Module build failures
- Resolution errors
- Compilation errors

**Configuration Errors:**

- Invalid configuration
- Missing required options
- Type mismatches

**Runtime Errors:**

- Plugin errors
- Loader errors
- File system errors

### Error Messages

**Guidelines:**

- Clear, actionable error messages
- Include context (file path, line number)
- Provide suggestions for fixes
- Link to documentation when relevant

**Example:**

```typescript
throw new Error(
	`Module not found: Can't resolve './missing-module' in '/path/to/file.js'\n` +
		`Did you mean './existing-module'?`
);
```

## Type Definitions

### Public API Types

All public APIs should have comprehensive TypeScript types.

**Guidelines:**

- Export types from main entry point
- Use interfaces for object shapes
- Use type aliases for unions/intersections
- Document complex types

**Example:**

```typescript
export interface RspackOptions {
	/**
	 * The entry point(s) of the compilation.
	 * @default './src/index.js'
	 */
	entry?: EntryNormalized;

	/**
	 * Options affecting the output of the compilation.
	 */
	output?: OutputNormalized;
}
```

### Internal Types

Internal types should not be exported unless necessary.

**Guidelines:**

- Use `@internal` tag for internal APIs
- Don't export implementation details=
- Use `export type` for type-only exports

## Documentation Standards

### API Documentation

**Required Elements:**

- Description of what the API does
- Parameter descriptions with types
- Return value description
- Usage examples
- Related APIs

**Example:**

````typescript
/**
 * Creates a new Rspack compiler instance.
 *
 * @param options - Configuration options for the compiler
 * @returns A new Compiler instance
 *
 * @example
 * ```ts
 * const compiler = rspack({
 *   entry: './src/index.js',
 *   output: { path: './dist' }
 * });
 * ```
 */
function rspack(options: RspackOptions): Compiler;
````

### Configuration Documentation

**Required Elements:**

- Description of the option
- Default value
- Type information
- Example usage
- Related options

## Testing Requirements

### API Testing

**Coverage:**

- All public APIs should have tests
- Test both success and error cases
- Test edge cases and boundary conditions
- Test compatibility with webpack

### Compatibility Testing

**Testing:**

- Test against webpack test suite (where applicable)
- Test popular plugins and loaders
- Test migration scenarios
- Document compatibility status

## Performance Considerations

### API Performance

**Guidelines:**

- Minimize overhead in hot paths
- Use efficient data structures
- Avoid unnecessary allocations
- Profile API calls

### Optimization Opportunities

**Areas:**

- Batch operations when possible
- Lazy evaluation for expensive operations
- Caching for repeated operations
- Parallel processing where applicable

## Extension Points

### Plugin Extension API

Plugins should be able to:

- Access compilation state
- Modify compilation process
- Add new assets
- Transform existing assets
- Hook into any compilation stage

### Loader Extension API

Loaders should be able to:

- Transform source code
- Access loader context
- Emit additional files
- Handle errors gracefully

### Custom Hooks

When adding new hooks:

- Follow existing naming conventions
- Document hook purpose and timing
- Provide type-safe signatures
- Consider performance implications

## Migration Guidelines

### From Webpack

**Migration Steps:**

1. Install Rspack
2. Update configuration (minimal changes)
3. Test build
4. Address compatibility issues
5. Optimize for Rspack-specific features

**Common Issues:**

- Plugin compatibility
- Loader compatibility
- Configuration differences
- Behavior differences

### Between Rspack Versions

**Migration Steps:**

1. Read migration guide
2. Update dependencies
3. Update configuration (if needed)
4. Update code (if breaking changes)
5. Test thoroughly

## Best Practices

### For API Consumers

- Use TypeScript for type safety
- Read documentation before using APIs
- Handle errors appropriately
- Follow migration guides for upgrades

### For API Designers

- Design for extensibility
- Consider performance implications
- Provide clear error messages
- Document thoroughly
- Maintain backward compatibility

## Resources

- [Webpack API Documentation](https://webpack.js.org/api/)
- [Rspack API Documentation](https://rspack.rs/api/)
- [Semantic Versioning](https://semver.org/)
- [TypeScript Handbook](https://www.typescriptlang.org/docs/handbook/intro.html)
