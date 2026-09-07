import { test, expect } from '@/fixtures';

const COLOR_BLUE = 'rgb(163, 255, 255)';
const COLOR_GREEN = 'rgb(0, 128, 0)';
const COLOR_NONE = 'rgba(0, 0, 0, 0)';

test('async chunk stylesheet is swapped and removed via hmr', async ({
  page,
  fileAction,
}) => {
  const body = page.locator('body');
  await expect(page.locator('#root')).toHaveText('feature loaded');
  await expect(body).toHaveCSS('background-color', COLOR_BLUE);

  // css of the lazily loaded chunk changes
  fileAction.updateFile('src/feature.css', (content) =>
    content.replace('rgb(163, 255, 255)', 'rgb(0, 128, 0)'),
  );
  await expect(body).toHaveCSS('background-color', COLOR_GREEN);
  await expect(page.locator('link[href*="feature.css"]')).toHaveCount(1);

  // the chunk loses its css: the stylesheet link must go away
  fileAction.updateFile('src/feature.js', (content) =>
    content.replace("import './feature.css';", ''),
  );
  await expect(body).toHaveCSS('background-color', COLOR_NONE);
  await expect(page.locator('link[href*="feature.css"]')).toHaveCount(0);
});
