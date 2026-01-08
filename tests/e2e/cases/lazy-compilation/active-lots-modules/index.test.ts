import { expect, test } from '@/fixtures';

test('should update style', async ({ page }) => {
  const body = await page.locator('body');
  await expect(body).toContainText('All Modules Loaded');
});
