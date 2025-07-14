import { expect, test } from "@/fixtures";

async function expect_content(page: any, data: string) {
	await expect(async () => {
		await page.reload();
		expect(await page.locator("div").innerText()).toBe(data);
	}).toPass();
}

test("should compile", async ({ page, fileAction, rspack }) => {
	await expect_content(page, "2");

	fileAction.updateFile("file.js", content => content.replace("1", "2"));

	await expect_content(page, "4");

	fileAction.updateFile("file.js", content => content.replace("2", "3"));

	await expect_content(page, "6");
});
