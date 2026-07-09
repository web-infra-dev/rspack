import { test, expect } from '@/fixtures';

const COLOR_BLUE = 'rgb(0, 0, 255)';
const COLOR_GREEN = 'rgb(0, 128, 0)';
const COLOR_NATIVE = 'rgb(255, 0, 0)';

test('extract-only css update does not disturb the native css runtime', async ({
  page,
  fileAction,
}) => {
  const root = page.locator('#root');
  await expect(root).toHaveText('feature loaded');
  await expect(root).toHaveCSS('color', COLOR_BLUE);
  await expect(page.locator('body')).toHaveCSS(
    'background-color',
    COLOR_NATIVE,
  );

  await page.evaluate(() => {
    for (const link of document.querySelectorAll('link[rel="stylesheet"]')) {
      link.setAttribute('data-e2e-initial', 'true');
    }
  });
  const cssRequests: string[] = [];
  page.on('request', (request) => {
    if (request.url().includes('.css')) {
      cssRequests.push(request.url());
    }
  });

  // only the extracted stylesheet changes, the native one is untouched
  fileAction.updateFile('src/blue.css', (content) =>
    content.replace('blue', 'green'),
  );

  await expect(root).toHaveCSS('color', COLOR_GREEN);
  await expect(page.locator('body')).toHaveCSS(
    'background-color',
    COLOR_NATIVE,
  );
  // the native stylesheet link survived the update untouched (no full reload, no swap)
  await expect(
    page.locator('link[data-e2e-initial][href*="feature.css"]'),
  ).toHaveCount(1);
  // only the extracted stylesheet was re-fetched, no url was synthesized for the native runtime
  expect(
    cssRequests.filter((url) => !url.includes('extract-main.css')),
  ).toEqual([]);
});
