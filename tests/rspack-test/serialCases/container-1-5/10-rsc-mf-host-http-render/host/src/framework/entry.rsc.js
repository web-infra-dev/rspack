import { closeRemoteServer, ensureRemoteServer } from './httpRemoteServer';

const REMOTE_PORT = 13010;

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

it('should consume mixed RSC expose patterns over HTTP federation', async () => {
  const {
    RemoteClient,
    getServerOnlyInfo,
    RemoteServerCard,
    RemoteNestedMixed,
  } = await loadRemoteMatrix();

  try {
    expect(__rspack_rsc_manifest__).toBeDefined();

    expect(typeof RemoteClient).toBe('function');
    expect(typeof RemoteServerCard).toBe('function');
    expect(typeof RemoteNestedMixed).toBe('function');
    expect(getServerOnlyInfo()).toBe('server-only-ok');

    const serverElement = RemoteServerCard({ label: 'server-component' });
    const mixedElement = RemoteNestedMixed({ label: 'nested-component' });
    expect(typeof serverElement).toBe('object');
    expect(typeof mixedElement).toBe('object');

    const manifest = __webpack_require__.rscM;
    const remoteManifest = await getRemoteBridgeManifest();
    expect(manifest).toBeDefined();
    const hostClientManifestKeys = Object.keys(manifest.clientManifest || {});
    expect(hostClientManifestKeys.length).toBeGreaterThan(0);
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
    expect(Object.keys(manifest.serverConsumerModuleMap).length).toBeGreaterThan(0);
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
    const firstActionId = remoteActionIds[0];
    const firstActionEntry = manifest.serverManifest[firstActionId];
    expect(firstActionEntry).toBeDefined();
    const firstActionProxy = __webpack_require__(firstActionEntry.id);
    expect(
      Object.prototype.hasOwnProperty.call(firstActionProxy, firstActionEntry.name),
    ).toBe(true);
    expect(Object.prototype.hasOwnProperty.call(firstActionProxy, firstActionId)).toBe(
      true,
    );

    const prefixedActionIds = Object.keys(manifest.serverManifest).filter((id) =>
      id.startsWith('remote:rscRemote:'),
    );
    expect(prefixedActionIds.length).toBeGreaterThanOrEqual(3);
  } finally {
    await closeRemoteServer();
  }
});
