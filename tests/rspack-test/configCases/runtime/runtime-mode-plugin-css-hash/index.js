it('loads the emitted CSS using its final chunk content hash and compilation hash', async () => {
  await import(/* webpackChunkName: "styles" */ './lazy.css');
  expect(__webpack_require__.pluginCssHash).toMatch(/^[a-f0-9]{8}/);
  const file = __webpack_require__.miniCssF('styles');
  expect(file).toBe(`styles.${__webpack_hash__.slice(0, 8)}.${__webpack_require__.pluginCssHash.slice(0, 8)}.css`);
  expect([...document.getElementsByTagName('link')].some(link => link.href.endsWith('/' + file))).toBe(true);
});
