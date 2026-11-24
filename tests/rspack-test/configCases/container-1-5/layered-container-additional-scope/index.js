const { createInstance } = require('paired-runtime');

it('registers layered and unlayered providers only when the host sends a scope map', async () => {
  const remote = require('./remoteEntry.js');
  const primary = {};
  const extra = {};
  const scalar = {};
  const scopes = { default: primary, extra, scalar };
  if (__PASS_SCOPE_MAP__) {
    await remote.init(primary, [], {
      shareScopeKeys: 'default',
      shareScopeMap: scopes,
    });

  } else {
    await remote.init(primary, []);

  }
  expect(Object.keys(scopes).sort()).toEqual(['default', 'extra', 'scalar']);
  const host = createInstance({ name: `host-${__PASS_SCOPE_MAP__}`, remotes: [] });
  host.initShareScopeMap('extra', extra);
  host.initShareScopeMap('scalar', scalar);
  const plainFactory = await host.loadShare('plain', {
    customShareInfo: { scope: ['scalar'], shareConfig: { requiredVersion: false } },
  });
  const factory = await host.loadShare('lib', {
    customShareInfo: { scope: ['extra'], shareConfig: { layer: 'server', requiredVersion: false } },
  });
  if (__PASS_SCOPE_MAP__) {
    expect(factory()).toBe('layered provider');
    expect(plainFactory()).toBe('unlayered provider');
  } else {
    expect(factory).toBe(false);
    expect(plainFactory).toBe(false);
  }
  expect(primary).toEqual({});
  expect((await remote.get('./provider'))()).toBe('layered provider');
});
