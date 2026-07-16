import { test, expect } from '@/fixtures';

const COLOR_BLUE = 'rgb(163, 255, 255)';
const COLOR_NONE = 'rgba(0, 0, 0, 0)';

// removing and re-adding the only css import within one session: the
// re-added stylesheet has no old link and the stale browser cache from
// before the removal must be bypassed
test('a stylesheet removed and re-added by hmr comes back', async ({
  page,
  fileAction,
}) => {
  const body = page.locator('body');
  await expect(body).toHaveCSS('background-color', COLOR_BLUE);
  await expect(page.locator('link[rel="stylesheet"]')).toHaveCount(1);

  fileAction.updateFile('src/index.js', (content) =>
    content.replace(`import './blue.css';\n`, ''),
  );

  await expect(body).toHaveCSS('background-color', COLOR_NONE);
  await expect(page.locator('link[rel="stylesheet"]')).toHaveCount(0);

  fileAction.updateFile(
    'src/index.js',
    (content) => `import './blue.css';\n${content}`,
  );

  await expect(body).toHaveCSS('background-color', COLOR_BLUE);
  await expect(page.locator('link[rel="stylesheet"]')).toHaveCount(1);
});
