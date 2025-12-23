import { expect, test } from "@/fixtures";

test("should load shared module in worker successfully", async ({ page }) => {
	await page.waitForSelector("div#worker-result");
	const workerResultText = await page
		.locator("div#worker-result")
		.textContent();
	expect(workerResultText).toContain("Shared value from library");
	expect(workerResultText).not.toContain("Error");
});
