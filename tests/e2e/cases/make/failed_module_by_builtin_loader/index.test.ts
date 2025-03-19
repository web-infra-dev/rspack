import { expect, test } from "@/fixtures";

test("should compile", async ({ page, fileAction, rspack }) => {
	await expect(page.getByText("index")).toBeVisible();

	fileAction.updateFile("src/index.js", content =>
		content.replace('div.innerText = "index";', 'div.innerText = "er;')
	);

	await page.reload();
	await expect(page.locator("#webpack-dev-server-client-overlay")).toHaveCount(
		1
	);
	let stats = rspack.compiler._lastCompilation
		?.getStats()
		.toJson({ all: false, errors: true });
	expect(stats?.errors?.length).toBe(1);

	fileAction.updateFile("src/index.js", content =>
		content.replace('div.innerText = "er;', 'div.innerText = "index";')
	);
	await page.reload();
	await expect(page.locator("#webpack-dev-server-client-overlay")).toHaveCount(
		0
	);
	await expect(page.getByText("index")).toBeVisible();
	stats = rspack.compiler._lastCompilation
		?.getStats()
		.toJson({ all: false, errors: true });
	expect(stats?.errors?.length).toBe(0);
});
