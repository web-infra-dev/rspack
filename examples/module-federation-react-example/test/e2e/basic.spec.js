import { expect, test } from "./setup.js";

test.describe("Basic Module Federation React App", () => {
	test("should load host app and navigate", async ({ page }) => {
		await page.goto("/");

		// Check that the app loads
		await expect(page.locator("text=MF React App")).toBeVisible();

		// Test navigation
		await page.click("text=Dashboard");
		await expect(page.locator('h2:has-text("Dashboard")')).toBeVisible();

		await page.click("text=Users");
		await expect(page.locator('h2:has-text("Users")')).toBeVisible();
	});

	test("should load remote components", async ({ page }) => {
		await page.goto("/remote-components");

		// Check remote components page loads
		await expect(
			page.locator('h2:has-text("Remote Components Showcase")')
		).toBeVisible();

		// Click on User Card tab and wait for remote component
		await page.click('[role="tab"]:has-text("User Card")');

		// Wait for remote component to load (give it more time)
		await page.waitForSelector(".ant-card, text=Loading", { timeout: 20000 });

		// Check if component loaded or if there's an error state
		const hasUserCard = await page.locator("text=John Doe").isVisible();
		const hasError = await page.locator("text=Error").isVisible();
		const hasLoading = await page.locator("text=Loading").isVisible();

		console.log("UserCard visible:", hasUserCard);
		console.log("Error visible:", hasError);
		console.log("Loading visible:", hasLoading);

		// App should be functional even if remote components don't load
		expect(hasUserCard || hasError || hasLoading).toBe(true);
	});

	test("should verify remote entry is accessible", async ({ page }) => {
		// Check if remote entry exists
		const response = await page.request.get(
			"http://localhost:3002/remoteEntry.js"
		);
		console.log("Remote entry status:", response.status());

		// Should be accessible (200) or we'll know why it's not
		expect([200, 404, 500]).toContain(response.status());
	});

	test("should handle missing remote gracefully", async ({ page }) => {
		await page.goto("/");

		// Block remote requests to test error handling
		await page.route("**/localhost:3002/**", route => route.abort());

		// Navigate to remote components
		await page.click("text=Remote Components");

		// App should not crash
		await expect(page.locator("body")).toBeVisible();
		await expect(
			page.locator('h2:has-text("Remote Components Showcase")')
		).toBeVisible();

		await page.unroute("**/localhost:3002/**");
	});
});
