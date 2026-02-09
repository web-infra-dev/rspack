import { closeRemoteServer, ensureRemoteServer } from './httpRemoteServer';

const REMOTE_PORT = 13011;

async function loadRemoteMatrix() {
  await ensureRemoteServer(REMOTE_PORT);
  const [clientMod, serverOnlyMod, serverCardMod, nestedMixedMod] =
    await Promise.all([
      import('rscRemote/ClientWidget'),
      import('rscRemote/ServerOnlyInfo'),
      import('rscRemote/ServerComponent'),
      import('rscRemote/NestedMixed'),
    ]);

  const unwrapExport = (module, key) =>
    module?.[key] ?? module?.default?.[key] ?? module?.default;

  return {
    RemoteClient: unwrapExport(clientMod, 'RemoteClient'),
    getServerOnlyInfo: unwrapExport(serverOnlyMod, 'getServerOnlyInfo'),
    RemoteServerCard: unwrapExport(serverCardMod, 'RemoteServerCard'),
    RemoteNestedMixed: unwrapExport(nestedMixedMod, 'RemoteNestedMixed'),
  };
}

it('should execute all remote server actions via prefixed action ids', async () => {
  const {
    RemoteClient,
    getServerOnlyInfo,
    RemoteServerCard,
    RemoteNestedMixed,
  } = await loadRemoteMatrix();

  try {
    expect(typeof RemoteClient).toBe('function');
    expect(typeof RemoteServerCard).toBe('function');
    expect(typeof RemoteNestedMixed).toBe('function');
    expect(getServerOnlyInfo()).toBe('server-only-ok');

    const manifest = __webpack_require__.rscM;
    expect(manifest).toBeDefined();

    const prefixedActionIds = Object.keys(manifest.serverManifest).filter((id) =>
      id.startsWith('remote:rscRemote:'),
    );
    expect(prefixedActionIds.length).toBeGreaterThanOrEqual(3);

    const actionResults = [];
    const actionErrors = [];
    for (const actionId of prefixedActionIds) {
      try {
        const proxyModuleId = manifest.serverManifest[actionId].id;
        const actionProxy = __webpack_require__(proxyModuleId);
        const result = await actionProxy[actionId]('from-host');
        if (typeof result === 'string') {
          actionResults.push(result);
        }
      } catch (error) {
        // Skip non-action entries that may share the prefixed id space.
        actionErrors.push(String(error));
      }
    }
    expect(actionResults).toEqual(
      expect.arrayContaining([
        'remote-action:from-host',
        'remote-secondary:from-host',
        'nested-action:from-host',
      ]),
    );
    expect(actionErrors.length).toBeLessThan(prefixedActionIds.length);
  } finally {
    await closeRemoteServer();
  }
});

it('should isolate default/ssr/rsc share scopes', async () => {
  const { RemoteClient } = await loadRemoteMatrix();
  try {
    expect(typeof RemoteClient).toBe('function');

    await __webpack_init_sharing__('default');
    await __webpack_init_sharing__('ssr');
    await __webpack_init_sharing__('rsc');

    const hostName = __webpack_require__.federation.initOptions.name;
    const federationShare = __FEDERATION__.__SHARE__[hostName];
    const hasReactInScope = (scope) => {
      if (!scope || typeof scope !== 'object') {
        return false;
      }
      if (scope.react) {
        return true;
      }
      return Object.values(scope).some(
        (value) => value && typeof value === 'object' && value.react,
      );
    };

    expect(Object.keys(federationShare || {})).toEqual(
      expect.arrayContaining(['default', 'ssr', 'rsc']),
    );
    expect(hasReactInScope(federationShare.rsc)).toBe(true);
    expect(federationShare.default).not.toBe(federationShare.ssr);
    expect(federationShare.default).not.toBe(federationShare.rsc);
    expect(federationShare.ssr).not.toBe(federationShare.rsc);
  } finally {
    await closeRemoteServer();
  }
});
