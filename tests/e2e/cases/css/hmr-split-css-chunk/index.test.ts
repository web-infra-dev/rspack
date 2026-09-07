import { test, expect } from '@/fixtures';

const COLOR_BLUE = 'rgb(163, 255, 255)';
const COLOR_GREEN = 'rgb(0, 128, 0)';

test('editing css applies to the stylesheet extracted into its own chunk', async ({
  page,
  fileAction,
}) => {
  const body = page.locator('body');
  await expect(body).toHaveCSS('background-color', COLOR_BLUE);

  const cssRequests: string[] = [];
  page.on('request', (request) => {
    if (request.url().includes('.css')) {
      cssRequests.push(request.url());
    }
  });

  fileAction.updateFile('src/blue.css', (content) =>
    content.replace('rgb(163, 255, 255)', 'rgb(0, 128, 0)'),
  );

  await expect(body).toHaveCSS('background-color', COLOR_GREEN);
  await expect(page.locator('link[rel="stylesheet"]')).toHaveCount(1);
  expect(cssRequests.length).toBe(1);
});

test('an unchanged stylesheet is not refetched when its module recompiles', async ({
  page,
  fileAction,
}) => {
  const body = page.locator('body');
  await expect(body).toHaveCSS('background-color', COLOR_BLUE);
  await page.evaluate(() => {
    document
      .querySelector('link[rel="stylesheet"]')!
      .setAttribute('data-e2e-initial', 'true');
  });
  const cssRequests: string[] = [];
  page.on('request', (request) => {
    if (request.url().includes('.css')) {
      cssRequests.push(request.url());
    }
  });

  // editing a dependency of the stylesheet rebuilds the css module while the
  // emitted css stays byte-identical (a tailwind-style rebuild)
  fileAction.updateFile('src/dep.txt', () => 'v1\n');
  fileAction.updateFile('src/index.js', (content) =>
    content.replace('step-0', 'step-1'),
  );

  await expect(page.locator('#root')).toHaveText('step-1');
  await expect(body).toHaveCSS('background-color', COLOR_BLUE);
  // the legacy loader-level css reload is debounced; give it time to misfire
  await page.waitForTimeout(500);
  await expect(page.locator('link[data-e2e-initial]')).toHaveCount(1);
  expect(cssRequests).toEqual([]);
});
