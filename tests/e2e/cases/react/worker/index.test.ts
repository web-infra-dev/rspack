import { test, expect } from '@/fixtures';

test('should successfully render the page', async ({ page }) => {
  expect(await page.textContent('button')).toBe('+');
  expect(await page.textContent('h1')).toBe('0');
});

// test("worker should work", async ({ page, fileAction, rspack }) => {
// 	await page.click("button");
// 	expect(await page.textContent("h1")).toBe("1");
// 	await page.click("button");
// 	expect(await page.textContent("h1")).toBe("2");
// });
