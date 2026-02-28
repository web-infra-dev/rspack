import { test, expect } from '@/fixtures';

test('should render correct style', async ({ page }) => {
  await expect(page.locator('body')).toHaveCSS(
    'background-color',
    'rgb(0, 0, 255)',
  );
});
