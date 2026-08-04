import { test, expect } from '@/fixtures';

const COLOR_BLUE = 'rgb(163, 255, 255)';
const COLOR_GREEN = 'rgb(0, 128, 0)';
const COLOR_RED = 'rgb(255, 0, 0)';
const COLOR_NONE = 'rgba(0, 0, 0, 0)';

test('removing the css import removes the stylesheet link', async ({
  page,
  fileAction,
}) => {
  await expect(page.locator('body')).toHaveCSS('background-color', COLOR_BLUE);
  await expect(page.locator('link[rel="stylesheet"]')).toHaveCount(1);

  fileAction.updateFile('src/index.js', (content) =>
    content.replace("import './blue.css';", ''),
  );

  await expect(page.locator('body')).toHaveCSS('background-color', COLOR_NONE);
  await expect(page.locator('link[rel="stylesheet"]')).toHaveCount(0);
});

test('js-only update does not touch the stylesheet', async ({
  page,
  fileAction,
}) => {
  await expect(page.locator('body')).toHaveCSS('background-color', COLOR_BLUE);
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

  fileAction.updateFile('src/index.js', (content) =>
    content.replace('step-0', 'step-1'),
  );

  await expect(page.locator('#root')).toHaveText('step-1');
  await expect(page.locator('body')).toHaveCSS('background-color', COLOR_BLUE);
  // the initial link element is still in place, it was neither re-fetched nor swapped
  await expect(page.locator('link[data-e2e-initial]')).toHaveCount(1);
  expect(cssRequests).toEqual([]);
});

test('css-only update swaps the stylesheet without piling up links', async ({
  page,
  fileAction,
}) => {
  await expect(page.locator('body')).toHaveCSS('background-color', COLOR_BLUE);

  fileAction.updateFile('src/blue.css', (content) =>
    content.replace('rgb(163, 255, 255)', 'rgb(0, 128, 0)'),
  );
  await expect(page.locator('body')).toHaveCSS('background-color', COLOR_GREEN);
  await expect(page.locator('link[rel="stylesheet"]')).toHaveCount(1);

  fileAction.updateFile('src/blue.css', (content) =>
    content.replace('rgb(0, 128, 0)', 'rgb(255, 0, 0)'),
  );
  await expect(page.locator('body')).toHaveCSS('background-color', COLOR_RED);
  await expect(page.locator('link[rel="stylesheet"]')).toHaveCount(1);
});
