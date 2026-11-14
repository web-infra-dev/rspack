it("keeps runtime extension globals writable and preserves runtime scope", async () => {
  const extension = __webpack_require__.extension;
  expect(extension.shared).toBe(42);
  expect(extension.namespace).toBe(true);
  expect(extension.shadowed).toBe("local-context");
  expect(extension.literal).toBe("中文😀__webpack_require__.p");
  expect(extension.strict).toBe(true);
  // Reading an optional helper must not install its runtime module.
  expect(extension.loadScriptType).toBe("undefined");
  expect(extension.scopedPath).toBe("local-context-/initial/first/second/");
  expect(__webpack_require__.generateValue).toBe(2);
  expect(__webpack_require__.bufferValue).toBe(3);
  expect(__webpack_require__.extensionHash).toBe(__webpack_hash__);
  __webpack_require__.setNonce('updated-nonce');
  expect(__webpack_nonce__).toBe('updated-nonce');
  expect(extension.updateNonce(1)).toEqual([1, 3, 6]);
  expect(__webpack_nonce__).toBe(6);
  if (ESM_OUTPUT) {
    const source = require('fs').readFileSync(require('path').join(__dirname, 'runtime1.mjs'), 'utf8');
    expect(source).toContain('export { setPublicPath };');
    expect(source).toContain('export { setEnsureChunk };');
  }
  expect(extension.getPath()).toBe("/initial/first/second/");
  extension.setPath("/changed/");
  expect(__webpack_public_path__).toBe("/changed/");
  __webpack_public_path__ = "/module/";
  expect(extension.getPath()).toBe("/module/");
  expect(__webpack_require__.e).toBe(extension.ensure);
  expect((await import(/* webpackChunkName: "extension-async" */ "./async.js")).value).toBe(123);
  expect(__webpack_public_path__).toBe("/module/async/");
  expect(extension.getPath()).toBe("/module/async/");
  extension.setPath("/after-async/");
  expect(__webpack_public_path__).toBe("/after-async/");
  expect(extension.getPath()).toBe("/after-async/");
  expect(__webpack_require__.getAsyncPath()).toBe("/after-async/");
  __webpack_public_path__ = "/after-module/";
  expect(extension.getPath()).toBe("/after-module/");
  expect(__webpack_require__.getAsyncPath()).toBe("/after-module/");
  __webpack_require__.setAsyncPath("/from-async/");
  expect(__webpack_public_path__).toBe("/from-async/");
  expect(extension.getPath()).toBe("/from-async/");
  // Modern-module output uses native import(); the context output calls ensureChunk.
  expect(extension.calls).toBe(ESM_OUTPUT ? 0 : 1);
  const replacement = chunkId => extension.ensure(chunkId);
  extension.replaceEnsure(replacement);
  expect(__webpack_require__.e).toBe(replacement);
});
