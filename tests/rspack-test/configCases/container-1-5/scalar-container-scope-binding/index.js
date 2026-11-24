it('binds the container scope to the host pool despite different scope names', async () => {
  const container = require('./remoteEntry.js');
  const pool = {};
  await container.init(pool, [], {
    shareScopeKeys: 'host-custom',
    shareScopeMap: { 'host-custom': pool },
  });

  const shared = {};
  pool.lib = { '1.0.0': { get: () => () => shared } };
  const widget = await container.get('./widget');
  expect(widget().default).toBe(shared);
});
