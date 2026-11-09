const modules = import.meta.glob(
  ['@/pages/*.js', '!@/pages/excluded.js'],
  { eager: true, import: 'default' },
);
const modulesWithSuffix = import.meta.glob('@params/pages/entry.js', {
  eager: true,
  import: 'default',
});

it('tries the next alias target when the first lacks the literal glob directory', () => {
  expect(modules).toEqual({ '/second/pages/entry.js': 'second' });
});

it('retains the query and fragment of the selected alias target', () => {
  expect(modulesWithSuffix).toEqual({
    '/second/pages/entry.js': '?selected#fragment',
  });
});
