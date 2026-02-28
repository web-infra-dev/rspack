import { expect, test } from '@/fixtures';

test('should load remote and shared success', async ({ page }) => {
  // Reload for "should have __webpack_require__.f.consumes" error, normally hmr will automatically reload
  // but the automatically reload sometimes is flaky for our e2e test, so we manually reload before anything.
  // To fix that we need to embed mf runtime as runtime module instead of entry module, for now we reload the
  // page to make e2e more reliable
  await page.reload();
  await page.waitForSelector('button:has-text("Click me")');

  // trigger lazy-compile
  await page.getByText('Click me').click();

  // Wait for the component to appear with a more reliable wait
  await page.waitForSelector('div:has-text("RemoteComponent")');

  // Check that the component was loaded and displayed
  const RemoteComponentCount = await page.getByText('RemoteComponent').count();
  expect(RemoteComponentCount).toBe(1);

  // Wait for the component to appear with a more reliable wait
  await page.waitForSelector('div:has-text("SharedReact")');
  // Check that the shared component was loaded and displayed
  const SharedReactCount = await page.getByText('SharedReact').count();
  expect(SharedReactCount).toBe(1);
});
