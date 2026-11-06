const modules = import.meta.glob(
  [
    './src/components/*.js',
    '/src/components/*.js',
    './src/utils/*.js',
    '!./src/components/excluded.js',
    '!./src/unrelated/*.js',
  ],
  { eager: true, import: 'default' },
);

it('keeps the first positive key and applies negative patterns', () => {
  expect(modules).toEqual({
    './src/components/alpha.js': 'alpha',
    './src/utils/beta.js': 'beta',
  });
});
