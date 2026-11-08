it('should keep the module id assigned by beforeModuleIds', () => {
  expect(module.id).toBe('custom-entry-id');
  expect(WATCH_STEP).toBe('0');
});
