import value from 'shared-lib';

it('should load an eager shared module without enabling chunk loading', () => {
  expect(value).toBe('shared value');
});
