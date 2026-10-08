import { assetUrl } from './shared.js';
it('should synchronously resolve assets from initial and async chunks', async () => {
  expect(assetUrl).toBe('https://test.cases/path/asset.txt');
  expect(import.meta.resolve('./asset.txt')).toBe(assetUrl);
  expect((await import('./lazy.js')).assetUrl).toBe(assetUrl);
  expect((await import('./async.txt')).default).toBe('/path/async.txt');
});
