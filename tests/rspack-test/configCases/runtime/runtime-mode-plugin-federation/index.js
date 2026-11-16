const startedBeforeEntry = __webpack_require__.federation.started;

it('runs the wrapped startup and calls remote and sharing handlers on a real import', async () => {
  expect(startedBeforeEntry).toBe(true);
  expect((await import(/* webpackChunkName: "remote" */ './remote.js')).value).toBe(42);
  expect(__webpack_require__.federation.calls).toEqual([['remote', 'remote'], ['shared', 'remote']]);
  expect(__webpack_require__.I('default')).toBe(__webpack_require__.S.default);
  expect(__webpack_require__.S.default.remote).toBe(true);
  const [runtimeId, entryId, startupEntrypoint, onChunksLoaded] = __webpack_require__.federation.runtime();
  expect(runtimeId).toBe('main');
  expect(entryId).toBe(require.resolve('./index.js'));
  expect([startupEntrypoint, onChunksLoaded]).toEqual(['entry ready', 'remote ready']);
});
