it('should reuse resolutions whose missing probes contain Windows path fragments', async () => {
  expect(require('./value.js?C:\\fixtures\\normal')).toBe('value');
  expect(require('./value.js#C:\\fixtures\\normal')).toBe('value');
  if (COMPILER_INDEX === 0) await NEXT_START();
});
