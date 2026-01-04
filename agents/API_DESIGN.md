# API Design Principles

API design principles for Rspack, focusing on webpack compatibility, versioning, and maintainability.

## Core Principles

### 1. Webpack Compatibility First

Prioritize compatibility with webpack's API for seamless migration.

**Compatibility Levels:**

- **Full**: Works identically to webpack
- **Partial**: Works with differences/limitations
- **Not Compatible**: Not supported (documented with alternatives)

**Goals:**

- API compatibility: Match webpack's public API
- Plugin compatibility: Support webpack plugins (when possible)
- Loader compatibility: Support webpack loaders
- Configuration compatibility: Accept webpack config format

### 2. Performance Over Compatibility (When Necessary)

When compatibility conflicts with performance, prioritize performance but provide alternatives.

**Examples:**

- Native Rust implementations over JavaScript
- Optimized algorithms over exact webpack behavior
- Parallel processing over sequential execution

### 3. Type Safety

APIs should be type-safe with good TypeScript support.

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

```typescript
interface RspackOptions {
  entry?: EntryNormalized;
  output?: OutputNormalized;
  module?: ModuleOptions;
  plugins?: RspackPluginInstance[];
}
```

**Principles:**

- Optional properties for all configuration
- Sensible defaults
- Nested objects for related options
- Support both object and array formats

### Plugin API

```typescript
interface Plugin {
  apply(compiler: Compiler): void;
  name?: string;
}
```

**Principles:**

- Simple, single-method interface
- Optional name for identification
- Access through hooks
- Support both class and function plugins

### Loader API

```typescript
function loader(
  this: LoaderContext,
  content: string | Buffer,
  sourceMap?: SourceMap,
): string | Buffer | void;
```

**Principles:**

- Simple function signature
- Context via `this`
- Support async loaders
- Chainable (output → next loader input)

### Hook API

```typescript
compiler.hooks.done.tap('PluginName', (stats) => {
  // Handle completion
});
```

**Principles:**

- Consistent naming (camelCase)
- Clear hook names
- Type-safe signatures
- Support sync and async hooks

## Versioning Strategy

### Semantic Versioning

Follow [Semantic Versioning](https://semver.org/):

- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

### Breaking Changes

Only in major versions, carefully planned.

**Types:**

- Removing/renaming public APIs
- Changing function signatures
- Changing default behavior
- Removing configuration options

**Migration Path:**

- Deprecation warnings before removal
- Migration guides for major versions
- Compatibility layers when possible

### Deprecation Policy

**Process:**

1. Mark with `@deprecated` tag
2. Add deprecation notice in docs
3. Log deprecation warning at runtime
4. Remove in next major version

**Timeline:**

- At least one minor version with warning
- Removal in next major version
- Migration guide provided

## Backward Compatibility

### Configuration Compatibility

- Old formats continue to work
- New options are additive (optional)
- Deprecated options show warnings but work
- Migration tools for major changes

### Plugin Compatibility

**Levels:**

- **Full**: Works without modification
- **Compatibility Layer**: Works with adapter
- **Not Compatible**: Needs rewrite (documented)

**Checks:**

- Test against popular webpack plugins
- Document compatibility status
- Provide alternatives for incompatible plugins

### API Compatibility

**Breaking Changes:**

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

- **Build Errors**: Module build failures, resolution errors, compilation errors
- **Configuration Errors**: Invalid config, missing options, type mismatches
- **Runtime Errors**: Plugin errors, loader errors, file system errors

### Error Messages

**Guidelines:**

- Clear, actionable messages
- Include context (file path, line number)
- Provide suggestions for fixes
- Link to documentation when relevant

## Type Definitions

### Public API Types

- Export types from main entry point
- Use interfaces for object shapes
- Use type aliases for unions/intersections
- Document complex types

### Internal Types

- Use `@internal` tag for internal APIs
- Don't export implementation details
- Use `export type` for type-only exports

## Documentation Standards

### API Documentation

**Required:**

- Description of what API does
- Parameter descriptions with types
- Return value description
- Usage examples
- Related APIs

### Configuration Documentation

**Required:**

- Description of option
- Default value
- Type information
- Example usage
- Related options

## Testing Requirements

### API Testing

- All public APIs should have tests
- Test success and error cases
- Test edge cases and boundaries
- Test compatibility with webpack

### Compatibility Testing

- Test against webpack test suite (where applicable)
- Test popular plugins and loaders
- Test migration scenarios
- Document compatibility status

## Performance Considerations

### API Performance

- Minimize overhead in hot paths
- Use efficient data structures
- Avoid unnecessary allocations
- Profile API calls

### Optimization Opportunities

- Batch operations when possible
- Lazy evaluation for expensive operations
- Caching for repeated operations
- Parallel processing where applicable

## Extension Points

### Plugin Extension API

Plugins can:

- Access compilation state
- Modify compilation process
- Add new assets
- Transform existing assets
- Hook into any compilation stage

### Loader Extension API

Loaders can:

- Transform source code
- Access loader context
- Emit additional files
- Handle errors gracefully

### Custom Hooks

When adding hooks:

- Follow existing naming conventions
- Document hook purpose and timing
- Provide type-safe signatures
- Consider performance implications

## Migration Guidelines

### From Webpack

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
