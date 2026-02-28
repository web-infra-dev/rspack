import { expect, test } from '@/fixtures';

test('should activate and compile lots of modules with long names', async ({
  page,
}) => {
  const body = await page.locator('body');
  await expect(body).toContainText('All Modules Loaded');
});
