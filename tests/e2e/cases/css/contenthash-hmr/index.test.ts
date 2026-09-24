import { test, expect } from '@/fixtures';

const COLOR_BLUE = 'rgb(10, 20, 30)';
const COLOR_RED = 'rgb(120, 0, 0)';
const COLOR_GREEN = 'rgb(0, 90, 0)';

// With a hashed extract-css filename, the runtime's `miniCssF` is only refreshed once an
// update is applied, so the HMR handler downloading that update resolved the pre-edit
// href and every edit got silently discarded. See rspack#6869.
test('should update the page and keep a single stylesheet link when the extracted CSS filename is hashed', async ({
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

  // Not just an off-by-one: the second edit in a row must also apply.
  fileAction.updateFile('src/index.css', (content) =>
    content.replace(COLOR_RED, COLOR_GREEN),
  );
  await expect(page.locator('body')).toHaveCSS('background-color', COLOR_GREEN);
  await expect(links).toHaveCount(1);
});
