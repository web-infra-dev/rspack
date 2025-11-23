import { expect, test } from "./setup.js";

test.describe("Development Mode Tests", () => {
	test("should access host app on port 3001", async ({ page }) => {
		// Listen for console errors and fail immediately
		page.on("console", msg => {
			if (msg.type() === "error") {
				throw new Error(`Console error: ${msg.text()}`);
			}
		});

		// Listen for page errors and fail immediately
		page.on("pageerror", error => {
			throw new Error(`Page error: ${error.message}`);
		});

		try {
			await page.goto("http://localhost:3001");

			// Just check if page loads (any content is fine)
			await page.waitForSelector("body", { timeout: 10000 });

			const title = await page.title();
			console.log("Host app title:", title);

			// Check if React app mounted
			const hasReactRoot = await page.locator("#root").isVisible();
			console.log("React root found:", hasReactRoot);

			expect(hasReactRoot).toBe(true);
		} catch (error) {
			console.log("Host app error:", error.message);
			throw error;
		}
	});

	test("should access remote app on port 3002", async ({ page }) => {
		// Listen for console errors and fail immediately
		page.on("console", msg => {
			if (msg.type() === "error") {
				throw new Error(`Console error: ${msg.text()}`);
			}
		});

		// Listen for page errors and fail immediately
		page.on("pageerror", error => {
			throw new Error(`Page error: ${error.message}`);
		});

		try {
			await page.goto("http://localhost:3002");

			// Just check if page loads
			await page.waitForSelector("body", { timeout: 10000 });

			const title = await page.title();
			console.log("Remote app title:", title);

			// Check if React app mounted
			const hasReactRoot = await page.locator("#root").isVisible();
			console.log("React root found:", hasReactRoot);

			expect(hasReactRoot).toBe(true);
		} catch (error) {
			console.log("Remote app error:", error.message);
			throw error;
		}
	});

	test("should verify remoteEntry.js is accessible", async ({ page }) => {
		const response = await page.request.get(
			"http://localhost:3002/remoteEntry.js"
		);
		console.log("RemoteEntry status:", response.status());
		console.log("RemoteEntry headers:", response.headers());

		expect(response.status()).toBe(200);
	});
});
