export { default as value } from './foo.cjs';

it('should re-export a CommonJS default from another entry', async () => {
  const { value } = await import(
    /* webpackIgnore: true */ './main.mjs'
  );
  expect(value).toBe(42);
});
