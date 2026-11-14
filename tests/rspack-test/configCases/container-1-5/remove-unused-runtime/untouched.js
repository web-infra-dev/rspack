it('retains unmatched entries even when another entry opts into filtering', () => {
  expect(Boolean(__webpack_require__.federation)).toBe(EXPECT_UNTOUCHED_FEDERATION);
  if (EXPECT_UNTOUCHED_FEDERATION) {
    expect(__webpack_require__.federation.exclusionPlugin).toBe('initialized');
    expect(__webpack_require__.federation.instance).toBeTruthy();
  }
});
