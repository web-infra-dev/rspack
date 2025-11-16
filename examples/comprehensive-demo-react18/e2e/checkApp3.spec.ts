import { expect, test } from "@playwright/test";

const base = "http://localhost:3003";

test.describe("Comprehensive Demo App3", () => {
	test("shows styled button", async ({ page }) => {
		await page.goto(base);

		await expect(page.locator("header").first()).toHaveCSS(
			"background-color",
			"rgb(25, 118, 210)"
		);
		await expect(
			page.getByRole("heading", { name: "Styled Components App" })
		).toBeVisible();

		const button = page.getByRole("button", { name: "💅 Test Button" });
		await expect(button).toBeVisible();
		await expect(button).toHaveCSS("background-color", "rgb(219, 112, 147)");
	});
});
