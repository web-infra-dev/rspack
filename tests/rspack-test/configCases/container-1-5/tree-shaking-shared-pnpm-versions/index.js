const mfScope = typeof __rspack_context === 'undefined' ? __webpack_require__ : __rspack_context;

it('should generate tree shaking shared fallbacks for every resolved version', async () => {
  await Promise.all([import('ui-lib'), import('dep')]);

  const fallbacks = mfScope.federation.sharedFallback['ui-lib'];
  expect(fallbacks.map(([, version]) => version).sort()).toEqual([
    '1.0.0',
    '2.0.0',
  ]);

  const fallbackValues = Object.fromEntries(
    fallbacks.map(([entry, version, globalName]) => {
      const container = eval("require")(`./${entry}`)[globalName];
      return [version, container.get()().value];
    }),
  );
  expect(fallbackValues).toEqual({
    '1.0.0': 'direct-1',
    '2.0.0': 'nested-2',
  });
});
