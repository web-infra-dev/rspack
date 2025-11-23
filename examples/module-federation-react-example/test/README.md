# Module Federation Test Suite

This directory contains a comprehensive test suite for the Module Federation example with SWC macro optimization.

## Test Structure

```
test/
├── unit/                    # Unit tests for optimization logic
│   └── optimization.test.js # Tests for tree shaking, macro processing
├── integration/             # Integration tests for the build process
│   └── module-federation.test.js # Tests for MF setup and optimization
├── e2e/                     # End-to-end tests using Puppeteer
│   └── app-functionality.test.js # Tests apps work after optimization
├── performance/             # Performance benchmarks
│   └── optimization-benchmark.test.js # Speed and memory tests
├── fixtures/                # Test fixtures
│   └── lodash-chunk.js     # Sample chunk for testing
├── utils/                   # Test utilities
│   └── optimization.js      # Helper functions for testing
├── setup.js                 # Vitest setup file
└── README.md               # This file
```

## Running Tests

```bash
# Run all tests
pnpm test

# Run tests in watch mode
pnpm test:watch

# Run with UI
pnpm test:ui

# Run with coverage
pnpm test:coverage

# Run specific test suites
pnpm test:unit        # Unit tests only
pnpm test:integration # Integration tests only
pnpm test:e2e        # E2E tests only

# Run all tests including build
pnpm test:all
```

## Test Categories

### Unit Tests (`test/unit/`)

Fast, isolated tests for specific functionality:

- Tree shaking logic
- Macro processing
- Configuration handling
- CommonJS chunk format support

### Integration Tests (`test/integration/`)

Tests that verify the build and optimization pipeline:

- Build output verification
- Share usage analysis
- Optimization results
- File generation

### E2E Tests (`test/e2e/`)

Browser-based tests using Puppeteer:

- App functionality after optimization
- Module Federation runtime
- Remote component loading
- Error handling

### Performance Tests (`test/performance/`)

Benchmarks and performance metrics:

- Optimization speed
- Memory usage
- Size reduction metrics
- Quality checks

## Writing Tests

### Basic Test Structure

```javascript
import { describe, it, expect } from "vitest";

describe("Feature Name", () => {
	it("should do something", () => {
		// Arrange
		const input = createTestInput();

		// Act
		const result = performAction(input);

		// Assert
		expect(result).toBe(expectedValue);
	});
});
```

### Using Test Utils

```javascript
import { optimizeChunk, analyzeChunk } from "../utils/optimization.js";

it("should optimize chunk", () => {
	const optimized = optimizeChunk(chunkPath, config);
	const analysis = analyzeChunk(optimizedPath);

	expect(analysis.reduction).toBeGreaterThan(30);
});
```

### Testing with Fixtures

```javascript
it("should handle fixture correctly", () => {
	const fixture = path.join(__dirname, "../fixtures/lodash-chunk.js");
	const result = processFixture(fixture);

	expect(result).toMatchSnapshot();
});
```

## Coverage Goals

- Unit tests: > 90% coverage
- Integration tests: All critical paths
- E2E tests: Main user workflows
- Performance: Regression prevention

## CI/CD Integration

Tests are run automatically on:

- Pull requests
- Main branch commits
- Release builds

## Debugging Tests

```bash
# Run with Node debugging
node --inspect-brk ./node_modules/.bin/vitest run

# Run specific test file
pnpm vitest run test/unit/optimization.test.js

# Run tests matching pattern
pnpm vitest -t "tree shaking"
```

## Best Practices

1. **Keep tests focused**: One concept per test
2. **Use descriptive names**: Test names should explain what and why
3. **Avoid hard-coded paths**: Use path utilities
4. **Mock external dependencies**: Keep tests fast and reliable
5. **Clean up after tests**: Remove generated files
6. **Use fixtures for complex data**: Don't embed large strings
7. **Test error cases**: Not just happy paths

## Common Issues

### Tests fail with "module not found"

- Run `pnpm install` in the root directory
- Ensure `pnpm build` has been run

### E2E tests timeout

- Check if preview servers are running
- Increase timeout in test configuration
- Check for port conflicts

### Performance tests are slow

- This is expected for large chunks
- Run with `--no-coverage` for faster execution
- Use smaller fixtures for development

## Contributing

When adding new tests:

1. Place in appropriate category directory
2. Follow existing naming conventions
3. Add fixtures if needed
4. Update this README if adding new patterns
