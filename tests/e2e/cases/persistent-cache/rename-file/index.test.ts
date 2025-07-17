import { expect, test } from "@/fixtures";

async function expect_content(page: any, data: string) {
	await expect(async () => {
		await page.reload();
		expect(await page.locator("div").innerText()).toBe(data);
	}).toPass();
}

test("should compile", async ({ page, fileAction, rspack }) => {
	await expect_content(page, "ab");
	await rspack.stop();
	await new Promise(res => {
		setTimeout(res, 500);
	});
	// switch a, b
	fileAction.renameFile("a.js", "temp.js");
	fileAction.renameFile("b.js", "a.js");
	fileAction.renameFile("temp.js", "b.js");

	await rspack.start();
	await expect_content(page, "ba");
});
