import { expect, test } from "@/fixtures";

test("should compile", async ({ page, fileAction, rspack }) => {
	await expect(page.locator("#index1")).toBeVisible();
	await expect(page.locator("#index2")).toBeVisible();

	fileAction.deleteFile("src/index2.js");
	fileAction.updateFile("src/index1.js", content =>
		content.replace(
			'div.innerText = "index1";',
			'div.innerText = "index1 updated";'
		)
	);

	await expect(async () => {
		await page.reload();
		expect(await page.locator("#index1").innerText()).toBe("index1 updated");
	}).toPass();
	await expect(page.locator("#index2")).toHaveCount(0);
	await expect(page.locator("#webpack-dev-server-client-overlay")).toHaveCount(
		0
	);

	const stats = rspack.compiler._lastCompilation
		?.getStats()
		.toJson({ all: false, errors: true });
	expect(stats?.errors?.length).toBe(0);
});
