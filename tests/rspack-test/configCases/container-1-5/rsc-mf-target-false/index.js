const fs = __non_webpack_require__('fs');
const path = __non_webpack_require__('path');

it('should not apply node-only rsc mf validation for target false', async () => {
  const remoteEntryPath = path.join(__dirname, 'remoteEntry.js');
  const remoteEntrySource = fs.readFileSync(remoteEntryPath, 'utf-8');

  expect(remoteEntrySource).not.toContain('__rspack_rsc_bridge__');
  expect(remoteEntrySource).not.toContain('rspack-rsc-runtime-plugin');

  const container = __non_webpack_require__(remoteEntryPath);
  await expect(
    Promise.resolve().then(() => container.get('./__rspack_rsc_bridge__')),
  ).rejects.toBeTruthy();
});
