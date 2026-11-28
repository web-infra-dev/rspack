it('resolves shared consumer fallback without requiring federation bundler runtime', async () => {
  expect(Boolean(__webpack_require__.federation)).toBe(EXPECT_CONSUME_FEDERATION);
  expect((await import('./consume-async.js')).default).toBe('shared value');
});
