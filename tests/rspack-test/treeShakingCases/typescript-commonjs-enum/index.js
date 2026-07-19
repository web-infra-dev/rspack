const { Used, readInternal, sideEffectCount } = require('./enums');

it('tree shakes only unused TypeScript CommonJS enums', () => {
  expect(Used.One).toBe(1);
  expect(Used[1]).toBe('One');
  expect(readInternal()).toBe('internal');
  expect(sideEffectCount()).toBe(1);
});
