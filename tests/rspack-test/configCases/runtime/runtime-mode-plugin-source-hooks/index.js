it('applies native source and generate changes before the extension runtime and the entry', () => {
  const paths = __webpack_require__.webExtRt;
  expect(paths.hasEntry()).toBe(true);
  expect(paths.condition).toBe(true);
  if (PATH_KIND === 'auto') {
    expect(paths.previous).toMatch(/\/worker\/generated\/$/);
  } else if (PATH_KIND === 'worker') {
    expect(paths.previous).toBe(new URL('.', self.location.href).href);
  } else {
    expect(paths.previous).toBe('/configured/');
  }
  expect(__webpack_require__.globalHookRan).toBe(true);
  expect(__webpack_require__.g).toBe(globalThis);
  expect(__webpack_require__.b).toBe('https://extension.test/');
  expect(__webpack_public_path__).toBe('https://extension.test/assets/');
});
