it('reuses resolved member roots without losing aliases or computed keys', () => {
  const alias = require;
  expect(alias('./value').nested.call()).toBe('value');
  expect(require.resolve('./value')).toBeDefined();
  expect(DEFINED.branch.value).toBe('defined');
  expect(typeof DEFINED.branch.value).toBe('string');
  const local = { nested: { call() { return this.value; }, value: 'local' } };
  expect(local['nested'].call()).toBe('local');
  expect(local?.nested.call()).toBe('local');
});

it('shares await-import member analysis for reads and calls', async () => {
  expect((await import('./value')).nested.value).toBe('value');
  expect((await import('./value')).nested.call()).toBe('value');
  expect((await import('./value')).nested?.call()).toBe('value');
  expect((await import('./value')).missing?.call()).toBeUndefined();
});

it('keeps special-root bindings in function, arrow and replacement-AST contexts', () => {
  (function() {
    // The IIFE aliases this to require; its arrow must inherit the same alias.
    const read = () => this('./value').nested.value;
    expect(read()).toBe('value');
  }).call(require);

  function readThis() {
    // DefinePlugin parses this replacement as a separate AST.
    return THIS_VALUE;
  }
  expect(readThis.call({ value: 'receiver' })).toBe('receiver');

  class Receiver {
    value = 'instance';
    read = () => this.value;
  }
  expect(new Receiver().read()).toBe('instance');

  function Target() {
    this.target = new.target;
  }
  expect(new Target().target).toBe(Target);
  expect(import.meta.webpack).toBeDefined();
});
