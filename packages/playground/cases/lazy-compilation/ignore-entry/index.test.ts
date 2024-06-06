import { expect, test } from "@/fixtures";

test("should load success", async ({ page }) => {
	await page.getByText("Click me").click();
	await expect(page).toHaveURL(/success/);
	const body = await page.$("body");
	const backgroundColor = await body!.evaluate(
		el => window.getComputedStyle(el).backgroundColor
	);
	expect(backgroundColor, "blue");
});
