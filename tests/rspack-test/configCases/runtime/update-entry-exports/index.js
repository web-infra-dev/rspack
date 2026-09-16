const initial = module.exports;
const entryId = module.id;

it('replaces the retained startup exports only for the owning entry', () => {
  const runtime = __TEST_RUNTIME__;
  if (!globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) expect(runtime.readStartupExports()).toBe(initial);
  expect(runtime.updateEntryExports(require.resolve('./first'), {})).toBe(false);
  if (!globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) expect(runtime.readStartupExports()).toBe(initial);
  const next = { generation: 2 };
  expect(runtime.updateEntryExports(entryId, next)).toBe(true);
  if (!globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) expect(runtime.readStartupExports()).toBe(next);
  expect(runtime.updateEntryExports('missing', {})).toBe(false);
  if (!globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) expect(runtime.readStartupExports()).toBe(next);
  // Synchronizing startup exports does not invalidate the module cache itself.
  expect(runtime.c[entryId].exports).toBe(initial);
});
