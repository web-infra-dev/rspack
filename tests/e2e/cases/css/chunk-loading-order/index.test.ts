import { test, expect } from "@/fixtures";

test("should be red", async ({ page }) => {
	await expect(page.locator("#status")).toHaveText("ok");
	await expect(page.locator("body")).toHaveCSS("color", "rgb(255, 0, 0)");
});
