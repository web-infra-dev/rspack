import { test, expect } from '@/fixtures';

const COLOR_BLUE = 'rgb(163, 255, 255)';
const COLOR_GREEN = 'rgb(0, 128, 0)';

// with `runtime: false` there is no hmrC.miniCss handler, so css edits can
// only reach the page through the loader-level cssReload path
test('css edits still apply when the css runtime is disabled', async ({
  page,
  fileAction,
}) => {
  const body = page.locator('body');
  await expect(body).toHaveCSS('background-color', COLOR_BLUE);

  fileAction.updateFile('src/style.css', (content) =>
    content.replace('rgb(163, 255, 255)', 'rgb(0, 128, 0)'),
  );

  await expect(body).toHaveCSS('background-color', COLOR_GREEN);
});
