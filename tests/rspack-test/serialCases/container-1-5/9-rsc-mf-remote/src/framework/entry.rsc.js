import { RemoteClient } from '../RemoteClient';

it('should build remote RSC manifest and action bridge data', async () => {
  expect(typeof RemoteClient).toBe('function');

  const manifest = __rspack_rsc_manifest__;
  expect(manifest).toBeDefined();

  const { clientManifest, serverManifest, serverConsumerModuleMap } = manifest;
  expect(clientManifest).toBeDefined();
  expect(Object.keys(clientManifest)).toContain(CLIENT_PATH);

  const actionIds = Object.keys(serverManifest || {});
  expect(actionIds.length).toBeGreaterThan(0);

  expect(serverConsumerModuleMap).toBeDefined();
  expect(Object.keys(serverConsumerModuleMap).length).toBeGreaterThan(0);
});
