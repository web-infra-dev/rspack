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
    expect(Object.keys(manifest.clientManifest).length).toBeGreaterThan(0);
    expect(Object.keys(manifest.clientManifest).some((key) => key.includes('RemoteClient'))).toBe(
      true,
    );
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

    const prefixedActionIds = Object.keys(manifest.serverManifest).filter((id) =>
      id.startsWith('remote:rscRemote:'),
    );
    expect(prefixedActionIds.length).toBeGreaterThanOrEqual(3);
  } finally {
    await closeRemoteServer();
  }
});
