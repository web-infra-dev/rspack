import './style.css';

const fs = require('fs');
const path = require('path');

const jsAsset = new URL('js-asset', import.meta.url);
const jsAssetUrl = new URL('js-asset-url', import.meta.url);

it('should resolve an asset external from javascript, whichever type it is', () => {
  expect(jsAsset.toString()).toBe('https://example.test/js-asset.png');
  expect(jsAssetUrl.toString()).toBe('https://example.test/js-asset-url.png');
});

it('should keep an asset external in the stylesheet, whichever type it is', () => {
  const css = fs.readFileSync(
    path.join(__STATS__.outputPath, 'bundle0.css'),
    'utf-8',
  );

  expect(css).toContain('url("https://example.test/css-asset.png")');
  expect(css).toContain('url("https://example.test/css-asset-url.png")');
});
