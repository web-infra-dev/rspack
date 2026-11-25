import assert from 'node:assert/strict';

export default function stackCleanupLoader(content) {
  const error = new Error('loader-error');
  // Exercise the built loader runner: this frame must survive compression.
  assert.match(error.stack, /at LOADER_EXECUTION/);
  const originalStack = error.stack;
  this.emitError(error);
  this.emitWarning(error);
  assert.equal(error.stack, originalStack);

  const custom = new Error('LOADER_EXECUTION in message');
  custom.stack = `Error: ${custom.message}\n    at customLoader (/project/LOADER_EXECUTION.js:1:1)\n    at caller (/project/caller.js:2:1)`;
  this.emitError(custom);

  const firstFrame = new Error('loader failed');
  firstFrame.stack =
    'Error: loader failed\n    at LOADER_EXECUTION (/loader.js:1:1)';
  this.emitError(firstFrame);

  const multiLine = new SyntaxError('Unexpected token\n  > 1 | foo(');
  multiLine.stack =
    'SyntaxError: Unexpected token\n  > 1 | foo(\n    at parse (/loader.js:10:5)\n    at LOADER_EXECUTION (/loader.js:1:1)';
  this.emitError(multiLine);

  const emptyMessage = new Error('');
  emptyMessage.stack =
    'Error\n    at run (/loader.js:10:5)\n    at LOADER_EXECUTION (/loader.js:1:1)';
  this.emitError(emptyMessage);

  return content;
}
