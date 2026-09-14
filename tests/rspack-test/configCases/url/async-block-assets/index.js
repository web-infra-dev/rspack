import sharedUrl from './shared.js';

const resourceUrl = new URL('./resource.txt', import.meta.url);
const inlineUrl = new URL('./inline.txt', import.meta.url);
const sourceUrl = new URL('./source.txt', import.meta.url);
const customUrl = new URL('./custom.txt', import.meta.url);

it('should retain synchronous asset URLs after probing target module types', () => {
  expect(resourceUrl.pathname).toBe('/path/resource.txt');
  expect(sharedUrl.href).toBe(resourceUrl.href);
  expect(inlineUrl.protocol).toBe('data:');
  expect(sourceUrl.pathname).toBe('/source-content');
  expect(customUrl.pathname).toBe('/custom/custom.txt');
});

it('should preserve regular dynamic imports of assets', async () => {
  const asset = await import('./dynamic.txt');
  expect(asset.default).toBe('/path/dynamic.txt');
});
