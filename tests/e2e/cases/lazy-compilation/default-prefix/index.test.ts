import { expect, test } from '@/fixtures';

test('should use default prefix for lazy compilation', async ({ page }) => {
  // Click the button that triggers dynamic import
  await page.waitForSelector('button:has-text("Click me")');
  await page.getByText('Click me').click();

  // Wait for the component to appear with a more reliable wait
  await page.waitForSelector('div:has-text("Component")');

  // Check that the component was loaded and displayed
  const component_count = await page.getByText('Component').count();
  expect(component_count).toBe(1);
});
