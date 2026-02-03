import { test, expect } from '@/fixtures';

test('@rspack/browser should bundle react app successfully', async ({
  page,
}) => {
  // There should be a long bundle result
  await expect
    .poll(async () => {
      const text = await page.locator('#output').innerText();
      return text.length;
    })
    .toBeGreaterThan(300);
});
