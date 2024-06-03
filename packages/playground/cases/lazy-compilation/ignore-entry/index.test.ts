import { expect, test } from "@/fixtures";

test("should load success", async ({ page }) => {
	await page.waitForTimeout(1000);
	const body = await page.$("body");
	expect(await body!.innerText()).toBe("hello world");
});
