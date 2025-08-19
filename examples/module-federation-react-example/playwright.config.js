import path from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig, devices } from "@playwright/test";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

/**
 * @see https://playwright.dev/docs/test-configuration
 */
export default defineConfig({
	testDir: "./test/e2e",
	globalSetup: "./test/e2e/global-setup.js",
	globalTeardown: "./test/e2e/global-teardown.js",
	// Avoid picking up Vitest-style *.test.js files that import vitest and conflict with Playwright expect
	testMatch: ["**/*.spec.js"],
	/* Run tests in files in parallel */
	fullyParallel: true,
	/* Fail the build on CI if you accidentally left test.only in the source code. */
	forbidOnly: !!process.env.CI,
	/* Retry on CI only */
	retries: process.env.CI ? 2 : 0,
	/* Opt out of parallel tests on CI. */
	workers: process.env.CI ? 1 : undefined,
	/* Reporter to use. See https://playwright.dev/docs/test-reporters */
	reporter: "list",
	/* Shared settings for all the projects below. See https://playwright.dev/docs/api/class-testoptions. */
	use: {
		/* Base URL to use in actions like `await page.goto('/')`. */
		baseURL: "http://localhost:3001",
		/* Collect trace when retrying the failed test. See https://playwright.dev/docs/trace-viewer */
		trace: "on-first-retry",
		screenshot: "only-on-failure",
		/* Fail tests on console errors */
		extraHTTPHeaders: {
			Accept: "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"
		}
	},

	/* Configure projects for major browsers */
	projects: [
		{
			name: "chromium",
			use: { ...devices["Desktop Chrome"] }
		}
	],

	/* Configure production server for testing - only start if not already running */
	webServer: process.env.PLAYWRIGHT_SKIP_WEBSERVER
		? undefined
		: [
				{
					command: "pnpm -C host serve",
					url: "http://localhost:3001/",
					reuseExistingServer: true,
					timeout: 120000
				},
				{
					command: "pnpm -C remote serve",
					url: "http://localhost:3002/",
					reuseExistingServer: true,
					timeout: 120000
				}
			]
});
