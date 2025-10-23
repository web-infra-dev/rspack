import { test, expect } from "@/fixtures";

test("should run while split chunk enabled", async ({ page }) => {
	await expect(page.locator("p")).toHaveText("Loaded");
});
