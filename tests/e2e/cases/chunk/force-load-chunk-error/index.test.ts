import { expect, test } from '@/fixtures';

test('should reload when a force-loaded chunk fails to load', async ({
  page,
  fileAction,
}) => {
  await expect(page).toHaveTitle('v1');

  // Dropping `hooks` renames page's chunk from `src_page_js-src_policy_js` to
  // `src_page_js`, which the update force-loads after applying.
  let aborted = false;
  await page.route('**/src_page_js.js', (route) => {
    if (aborted) {
      return route.continue();
    }
    aborted = true;
    return route.abort();
  });
  const reloaded = page.waitForEvent('load');
  fileAction.updateFile(
    'src/page.js',
    () => `document.title = 'v2';\nimport.meta.webpackHot.accept();\n`,
  );
  await reloaded;
  expect(aborted).toBe(true);
  await expect(page).toHaveTitle('v2');

  fileAction.updateFile(
    'src/page.js',
    () => `document.title = 'v3';\nimport.meta.webpackHot.accept();\n`,
  );
  await expect(page).toHaveTitle('v3');
});
