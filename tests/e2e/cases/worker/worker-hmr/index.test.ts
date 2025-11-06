import { expect, test } from "@/fixtures";

test("should load shared module in worker successfully", async ({ page, fileAction }) => {
	await page.waitForSelector('div#worker-result:has-text("__PLACEHOLDER__")');
	const workerResultText = await page.locator('div#worker-result').textContent();
	expect(workerResultText).toContain("__PLACEHOLDER__");

	fileAction.updateFile("src/worker.js", content =>
		content.replace("__PLACEHOLDER__", "__EDITED__")
	);
	await page.waitForSelector('div#worker-result:has-text("__EDITED__")');
	const editedWorkerResultText = await page.locator('div#worker-result').textContent();
	expect(editedWorkerResultText).toContain("__EDITED__");
});
