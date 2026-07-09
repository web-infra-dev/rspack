import { test, expect } from '@/fixtures';

const COLOR_EXTRACT = 'rgb(255, 0, 0)';
const COLOR_EXTRACT_UPDATED = 'rgb(128, 0, 128)';
const COLOR_NATIVE = 'rgb(0, 128, 0)';

test('extract-only css update does not disturb the native css runtime', async ({
  page,
  fileAction,
}) => {
  const root = page.locator('#root');
  await expect(root).toHaveText('feature loaded');
  // the entry chunk carries both css kinds: extracted (#root) and native (body)
  await expect(root).toHaveCSS('color', COLOR_EXTRACT);
  await expect(page.locator('body')).toHaveCSS(
    'background-color',
    COLOR_NATIVE,
  );

  // a full page reload would recreate #root and lose this marker
  await page.evaluate(() => {
    document.getElementById('root')!.setAttribute('data-e2e-initial', 'true');
  });
  const cssRequests: string[] = [];
  page.on('request', (request) => {
    if (request.url().includes('.css')) {
      cssRequests.push(request.url());
    }
  });

  // only the extracted stylesheet changes, the native one is untouched
  fileAction.updateFile('src/blue.css', (content) =>
    content.replace('red', 'purple'),
  );

  await expect(root).toHaveCSS('color', COLOR_EXTRACT_UPDATED);
  await expect(page.locator('body')).toHaveCSS(
    'background-color',
    COLOR_NATIVE,
  );
  await expect(page.locator('#root[data-e2e-initial]')).toHaveCount(1);
  // the native css runtime must not re-fetch its stylesheet
  // (its hot-update urls carry the `hmr` query) for an extract-only change
  expect(cssRequests.filter((url) => url.includes('hmr='))).toEqual([]);
});
