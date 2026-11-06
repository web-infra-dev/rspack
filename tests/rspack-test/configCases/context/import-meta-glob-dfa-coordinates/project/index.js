const modules = import.meta.glob(['/**/*.mjs', '../outside/*.mjs'], {
  exhaustive: true,
  eager: true,
  import: 'default',
});

it('preserves root-relative keys when the scan starts above the project', () => {
  expect(modules).toEqual({ '/../outside/value.mjs': 'outside' });
});
