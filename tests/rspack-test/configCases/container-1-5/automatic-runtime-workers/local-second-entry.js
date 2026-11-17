it('omits unused federation runtime from the second local entry in the same compilation', async () => {
  expect(Boolean(__webpack_require__.federation)).toBe(EXPECT_PARENT_FEDERATION);
  expect((await import('./local.js')).default).toBe('local value');
});
