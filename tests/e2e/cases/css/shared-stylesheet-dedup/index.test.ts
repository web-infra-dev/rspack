import { test, expect } from '@/fixtures';

const COLOR_BLUE = 'rgb(10, 20, 30)';
const COLOR_RED = 'rgb(120, 0, 0)';
const COLOR_GREEN = 'rgb(0, 90, 0)';

// The hot-update lists both the `style` and `main` chunks while the fixed
// `filename` maps them to one stylesheet. Without de-duplication the handler
// re-fetched it once per chunk and leaked one <link> per update.
test('should keep a single stylesheet link when several updated chunks share it', async ({
  page,
  fileAction,
}) => {
  const links = page.locator('link[rel="stylesheet"]');
  await expect(page.locator('body')).toHaveCSS('background-color', COLOR_BLUE);
  await expect(links).toHaveCount(1);

  fileAction.updateFile('src/index.css', (content) =>
    content.replace(COLOR_BLUE, COLOR_RED),
  );
  await expect(page.locator('body')).toHaveCSS('background-color', COLOR_RED);
  await expect(links).toHaveCount(1);

  fileAction.updateFile('src/index.css', (content) =>
    content.replace(COLOR_RED, COLOR_GREEN),
  );
  await expect(page.locator('body')).toHaveCSS('background-color', COLOR_GREEN);
  await expect(links).toHaveCount(1);
});
