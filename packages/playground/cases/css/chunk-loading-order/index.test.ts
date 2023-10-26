import { test, expect } from "@/fixtures";

test("should be red", async ({ page, rspack }) => {
	await rspack.waitUntil(async function () {
		return (await page.textContent("#status")) === "ok";
	});
	await expect(page.locator("body")).toHaveCSS("color", "rgb(255, 0, 0)");
});
