import { defineConfig, devices } from "@playwright/test";

const useLegacyStart = !!process.env.LEGACY_START;

export default defineConfig({
	testDir: "./e2e",
	timeout: 60000,
	expect: {
		timeout: 15000
	},
	fullyParallel: true,
	forbidOnly: !!process.env.CI,
	retries: process.env.CI ? 1 : 0,
	workers: process.env.CI ? 1 : undefined,
	reporter: [
		["html", { outputFolder: "playwright-report", open: "never" }],
		["list"]
	],
	use: {
		baseURL: "http://localhost:3001",
		trace: "on-first-retry",
		screenshot: "only-on-failure",
		video: "retain-on-failure",
		viewport: { width: 1920, height: 1080 },
		// Clear all browser storage to prevent stale Module Federation metadata
		storageState: undefined
	},

	projects: [
		{
			name: "chromium",
			use: { ...devices["Desktop Chrome"] }
		}
	],

	webServer: useLegacyStart
		? {
				command: "node start-legacy-servers.js",
				port: 3001,
				reuseExistingServer: false,
				timeout: 180000
			}
		: {
				command: "pnpm start",
				port: 3001,
				reuseExistingServer: false,
				timeout: 120000
			}
});
