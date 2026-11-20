const load = (scenario) => __non_webpack_require__(`./${scenario}/main.js`);

for (const [scenario, reason] of [
  ['consumer-layer', /consumer layer "server"/],
  ['provider-layer', /provider layer "server"/],
  ['fallback-scopes', /distinct full fallbacks for version "1.0.0"/],
  ['fallback-layers', /provider layer/],
]) {
  it(`rejects ${scenario} on a runtime without shared identity support`, () => {
    let error;
    try {
      load(scenario);
    } catch (e) {
      error = e;
    }
    expect(error).toBeInstanceOf(Error);
    expect(error.message).toMatch(reason);
    expect(error.message).toMatch(/supportsSharedIdentity/);
    expect(error.message).toMatch(/implementation/);
  });
}

it('does not check unused issuerLayer configurations', async () => {
  expect(await load('unused-issuer-layer')()).toBe('one');
  expect(await load('reachable-child-shared')()).toBe('dependency');
});

for (const scenario of ['safe-scopes', 'paired-scopes', 'paired-layers']) {
  it(`keeps full fallbacks distinct: ${scenario}`, async () => {
    const federation = load(scenario);
    expect(await federation.load()).toEqual(['one', 'two']);
    // Full getters are observable by external hosts using the registration.
    for (const arg of federation.initOptions.shared.key) {
      const name = scenario.endsWith('layers') ? arg.shareConfig.layer : arg.scope[0];
      expect((await arg.get())()).toBe(name);
    }
  });
}

it('does not hand a layered provider to an unlayered host', async () => {
  const federation = load('paired-isolated-provider');
  expect(federation.bundlerRuntime.supportsSharedIdentity).toBe(true);
  await federation.instance.initializeSharing('default');
  const host = federation.runtime.init({
    name: 'unlayered_external_host',
    shared: {
      one: {
        version: '1.0.0',
        scope: ['default'],
        shareConfig: { requiredVersion: false },
        get: () => () => 'unlayered',
      },
    },
  });
  host.initShareScopeMap('default', federation.instance.shareScopeMap.default);
  expect((await host.loadShare('one'))()).toBe('unlayered');
});
