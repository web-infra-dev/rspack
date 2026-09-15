it('should emit asset from child compiler when using JsonpTemplatePlugin', () => {
  const child = require('./child.js');
  const assetHref = child.default ?? child;

  expect(assetHref).toBe('https://test.cases/path/asset.png');
});
