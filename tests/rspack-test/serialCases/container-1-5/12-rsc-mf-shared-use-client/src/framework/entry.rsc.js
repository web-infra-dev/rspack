import { RemoteClient } from '../RemoteClient';

it('should include use-client shared package modules in the RSC manifest', async () => {
  expect(typeof RemoteClient).toBe('function');

  const manifest = __rspack_rsc_manifest__;
  expect(manifest).toBeDefined();

  const { clientManifest, serverManifest, serverConsumerModuleMap } = manifest;
  expect(clientManifest).toBeDefined();
  expect(Object.keys(clientManifest)).toContain(CLIENT_PATH);

  const sharedClientManifestKey = Object.keys(clientManifest).find((key) =>
    key.includes(SHARED_CLIENT_MARKER),
  );
  expect(sharedClientManifestKey).toBeDefined();

  const sharedClientManifestExport = clientManifest[sharedClientManifestKey];
  expect(sharedClientManifestExport).toBeDefined();
  expect(serverConsumerModuleMap[sharedClientManifestExport.id]).toBeDefined();
  if (!sharedClientManifestExport.id.startsWith('(server-side-rendering)/')) {
    expect(
      serverConsumerModuleMap[
        `(server-side-rendering)/${sharedClientManifestExport.id}`
      ],
    ).toBeDefined();
  }

  const consumeSharedManifestKey = Object.keys(clientManifest).find(
    (key) =>
      key.includes('webpack/sharing/consume/') &&
      key.includes(SHARED_CLIENT_MARKER),
  );
  expect(consumeSharedManifestKey).toBeDefined();

  const consumeSharedManifestExport = clientManifest[consumeSharedManifestKey];
  expect(consumeSharedManifestExport).toBeDefined();
  expect(serverConsumerModuleMap[consumeSharedManifestExport.id]).toBeDefined();
  if (!consumeSharedManifestExport.id.startsWith('(server-side-rendering)/')) {
    expect(
      serverConsumerModuleMap[
        `(server-side-rendering)/${consumeSharedManifestExport.id}`
      ],
    ).toBeDefined();
  }

  const actionIds = Object.keys(serverManifest || {});
  expect(actionIds.length).toBeGreaterThan(0);

  expect(serverConsumerModuleMap).toBeDefined();
  expect(Object.keys(serverConsumerModuleMap).length).toBeGreaterThan(0);
});
