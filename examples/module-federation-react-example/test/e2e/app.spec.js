import { expect, test } from "./setup.js";

test.describe("Module Federation React App", () => {
	test.beforeEach(async ({ page }) => {
		await page.goto("/");
	});

	test("should load the main application", async ({ page }) => {
		// Check that the app loads with the expected title
		await expect(page).toHaveTitle(/Module Federation React/);

		// Check for main navigation elements
		await expect(page.locator("text=MF React App")).toBeVisible();
		await expect(
			page.locator("text=Module Federation React Demo")
		).toBeVisible();
	});

	test("should navigate between pages", async ({ page }) => {
		// Test navigation to Dashboard
		await page.click("text=Dashboard");
		await expect(page.locator('h2:has-text("Dashboard")')).toBeVisible();

		// Test navigation to Analytics
		await page.click("text=Analytics");
		await expect(page.locator('h2:has-text("Analytics")')).toBeVisible();

		// Test navigation to Users
		await page.click("text=Users");
		await expect(page.locator('h2:has-text("Users")')).toBeVisible();

		// Test navigation to Remote Components
		await page.click("text=Remote Components");
		await expect(
			page.locator('h2:has-text("Remote Components Showcase")')
		).toBeVisible();

		// Test navigation to Settings
		await page.click("text=Settings");
		await expect(page.locator('h2:has-text("Settings")')).toBeVisible();
	});

	test("should display dashboard statistics", async ({ page }) => {
		await page.click("text=Dashboard");

		// Wait for statistics to load
		await page.waitForSelector('[data-testid="stat-card"], .ant-statistic', {
			timeout: 10000
		});

		// Check for statistic cards
		await expect(page.locator("text=Total Users")).toBeVisible();
		await expect(page.locator("text=Active Users")).toBeVisible();
		await expect(page.locator("text=Revenue")).toBeVisible();
		await expect(page.locator("text=Growth")).toBeVisible();

		// Check for recent activity
		await expect(page.locator("text=Recent Activity")).toBeVisible();
	});

	test("should display analytics charts", async ({ page }) => {
		await page.click("text=Analytics");

		// Wait for charts to load
		await page.waitForSelector("canvas", { timeout: 15000 });

		// Check for chart containers
		await expect(page.locator("text=Revenue Trend")).toBeVisible();
		await expect(page.locator("text=User Growth")).toBeVisible();
		await expect(page.locator("text=Device Categories")).toBeVisible();

		// Verify charts are rendered (canvas elements)
		const charts = page.locator("canvas");
		await expect(charts).toHaveCount({ min: 3 });
	});

	test("should display users table", async ({ page }) => {
		await page.click("text=Users");

		// Wait for table to load
		await page.waitForSelector(".ant-table", { timeout: 10000 });

		// Check table headers
		await expect(page.locator('th:has-text("Name")')).toBeVisible();
		await expect(page.locator('th:has-text("Email")')).toBeVisible();
		await expect(page.locator('th:has-text("Role")')).toBeVisible();
		await expect(page.locator('th:has-text("Status")')).toBeVisible();

		// Check for user data
		await expect(page.locator("text=John Doe")).toBeVisible();
		await expect(page.locator("text=jane@example.com")).toBeVisible();
	});
});
