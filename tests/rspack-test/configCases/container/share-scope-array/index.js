it('should initialize a single-item shareScope array', async () => {
  const remote = require('./federation.js');
  const scope = {};
  await remote.init(scope);
  const factory = await remote.get('./scope');
  expect(factory()).toBe(scope);
});
