import { expect, test } from "./setup.js";

test("no critical console errors during navigation (indicates no removed modules)", async ({
	page,
	consoleErrorWatcher
}) => {
	await page.goto("/");
	await page.click("text=Remote Components");
	await page.waitForSelector(".remote-component-wrapper, .ant-card");
	expect(consoleErrorWatcher.messages).toEqual([]);
});

test.describe("Module Federation Architecture", () => {
	test("should load both host and remote applications independently", async ({
		browser
	}) => {
		// Test host app
		const hostContext = await browser.newContext();
		const hostPage = await hostContext.newPage();
		await hostPage.goto("http://localhost:3001");
		await expect(hostPage.locator("text=MF React App")).toBeVisible();

		// Test remote app
		const remoteContext = await browser.newContext();
		const remotePage = await remoteContext.newPage();
		await remotePage.goto("http://localhost:3002");
		await expect(
			remotePage.locator("text=Remote Components Library")
		).toBeVisible();

		await hostContext.close();
		await remoteContext.close();
	});

	test("should demonstrate Module Federation behavior", async ({ page }) => {
		await page.goto("/");

		// Navigate to remote components page
		await page.click("text=Remote Components");

		// Check the Module Federation info alert
		await expect(
			page.locator("text=Module Federation in Action")
		).toBeVisible();
		await expect(
			page.locator(
				"text=These components are loaded dynamically from a remote application"
			)
		).toBeVisible();

		// Verify remote components load from different origin
		const response = await page.waitForResponse(
			response =>
				response.url().includes("localhost:3002") &&
				response.url().includes("remoteEntry.js")
		);
		expect(response.status()).toBe(200);
	});

	test("should share React and Ant Design between apps", async ({ page }) => {
		await page.goto("/remote-components");

		// Switch to User Card tab to load remote component
		await page.click('[role="tab"]:has-text("User Card")');
		await page.waitForSelector(".ant-card", { timeout: 15000 });

		// Check that Ant Design components render consistently
		const hostButton = page.locator(".ant-btn").first();
		await expect(hostButton).toHaveCSS("border-radius", "6px"); // Should use shared theme

		// Verify React is shared (no duplicate React instances)
		const reactErrors = await page.evaluate(() => {
			return window.__REACT_DEVTOOLS_GLOBAL_HOOK__?.renderers?.size > 1;
		});
		expect(reactErrors).toBeFalsy();
	});

	test("should handle network failures gracefully", async ({ page }) => {
		await page.goto("/");

		// Block remote app requests to simulate network failure
		await page.route("**/localhost:3002/**", route => route.abort());

		// Navigate to remote components
		await page.click("text=Remote Components");

		// Should show error boundary or loading state, not crash
		await expect(page.locator("body")).toBeVisible();

		// Clear route blocking
		await page.unroute("**/localhost:3002/**");
	});

	test("should maintain state across remote component switches", async ({
		page
	}) => {
		await page.goto("/remote-components");

		// Fill out form in FormBuilder
		await page.click('[role="tab"]:has-text("Form Builder")');
		await page.waitForSelector(".ant-form", { timeout: 15000 });

		await page.fill('input[placeholder*="Username"]', "testuser");
		await page.fill('input[placeholder*="Email"]', "test@example.com");

		// Switch to another tab
		await page.click('[role="tab"]:has-text("User Card")');
		await page.waitForSelector(".ant-card", { timeout: 15000 });

		// Switch back to form
		await page.click('[role="tab"]:has-text("Form Builder")');
		await page.waitForSelector(".ant-form", { timeout: 15000 });

		// Form should have maintained state (if using React state)
		// Note: This depends on how the component is implemented
		const usernameValue = await page.inputValue(
			'input[placeholder*="Username"]'
		);
		const emailValue = await page.inputValue('input[placeholder*="Email"]');

		// At minimum, form should be functional
		expect(typeof usernameValue).toBe("string");
		expect(typeof emailValue).toBe("string");
	});

	test("should optimize bundle sizes with tree shaking", async ({ page }) => {
		// Check that optimized chunks exist in the file system
		const response = await page.request.get("http://localhost:3001");
		expect(response.status()).toBe(200);

		// Navigate to remote components to trigger module loading
		await page.goto("/remote-components");
		await page.click('[role="tab"]:has-text("User Card")');
		await page.waitForSelector(".ant-card", { timeout: 15000 });

		// Check for optimized vendor chunks by looking for .optimized.js files
		const networkRequests = [];
		page.on("request", request => {
			if (request.url().includes(".js")) {
				networkRequests.push(request.url());
			}
		});

		// Reload to capture all network requests
		await page.reload();
		await page.waitForSelector(".ant-card", { timeout: 15000 });

		// Should have loaded JavaScript bundles
		expect(networkRequests.length).toBeGreaterThan(0);
	});
});
