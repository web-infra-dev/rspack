import { defineConfig, devices } from "@playwright/test";
import type { RspackOptions } from "./fixtures";

const TIMEOUT = 2 * 60 * 1000;

export default defineConfig<RspackOptions>({
	// Look for test files in the "fixtures" directory, relative to this configuration file.
	testDir: "./cases",

	//	globalSetup: require.resolve("./scripts/globalSetup"),

	// Run all tests in parallel.
	//	fullyParallel: true,

	// Fail the build on CI if you accidentally left test.only in the source code.
	forbidOnly: !!process.env.CI,

	retries: 0,

	// timeout 30s
	timeout: TIMEOUT,

	// expect
	expect: {
		// auto-assertion could be used with HMR.
		timeout: TIMEOUT
	},

	// Opt out of parallel tests on CI.
	workers: process.env.CI ? 4 : undefined,

	// Reporter to use
	reporter: "html",

	use: {
		// Base URL to use in actions like `await page.goto('/')`.
		// baseURL: "http://127.0.0.1:3000",

		// Collect trace when retrying the failed test.
		trace: "on-first-retry"
	},
	// Configure projects for major browsers.
	projects: [
		{
			name: "chromium",
			use: {
				rspackConfig: {
					handleConfig: (config: any) => {
						return config;
					}
				},
				...devices["Desktop Chrome"]
			}
		}
	]
	// Run your local dev server before starting the tests.
	// webServer: {
	//	 command: "npm run start",
	//	 url: "http://127.0.0.1:3000",
	//	 reuseExistingServer: !process.env.CI
	// }
});
