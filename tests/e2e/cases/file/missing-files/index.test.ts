import { test, expect } from "@/fixtures";

test("missing files should be able to recover if being added back", async ({
	page,
	fileAction
}) => {
	const overlay = page.frameLocator("#webpack-dev-server-client-overlay");
	await expect(
		overlay.getByText("Can't resolve './missing-file-1'")
	).toBeVisible();
	await expect(
		overlay.getByText("Can't resolve './missing-file-2'")
	).toBeVisible();

	fileAction.updateFile(
		"src/missing-file-1.js",
		() => "export const a = 'missing-file-1'"
	);

	fileAction.updateFile(
		"src/missing-file-2.js",
		() => "export const b = 'missing-file-2'"
	);

	await expect(page.locator("#missing-file-1")).toHaveText("missing-file-1");
	await expect(page.locator("#missing-file-2")).toHaveText("missing-file-2");

	fileAction.deleteFile("src/missing-file-1.js");

	await expect(
		overlay.getByText("Can't resolve './missing-file-1'")
	).toBeVisible();

	fileAction.updateFile(
		"src/missing-file-1.js",
		() => "export const a = 'missing-file-1'"
	);

	await expect(page.locator("#missing-file-1")).toHaveText("missing-file-1");
});
