import { test, expect } from "@/fixtures";

test("render should work", async ({ page }) => {
	expect(await page.textContent("button")).toBe("+");
});

test("worker should work", async ({ page, fileAction, rspack }) => {
	expect(await page.textContent("h1")).toBe("0");
	await page.click("button");
	expect(await page.textContent("h1")).toBe("1");
	await page.click("button");
	expect(await page.textContent("h1")).toBe("2");
});
