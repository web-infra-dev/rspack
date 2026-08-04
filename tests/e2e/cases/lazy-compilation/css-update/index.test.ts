import { expect, test } from '@/fixtures';

test('should update style', async ({ page }) => {
  const body = page.locator('body');
  // first time enter the page, style is red
  await expect(body).toHaveCSS('background-color', 'rgb(255, 0, 0)');

  // second time enter the page, this time brings query,
  // trigger lazy-compile
  const url = await page.evaluate(() => window.location.href);
  await page.goto(`${url}?1`);
  await expect(body).toHaveCSS('background-color', 'rgb(0, 0, 255)');
});
