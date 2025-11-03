import { expect, test } from "@/fixtures";

test("should load shared module in worker successfully", async ({ page }) => {
	// Wait for the worker to load the shared module and display the result
	await page.waitForSelector('div#worker-result:has-text("Worker: Shared value")');

	// Check that the shared value from the worker was loaded and displayed
	const workerResultText = await page.locator('#worker-result:has-text("Shared value")').textContent();
	expect(workerResultText).toContain("Shared value from library");
});
