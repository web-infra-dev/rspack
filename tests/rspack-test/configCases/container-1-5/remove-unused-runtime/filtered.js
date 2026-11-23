it('preserves initialization according to the filtering scope and shared runtime', () => {
  expect(Boolean(__webpack_require__.federation)).toBe(EXPECT_FILTERED_FEDERATION);
  if (EXPECT_FILTERED_FEDERATION) {
    expect(__webpack_require__.federation.exclusionPlugin).toBe('initialized');
    expect(__webpack_require__.federation.instance).toBeTruthy();
  }
});
