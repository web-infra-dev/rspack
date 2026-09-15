const resourceUrl = new URL('./resource.txt', import.meta.url);
const inlineUrl = new URL('./inline.txt', import.meta.url);
const sourceUrl = new URL('./source.txt', import.meta.url);
const customUrl = new URL('./custom.txt', import.meta.url);

it('should retain synchronous asset URLs after probing target module types', () => {
  expect(resourceUrl.pathname).toBe('/path/resource.txt');
  expect(inlineUrl.protocol).toBe('data:');
  expect(sourceUrl.pathname).toBe('/source-content');
  expect(customUrl.pathname).toBe('/custom/custom.txt');
});

it('should retain asset URLs inside a require.ensure chunk group', async () => {
  const urls = await new Promise((resolve, reject) => {
    require.ensure([], require => {
      resolve([
        new URL('./resource.txt', import.meta.url),
        new URL('./inline.txt', import.meta.url),
        new URL('./source.txt', import.meta.url),
        new URL('./custom.txt', import.meta.url),
      ]);
    }, error => reject(error));
  });
  expect(urls[0].pathname).toBe('/path/resource.txt');
  expect(urls[1].protocol).toBe('data:');
  expect(urls[2].pathname).toBe('/source-content');
  expect(urls[3].pathname).toBe('/custom/custom.txt');
});
