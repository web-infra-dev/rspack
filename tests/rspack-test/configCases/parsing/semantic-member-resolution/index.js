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
