import { expect, test } from "@/fixtures";

test("should load success", async ({ page }) => {
	await page.getByText("Click me").click();
	await expect(page).toHaveURL(/success/);
});
