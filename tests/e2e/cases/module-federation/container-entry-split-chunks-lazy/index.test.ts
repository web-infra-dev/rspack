import { expect, test } from '@/fixtures';

test('should load remote and shared success with lazyCompilation entries', async ({
  page,
}) => {
  // Same self-referential MF setup as container-entry-split-chunks, but with
  // `lazyCompilation: { entries: true }`. The entry is activated over HMR, which needs a
  // single `self["rspackHotUpdate"]`; `optimization.runtimeChunk: 'single'` (see
  // rspack.config.js) keeps one runtime so remoteEntry and main don't clobber that global
  // and activation works. See #12443.
  await page.waitForSelector('p:has-text("Remote Component")');
  const RemoteComponentCount = await page.getByText('Remote Component').count();
  expect(RemoteComponentCount).toBe(1);
});
