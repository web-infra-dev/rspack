import { test, expect } from '@/fixtures';

test('@rspack/browser should bundle react app successfully', async ({
  page,
}) => {
  await expect(page.locator('#output')).toContainText('console.log');
  await expect(page.locator('#output')).toContainText('rspack');
});
