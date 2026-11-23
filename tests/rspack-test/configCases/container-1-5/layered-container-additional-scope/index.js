const { createInstance } = require('paired-runtime');

it('registers the additional layered provider only when the host sends a scope map', async () => {
  const remote = require('./remoteEntry.js');
  const primary = {};
  const extra = {};
  const scopes = { default: primary, extra };
  if (__PASS_SCOPE_MAP__) {
    await remote.init(primary, [], {
      shareScopeKeys: 'default',
      shareScopeMap: scopes,
    });

  } else {
    await remote.init(primary, []);

  }
  const host = createInstance({ name: `host-${__PASS_SCOPE_MAP__}`, remotes: [] });
  host.initShareScopeMap('extra', extra);
  const factory = await host.loadShare('lib', {
    customShareInfo: { scope: ['extra'], shareConfig: { layer: 'server', requiredVersion: false } },
  });
  if (__PASS_SCOPE_MAP__) {
    expect(factory()).toBe('layered provider');
  } else {
    expect(factory).toBe(false);
  }
  expect(primary).toEqual({});
  expect((await remote.get('./provider'))()).toBe('layered provider');
});
