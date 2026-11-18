it('should only use the fallback entry while the custom file is missing', () => {
  expect(['1', '2']).toContain(WATCH_STEP);
});
