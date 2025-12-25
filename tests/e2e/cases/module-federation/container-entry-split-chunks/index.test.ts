import { expect, test } from '@/fixtures';

test('should load remote and shared success', async ({ page }) => {
  // page is correct displayed, we don't test hmr for now since the remoteEntry and main
  // are running at the same page, so there `self["webpackHotUpdate"]` is conflicted, hmr
  // is expected to be broken
  await page.waitForSelector('p:has-text("Remote Component")');
  const RemoteComponentCount = await page.getByText('Remote Component').count();
  expect(RemoteComponentCount).toBe(1);
});
