it('should restore a nested import when the shared chunk gains this runtime', async () => {
  const { loadFeature } = await import(/* webpackChunkName: "bridge" */ './bridge');
  const imported = await loadFeature();
  expect(imported.value).toBe('leaf');
});
