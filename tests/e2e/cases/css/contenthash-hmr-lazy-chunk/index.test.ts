import { test, expect } from '@/fixtures';

const COLOR_BLUE = 'rgb(10, 20, 30)';
const COLOR_RED = 'rgb(120, 0, 0)';

// A chunk that hasn't been imported yet has no <link> for the HMR handler to swap;
// the first import() afterwards must still request the post-edit hashed filename,
// which relies on the chunk filename runtime module (`miniCssF`) having been re-applied
// by the update. This is the scenario rspack#6869 itself is about (lazy components),
// so it's guarded separately from the always-imported one.
test('should apply a CSS edit made before the chunk owning it is ever imported', async ({
  page,
  fileAction,
}) => {
  // Unlike the sibling contenthash-hmr case (which mutates an already-loaded
  // stylesheet and lets toHaveCSS retry), this one must wait for the update to be fully
  // *applied* before triggering the first import(): if #load runs while `miniCssF` is
  // still the pre-edit one, the chunk loads the stale href and the failure is
  // permanent (the chunk is marked errored and never re-fetched). "[HMR] App is up to
  // date." is emitted by this repo's own hot client (packages/rspack/hot/dev-server.js)
  // exactly on apply completion, so it's the correct - and repo-owned, not third-party -
  // signal to gate on.
  const hmrSettled = page.waitForEvent('console', {
    predicate: (msg) => msg.text().includes('up to date'),
  });

  fileAction.updateFile('src/lazy.css', (content) =>
    content.replace(COLOR_BLUE, COLOR_RED),
  );

  await hmrSettled;

  await page.click('#load');

  await expect(page.locator('body')).toHaveCSS('background-color', COLOR_RED);
  await expect(page.locator('link[rel="stylesheet"]')).toHaveCount(1);
});
