import { test, expect } from '@/fixtures';

const COLOR_BLUE = 'rgb(163, 255, 255)';
const COLOR_NONE = 'rgba(0, 0, 0, 0)';

test('adding the very first native css import loads the stylesheet via hmr', async ({
  page,
  fileAction,
}) => {
  await expect(page.locator('body')).toHaveCSS('background-color', COLOR_NONE);
  await expect(page.locator('link[rel="stylesheet"]')).toHaveCount(0);

  fileAction.updateFile(
    'src/index.js',
    (content) => `import './blue.css';\n${content}`,
  );

  await expect(page.locator('body')).toHaveCSS('background-color', COLOR_BLUE);
  await expect(page.locator('link[rel="stylesheet"]')).toHaveCount(1);
});
