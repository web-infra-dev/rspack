it('updates federation inclusion when static usage changes', async () => {
  expect(Boolean(__webpack_require__.federation)).toBe(WATCH_STEP === '1' || WATCH_STEP === '2');
  const { default: value } = await import('./feature.js');
  expect(value).toBe(['local', 'shared', 'shared changed', 'local again', 'local changed'][WATCH_STEP]);
});
