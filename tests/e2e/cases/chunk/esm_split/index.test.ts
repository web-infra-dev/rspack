import { test, expect } from '@/fixtures';

test('should load split chunk while enable esm chunk', async ({ page }) => {
  await expect(page.locator('p')).toHaveText('Loaded');
});
