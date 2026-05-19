it('should emit asset from child compiler when using JsonpTemplatePlugin', () => {
  const child = __non_webpack_require__('./child.js');
  const assetHref = child.default ?? child;

  expect(assetHref).toBe('https://test.cases/path/asset.png');
});
