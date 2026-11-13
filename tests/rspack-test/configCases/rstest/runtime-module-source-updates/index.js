it('keeps requirements in sync across runtime source updates', async () => {
  const extension = globalThis.runtimeSourceUpdates;
  expect(extension.strict).toBe(true);
  expect(extension.loaderType).toBe('function');
  expect(extension.defineType()).toBe('function');
  expect(extension.read()).toBe('/initial/first/second/');
  expect(__webpack_public_path__).toBe('/initial/first/second/');
  extension.write('/changed/');
  expect(__webpack_public_path__).toBe('/changed/');
  __webpack_public_path__ = '/module/';
  expect(extension.read()).toBe('/module/');
  expect((await import('./async.js')).value).toBe(42);

  const source = require('fs').readFileSync(
    require('path').join(__dirname, RUNTIME_FILENAME),
    'utf8',
  );
  expect(source).not.toContain('setRuntimeId');
  expect(source).not.toContain('setScriptNonce');
});
