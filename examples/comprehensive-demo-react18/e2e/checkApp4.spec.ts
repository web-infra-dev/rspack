import { expect, test } from "@playwright/test";

test.describe("Comprehensive Demo App4", () => {
	test("shows svelte greeting", async ({ page }) => {
		await page.goto("http://localhost:3004");
		await expect(page.locator("h1").first()).toHaveText(
			"Hello From Svelte world!"
		);
	});
});
