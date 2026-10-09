import { expect, test } from '@/fixtures';

test('activating redirected lazy reexports preserves the barrel factories', async ({
  page,
  rspack,
}) => {
  await expect(page.locator('body')).toHaveAttribute(
    'data-infra',
    'star,named',
  );

  const factories = () => {
    const assets = rspack.compiler._lastCompilation!.getAssets();
    return ['index', 'named'].map((name) => {
      const marker = `"./src/lib/${name}.js"(`;
      for (const asset of assets) {
        if (
          !asset.name.endsWith('.js') ||
          asset.name.endsWith('.hot-update.js')
        ) {
          continue;
        }
        const source = asset.source.source().toString();
        const start = source.indexOf(marker);
        if (start !== -1) {
          const end = source.indexOf('\n}', start);
          expect(end).toBeGreaterThan(start);
          return source.slice(start, end + 2);
        }
      }
      throw new Error(`Missing barrel factory: ${name}`);
    });
  };
  const before = factories();
  const updates: Promise<string>[] = [];
  page.on('response', (response) => {
    if (response.url().endsWith('.hot-update.js')) {
      updates.push(response.text());
    }
  });

  await page.getByText('Activate feature').click();
  await expect(page.locator('body')).toHaveAttribute(
    'data-feature',
    'feature,named-feature',
  );

  expect(
    await page.evaluate(() => sessionStorage.getItem('documentLoads')),
  ).toBe('1');
  expect(factories()).toEqual(before);
  const hotUpdates = (await Promise.all(updates)).join('\n');
  expect(hotUpdates).not.toContain('"./src/lib/index.js"(');
  expect(hotUpdates).not.toContain('"./src/lib/named.js"(');
});
