it('activates hoisted and lexical declarations in their owning scopes', () => {
  function hoisted() {
    const value = require('./missing-function');
    function require() { return 'function'; }
    return value;
  }
  expect(hoisted()).toBe('function');
  for (let require = () => 'loop'; require; require = null) {
    expect(require('./missing-loop')).toBe('loop');
    const nested = () => 'body';
    expect(nested()).toBe('body');
  }
  switch (1) {
    case 1: {
      const require = () => 'switch';
      expect(require('./missing-switch')).toBe('switch');
      break;
    }
  }
  try { throw 1; } catch (_) {
    const require = () => 'catch body';
    expect(require('./missing-catch-body')).toBe('catch body');
  }
  class Local {
    static value;
    static {
      const require = () => 'static';
      this.value = require('./missing-static');
    }
  }
  expect(Local.value).toBe('static');
  expect(REPLACEMENT).toBe('replacement');
});

it('preserves parameter redeclarations and block function fallbacks', () => {
  function merged(require) {
    var require;
    return require('./missing-merged');
  }
  expect(merged(() => 'parameter')).toBe('parameter');
  if (true) {
    function require() { return 'block'; }
  }
  expect(require('./missing-block')).toBe('block');
});
