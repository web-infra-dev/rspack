it('should restore a second nested import after another sibling activates', async () => {
  const { loadSecondFeature } = await import(
    /* webpackChunkName: "second-bridge" */ './second-bridge'
  );
  const imported = await loadSecondFeature();
  expect(imported.value).toBe('second-leaf');
});
