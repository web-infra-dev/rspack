it('omits unused federation runtime from the first local entry alongside federation entries and Workers', async () => {
  expect(Boolean(__webpack_require__.federation)).toBe(EXPECT_PARENT_FEDERATION);
  expect((await import('./local.js')).default).toBe('local value');
});
