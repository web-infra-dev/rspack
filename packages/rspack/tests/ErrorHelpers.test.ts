import { describe, expect, it } from 'rstack/test';
import { cleanUp, cutOffMessage } from '../src/ErrorHelpers';

describe('ErrorHelpers', () => {
  it('should strip message when loader execution is the first stack frame', () => {
    const stack =
      'Error: loader failed\n    at LOADER_EXECUTION (/loader.js:1:1)';
    expect(cleanUp(stack, 'Error', 'loader failed')).toBe('');
    expect(
      cutOffMessage('Error: loader failed', 'Error', 'loader failed'),
    ).toBe('');
  });

  it('should strip multi-line error messages from stack traces', () => {
    const stack =
      'SyntaxError: Unexpected token\n  > 1 | foo(\n    at parse (/loader.js:10:5)\n    at LOADER_EXECUTION (/loader.js:1:1)';
    expect(
      cleanUp(stack, 'SyntaxError', 'Unexpected token\n  > 1 | foo('),
    ).toBe('    at parse (/loader.js:10:5)');
  });

  it('should strip empty error messages from stack traces', () => {
    const stack =
      'Error\n    at run (/loader.js:10:5)\n    at LOADER_EXECUTION (/loader.js:1:1)';
    expect(cleanUp(stack, 'Error', '')).toBe('    at run (/loader.js:10:5)');
    expect(
      cleanUp('Error\n    at LOADER_EXECUTION (/loader.js:1:1)', 'Error', ''),
    ).toBe('');
  });
});
