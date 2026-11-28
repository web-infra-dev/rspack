it('omits federation bundler runtime for a shared-only consumer while keeping fallback working', async () => {
  expect((await import('shared-lib')).default).toBe('shared value');
  expect(Boolean(__webpack_require__.federation)).toBe(EXPECT_PARENT_FEDERATION);
});
