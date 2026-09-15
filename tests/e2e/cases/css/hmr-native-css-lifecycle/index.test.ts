import { test, expect } from '@/fixtures';

const COLOR_BLUE = 'rgb(163, 255, 255)';
const COLOR_GREEN = 'rgb(0, 128, 0)';
const COLOR_NONE = 'rgba(0, 0, 0, 0)';

test('native css hmr handles the whole stylesheet lifecycle', async ({
  page,
  fileAction,
}) => {
  const body = page.locator('body');
  await expect(body).toHaveCSS('background-color', COLOR_BLUE);

  // js-only update: the stylesheet must be left untouched
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
  await expect(body).toHaveCSS('background-color', COLOR_BLUE);
  await expect(page.locator('link[data-e2e-initial]')).toHaveCount(1);
  expect(cssRequests).toEqual([]);

  // css-only update: the stylesheet is swapped
  fileAction.updateFile('src/blue.css', (content) =>
    content.replace('rgb(163, 255, 255)', 'rgb(0, 128, 0)'),
  );
  await expect(body).toHaveCSS('background-color', COLOR_GREEN);
  await expect(page.locator('link[rel="stylesheet"]')).toHaveCount(1);

  // css import removed: the stylesheet link is removed
  fileAction.updateFile('src/index.js', (content) =>
    content.replace("import './blue.css';", ''),
  );
  await expect(body).toHaveCSS('background-color', COLOR_NONE);
  await expect(page.locator('link[rel="stylesheet"]')).toHaveCount(0);

  // css import added back: the stylesheet is loaded again
  fileAction.updateFile(
    'src/index.js',
    (content) => `import './blue.css';\n${content}`,
  );
  await expect(body).toHaveCSS('background-color', COLOR_GREEN);
  await expect(page.locator('link[rel="stylesheet"]')).toHaveCount(1);
});
