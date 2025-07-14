import { expect, test } from "@/fixtures";

test.skip("should compile", async ({ page, fileAction, rspack }) => {
	await expect(page.getByText("2")).toBeVisible();

	fileAction.updateFile("file.js", content => content.replace("1", "2"));

	await page.reload();
	await expect(page.getByText("4")).toBeVisible();

	fileAction.updateFile("file.js", content => content.replace("2", "3"));
	await page.reload();
	await expect(page.getByText("6")).toBeVisible();
});
