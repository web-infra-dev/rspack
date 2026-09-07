import { test, expect } from '@/fixtures';

// the async chunk is never imported by the page, so a css edit for it must
// not inject its stylesheet into the current document
test('editing css of an unloaded async chunk does not leak into the page', async ({
  page,
  fileAction,
}) => {
  await expect(page.locator('#root')).toHaveText('step-0');
  await expect(page.locator('link[rel="stylesheet"]')).toHaveCount(0);

  fileAction.updateFile('src/feature.css', (content) =>
    content.replace('rgb(200, 0, 200)', 'rgb(0, 200, 200)'),
  );

  // give the hot update time to arrive and (wrongly) apply
  await page.waitForTimeout(1500);
  await expect(page.locator('link[rel="stylesheet"]')).toHaveCount(0);
  await expect(page.locator('body')).not.toHaveCSS(
    'background-color',
    'rgb(0, 200, 200)',
  );
});
