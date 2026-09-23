import { assetUrl } from './shared.js';
it('should synchronously resolve assets from initial and async chunks', async () => {
  expect(assetUrl).toBe('https://test.cases/path/asset.txt');
  expect((await import('./lazy.js')).assetUrl).toBe(assetUrl);
});
