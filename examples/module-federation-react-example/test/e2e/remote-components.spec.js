import { expect, test } from "./setup.js";

test.describe.configure({ mode: "serial" });

test.describe("Remote Components Integration", () => {
	test.beforeEach(async ({ page }) => {
		// Debug listeners
		page.on("console", msg => {
			if (msg.type() === "error")
				console.log("Browser console error:", msg.text());
		});
		page.on("pageerror", err =>
			console.log("Browser page error:", err.message)
		);

		// Navigate via app shell, then fall back to direct route if needed
		await page.goto("/");
		await page
			.waitForSelector(".app-layout", { timeout: 15000 })
			.catch(() => {});

		try {
			const menuItem = page
				.getByRole("menuitem", { name: /Remote Components/i })
				.first();
			await menuItem.waitFor({ state: "visible", timeout: 5000 });
			await menuItem.click();
			await page.waitForURL("**/remote-components", { timeout: 10000 });
		} catch {
			await page.goto("/remote-components");
		}

		// Final ensure content present
		await page.waitForSelector('h2:has-text("Remote Components Showcase")', {
			timeout: 15000
		});
	});

	test("should verify remote entry is accessible", async ({
		page,
		moduleFederation
	}) => {
		const isLoaded = await moduleFederation.isRemoteEntryLoaded();
		console.log("remoteEntry.js HTTP 200:", isLoaded);
		const probe = await page.evaluate(() => {
			const r = window.remote;
			return {
				hasRemote: !!r,
				type: r ? typeof r : "undefined",
				hasGet: r ? typeof r.get === "function" : false,
				hasInit: r ? typeof r.init === "function" : false,
				keys: r ? Object.keys(r) : []
			};
		});
		console.log("Remote container probe:", probe);
		expect(isLoaded).toBe(true);
		expect(probe.hasRemote).toBe(true);
		expect(probe.hasGet).toBe(true);
		expect(probe.hasInit).toBe(true);
	});

	test("should load remote components showcase page", async ({ page }) => {
		await expect(
			page.locator('h2:has-text("Remote Components Showcase")')
		).toBeVisible();
		await expect(
			page.locator("text=Module Federation in Action")
		).toBeVisible();
	});

	test("should display UserCard component from remote", async ({
		page,
		moduleFederation
	}) => {
		// Click on User Card tab
		await moduleFederation.switchToRemoteTab("User Card");

		// Wait for remote component to load
		await moduleFederation.waitForRemoteComponent(
			'.ant-card:has-text("User Profile Card Component")'
		);

		// Check UserCard content
		await expect(page.locator("text=John Doe")).toBeVisible();
		await expect(page.locator("text=john.doe@example.com")).toBeVisible();
		await expect(page.locator("text=Senior Developer")).toBeVisible();
		await expect(page.locator("text=Engineering Department")).toBeVisible();
	});

	test("should display DataTable component from remote", async ({
		page,
		moduleFederation
	}) => {
		// Click on Data Table tab
		await moduleFederation.switchToRemoteTab("Data Table");

		// Wait for remote component to load
		await page.waitForSelector(".ant-table", { timeout: 15000 });

		// Check DataTable headers
		await expect(page.locator('th:has-text("Product")')).toBeVisible();
		await expect(page.locator('th:has-text("Category")')).toBeVisible();
		await expect(page.locator('th:has-text("Price")')).toBeVisible();
		await expect(page.locator('th:has-text("Stock")')).toBeVisible();
		await expect(page.locator('th:has-text("Status")')).toBeVisible();

		// Check for sample data
		await expect(page.locator("text=iPhone 14 Pro")).toBeVisible();
		await expect(page.locator("text=MacBook Pro M3")).toBeVisible();
	});

	test("should display ChartWidget component from remote", async ({
		page,
		moduleFederation
	}) => {
		// Click on Charts tab
		await moduleFederation.switchToRemoteTab("Charts");

		// Wait for remote component to load
		await page.waitForSelector("canvas", { timeout: 15000 });

		// Check for chart canvas
		const chart = page.locator("canvas").first();
		await expect(chart).toBeVisible();

		// Verify chart container
		await expect(page.locator("text=Chart Widgets")).toBeVisible();
	});

	test("should display FormBuilder component from remote", async ({
		page,
		moduleFederation
	}) => {
		// Click on Form Builder tab
		await moduleFederation.switchToRemoteTab("Form Builder");

		// Wait for remote component to load
		await page.waitForSelector(".ant-form", { timeout: 15000 });

		// Check form fields
		await expect(page.locator('label:has-text("First Name")')).toBeVisible();
		await expect(page.locator('label:has-text("Last Name")')).toBeVisible();
		await expect(page.locator('label:has-text("Email")')).toBeVisible();
		await expect(page.locator('label:has-text("Department")')).toBeVisible();

		// Test form interaction
		await page.fill('input[placeholder*="First Name"]', "Test");
		await page.fill('input[placeholder*="Last Name"]', "User");
		await page.fill('input[placeholder*="Email"]', "test@example.com");

		// Check form buttons
		await expect(page.locator('button:has-text("Submit")')).toBeVisible();
		await expect(page.locator('button:has-text("Reset")')).toBeVisible();
	});

	test("should handle remote component loading states", async ({
		page,
		moduleFederation
	}) => {
		// Check for loading indicators when switching tabs
		await moduleFederation.switchToRemoteTab("User Card");

		// Should show loading initially
		const loadingText = page.locator("text=Loading remote component...");

		// Wait for actual component to load
		await page.waitForSelector(".ant-card", { timeout: 15000 });

		// Loading should be gone
		await expect(loadingText).not.toBeVisible();
	});

	test("should demonstrate shared dependencies", async ({
		page,
		moduleFederation
	}) => {
		// Switch between different remote components to verify they all use shared Ant Design
		const tabs = [
			{ name: "User Card", selector: ".ant-avatar" },
			{ name: "Data Table", selector: ".ant-table" },
			{ name: "Form Builder", selector: ".ant-form" }
		];

		for (const tab of tabs) {
			await moduleFederation.switchToRemoteTab(tab.name);
			await page.waitForSelector(tab.selector, { timeout: 15000 });
			await expect(page.locator(tab.selector)).toBeVisible();
		}
	});
});
