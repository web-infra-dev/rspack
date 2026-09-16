it('should export a usable module container with the runtime startup mode', async () => {
  const container = await import(/* webpackIgnore: true */ './remoteEntry.mjs');
  expect(typeof container.init).toBe('function');
  expect(typeof container.get).toBe('function');
  await container.init({});
  expect((await container.get('./exposed'))().default).toBe('exposed');
  expect((await import('lib')).default).toBe('lib');
});
