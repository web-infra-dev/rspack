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

async function getRemoteBridgeManifest() {
  const instance = __webpack_require__.federation?.instance;
  expect(typeof instance?.loadRemote).toBe('function');
  const bridge = await instance.loadRemote('rscRemote/__rspack_rsc_bridge__');
  expect(typeof bridge?.getManifest).toBe('function');
  return bridge.getManifest();
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
    const remoteManifest = await getRemoteBridgeManifest();
    expect(manifest).toBeDefined();
    const hostClientManifestKeys = Object.keys(manifest.clientManifest || {});
    expect(hostClientManifestKeys.some((key) => key.startsWith('remote-module:rscRemote:'))).toBe(
      true,
    );
    const remoteClientManifestKeys = Object.keys(remoteManifest.clientManifest || {});
    expect(remoteClientManifestKeys.length).toBeGreaterThan(0);
    const hasUnscopedClientManifestAlias = remoteClientManifestKeys.some((key) =>
      Object.prototype.hasOwnProperty.call(manifest.clientManifest, key),
    );
    expect(hasUnscopedClientManifestAlias).toBe(true);
    const hasScopedClientManifestAlias = remoteClientManifestKeys.some((key) =>
      Object.prototype.hasOwnProperty.call(
        manifest.clientManifest,
        `remote-module:rscRemote:${String(key)}`,
      ),
    );
    expect(hasScopedClientManifestAlias).toBe(true);
    const scopedClientManifestKey = remoteClientManifestKeys.find(
      (key) =>
        String(key).includes('RemoteClient') &&
        Object.prototype.hasOwnProperty.call(
          manifest.clientManifest,
          `remote-module:rscRemote:${String(key)}`,
        ),
    );
    expect(scopedClientManifestKey).toBeDefined();
    const resolvedClientEntry =
      manifest.clientManifest[`remote-module:rscRemote:${String(scopedClientManifestKey)}`];
    expect(resolvedClientEntry?.id).toBeDefined();
    if (
      String(resolvedClientEntry.id).startsWith(
        'webpack/container/remote/rscRemote/',
      )
    ) {
      expect(typeof __webpack_require__.m[resolvedClientEntry.id]).toBe(
        'function',
      );
    }
    expect(
      Object.prototype.hasOwnProperty.call(
        manifest.serverConsumerModuleMap,
        resolvedClientEntry.id,
      ),
    ).toBe(true);
    const remoteScopedManifest = manifest.remoteManifests?.rscRemote;
    expect(remoteScopedManifest?.moduleLoading ?? manifest.moduleLoading?.rscRemote).toEqual(
      remoteManifest.moduleLoading,
    );
    expect(Array.isArray(manifest.entryJsFiles)).toBe(true);
    expect(remoteScopedManifest?.entryJsFiles ?? manifest.entryJsFiles?.rscRemote).toEqual(
      remoteManifest.entryJsFiles,
    );
    expect(remoteScopedManifest?.entryCssFiles ?? manifest.entryCssFiles?.rscRemote).toEqual(
      remoteManifest.entryCssFiles,
    );

    const remoteActionIds = Object.keys(remoteManifest.serverManifest || {});
    expect(remoteActionIds.length).toBeGreaterThanOrEqual(3);
    const hasUnscopedActionAlias = remoteActionIds.some((actionId) =>
      Object.prototype.hasOwnProperty.call(manifest.serverManifest, actionId),
    );
    expect(hasUnscopedActionAlias).toBe(true);
    const hasScopedActionAlias = remoteActionIds.some((actionId) =>
      Object.prototype.hasOwnProperty.call(
        manifest.serverManifest,
        `remote:rscRemote:${String(actionId)}`,
      ),
    );
    expect(hasScopedActionAlias).toBe(true);

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
        expect(Object.prototype.hasOwnProperty.call(actionProxy, actionId)).toBe(
          true,
        );
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

    const rawActionResults = [];
    const rawActionErrors = [];
    for (const actionId of remoteActionIds) {
      try {
        const actionEntry = manifest.serverManifest[actionId];
        const actionProxy = __webpack_require__(actionEntry.id);
        expect(
          Object.prototype.hasOwnProperty.call(actionProxy, actionEntry.name),
        ).toBe(true);
        expect(Object.prototype.hasOwnProperty.call(actionProxy, actionId)).toBe(
          true,
        );
        const result = await actionProxy[actionEntry.name]('from-host-raw');
        if (typeof result === 'string') {
          rawActionResults.push(result);
        }
      } catch (error) {
        rawActionErrors.push(String(error));
      }
    }
    expect(rawActionResults.length).toBeGreaterThan(0);
    expect(rawActionErrors.length).toBeLessThan(remoteActionIds.length);
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
