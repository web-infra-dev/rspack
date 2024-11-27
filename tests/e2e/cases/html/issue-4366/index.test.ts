import { test, expect } from "@/fixtures";

test("html should refresh after reload", async ({ page, fileAction }) => {
	await expect(page).toHaveTitle("123");
	fileAction.updateFile("./src/index.html", content =>
		content.replace("123", "456")
	);
	await expect(async () => {
		await page.reload();
		expect(await page.title()).toBe("456");
	}).toPass();
});
