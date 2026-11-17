it('loads local async chunks with the runtime selected for its entry', async () => {
  expect(Boolean(__webpack_require__.federation)).toBe(EXPECT_FEDERATION);
  expect((await import('./local.js')).default).toBe('local value');
});
