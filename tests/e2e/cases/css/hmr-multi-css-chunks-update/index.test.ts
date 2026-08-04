import { test, expect } from '@/fixtures';

// two async chunks change their stylesheets in the same rebuild: the
// manifest carries multiple miniCss.c entries and both links are swapped
test('stylesheets of several chunks update within one hot update', async ({
  page,
  fileAction,
}) => {
  const root = page.locator('#root');
  const body = page.locator('body');
  await expect(root).toHaveCSS('color', 'rgb(200, 0, 0)');
  await expect(body).toHaveCSS('background-color', 'rgb(163, 255, 255)');
  await expect(page.locator('link[rel="stylesheet"]')).toHaveCount(2);

  fileAction.updateFile('src/a.css', (content) =>
    content.replace('rgb(200, 0, 0)', 'rgb(0, 0, 200)'),
  );
  fileAction.updateFile('src/b.css', (content) =>
    content.replace('rgb(163, 255, 255)', 'rgb(0, 128, 0)'),
  );

  await expect(root).toHaveCSS('color', 'rgb(0, 0, 200)');
  await expect(body).toHaveCSS('background-color', 'rgb(0, 128, 0)');
  await expect(page.locator('link[rel="stylesheet"]')).toHaveCount(2);
});
