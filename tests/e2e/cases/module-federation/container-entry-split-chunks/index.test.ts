import { expect, test } from '@/fixtures';

test('should load remote and shared success', async ({ page }) => {
  // Self-referential MF build: remoteEntry and main run on the same page. lazyCompilation
  // `entries` activates the entry over HMR, which requires a single `self["rspackHotUpdate"]`.
  // Without a shared runtime the remoteEntry and main runtimes clobber that global and the
  // activation update is lost; `optimization.runtimeChunk: 'single'` (see rspack.config.js)
  // keeps one runtime so activation works. See #12443.
  await page.waitForSelector('p:has-text("Remote Component")');
  const RemoteComponentCount = await page.getByText('Remote Component').count();
  expect(RemoteComponentCount).toBe(1);
});
