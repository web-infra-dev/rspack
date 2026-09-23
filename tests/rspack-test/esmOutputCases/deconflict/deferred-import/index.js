export * from '../../basic/deferred-import/index.js';

export const __rspack_deferred_exports = 42;

it('should not collide with the deferred namespace cache', async () => {
  const lib = await import(/* webpackIgnore: true */ './main.mjs');
  expect(__rspack_deferred_exports).toBe(42);
  expect(lib.__rspack_deferred_exports).toBe(42);
});
