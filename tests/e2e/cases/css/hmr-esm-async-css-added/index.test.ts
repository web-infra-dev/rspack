import { test, expect } from '@/fixtures';

const COLOR_BLUE = 'rgb(163, 255, 255)';

// with `chunkLoading: 'import'` loaded chunks are recorded under the module
// hmr state, not the jsonp one. The async chunk is loaded at startup, gains
// its first stylesheet through the edit, and self-accepts so nothing
// re-runs the dynamic import: the stylesheet can only arrive through the
// hmr handler.
test('a loaded esm async chunk gaining css applies the stylesheet', async ({
  page,
  fileAction,
}) => {
  await expect(page.locator('#root')).toHaveText('feature-v1');
  await expect(page.locator('link[rel="stylesheet"]')).toHaveCount(0);

  fileAction.updateFile('src/feature.js', (content) =>
    `import './feature.css';\n${content}`.replace('feature-v1', 'feature-v2'),
  );

  // the js side of the update applies...
  await expect(page.locator('#root')).toHaveText('feature-v2');
  // ...and so must the stylesheet the async chunk just gained
  await expect(page.locator('body')).toHaveCSS('background-color', COLOR_BLUE);
});
