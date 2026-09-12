it('should assign exposes layers without module rules', async () => {
  const container = require('./container-file.js');
  await container.init({});
  const modules = await Promise.all(
    ['./server', './client', './empty', './default'].map(async (key) => {
      const factory = await container.get(key);
      return factory();
    }),
  );

  expect(modules.map((module) => module.layer)).toEqual([
    'server',
    'client',
    '',
    null,
  ]);
  expect(new Set(modules).size).toBe(4);
});
