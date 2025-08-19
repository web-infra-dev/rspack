// E2E Test Setup for Module Federation React Example

import { test as base, expect } from "@playwright/test";

// Extend base test with custom fixtures
export const test = base.extend({
	// Automatically watch for critical console/page errors that indicate removed modules
	consoleErrorWatcher: [
		async ({ page }, use) => {
			const messages = [];
			const isCritical = text =>
				/is not a function|Cannot read (?:properties|property) of undefined|undefined is not a function/i.test(
					text
				);

			page.on("console", msg => {
				const text = msg.text?.() ?? String(msg);
				if (msg.type?.() === "error" && isCritical(text)) {
					messages.push(`[console.${msg.type?.()}] ${text}`);
				}
			});

			page.on("pageerror", error => {
				const text = error?.message || String(error);
				if (isCritical(text)) {
					messages.push(`[pageerror] ${text}`);
				}
			});

			await use({ messages });
		},
		{ auto: true }
	],
	// Custom fixture for Module Federation testing
	moduleFederation: async ({ page }, use) => {
		// Reduce UI flakiness by disabling animations/transitions
		await page.addStyleTag({
			content: "* { transition: none !important; animation: none !important; }"
		});
		const mf = {
			// Robust tab click helper to avoid detachment during AntD re-renders
			async clickTab(tabName) {
				// Ensure some tab container is present (robust to markup differences)
				await page.waitForSelector('[role="tab"], .ant-tabs-tab', {
					timeout: 10000
				});
				const tab = page.getByRole("tab", { name: tabName }).first();
				await tab.scrollIntoViewIfNeeded();
				await tab.waitFor({ state: "visible" });
				// Attempt click with small retries in case of React re-render
				for (let attempt = 0; attempt < 3; attempt += 1) {
					try {
						await tab.click({ timeout: 3000 });
						return;
					} catch (_err) {
						await page.waitForTimeout(150);
					}
				}
				// Final forced click as fallback
				await tab.click({ force: true });
			},
			// Helper to wait for remote components to load
			async waitForRemoteComponent(selector, timeout = 15000) {
				await page.waitForSelector(selector, { timeout });
				// Additional wait to ensure component is fully rendered
				await page.waitForTimeout(100);
			},

			// Helper to check if remote entry is loaded
			async isRemoteEntryLoaded() {
				const response = await page.request.get(
					"http://localhost:3002/remoteEntry.js"
				);
				return response.status() === 200;
			},

			// Helper to switch remote component tabs
			async switchToRemoteTab(tabName) {
				await this.clickTab(tabName);
				await this.waitForRemoteComponent(
					".remote-component-wrapper, .ant-card, .ant-table, .ant-form"
				);
			},

			// Helper to check shared dependency loading
			async checkSharedDependencies() {
				const sharedLibs = await page.evaluate(() => {
					// Check for shared React instance
					const reactVersions =
						window.__REACT_DEVTOOLS_GLOBAL_HOOK__?.renderers?.size || 0;

					// Check for Ant Design components
					const antElements =
						document.querySelectorAll('[class*="ant-"]').length;

					return {
						reactInstances: reactVersions,
						antComponents: antElements > 0
					};
				});

				return sharedLibs;
			}
		};

		await use(mf);
	},

	// Custom fixture for performance monitoring
	performanceMonitor: async ({ page }, use) => {
		const metrics = {
			resourceSizes: [],
			networkRequests: [],
			timings: {}
		};

		// Monitor network requests
		page.on("request", request => {
			metrics.networkRequests.push({
				url: request.url(),
				method: request.method(),
				timestamp: Date.now()
			});
		});

		page.on("response", async response => {
			if (response.url().includes(".js") && response.status() === 200) {
				const headers = response.headers();
				const contentLength = headers["content-length"];
				if (contentLength) {
					metrics.resourceSizes.push({
						url: response.url(),
						size: parseInt(contentLength),
						compressed: !!headers["content-encoding"]
					});
				}
			}
		});

		const monitor = {
			getMetrics: () => metrics,

			async measureLoadTime(action) {
				const start = Date.now();
				await action();
				const end = Date.now();
				return end - start;
			},

			getTotalBundleSize() {
				return metrics.resourceSizes.reduce(
					(sum, resource) => sum + resource.size,
					0
				);
			},

			getRequestCount() {
				return metrics.networkRequests.length;
			}
		};

		await use(monitor);
	}
});

export { expect };

// Global test configuration
export const E2E_CONFIG = {
	TIMEOUTS: {
		REMOTE_COMPONENT: 15000,
		CHART_LOADING: 10000,
		NAVIGATION: 5000
	},

	URLS: {
		HOST: "http://localhost:3001",
		REMOTE: "http://localhost:3002"
	},

	PERFORMANCE_BUDGETS: {
		INITIAL_LOAD: 5000,
		REMOTE_COMPONENT: 3000,
		TOTAL_BUNDLE_SIZE: 2 * 1024 * 1024, // 2MB
		MAX_CHUNK_SIZE: 1024 * 1024 // 1MB
	}
};

// After each test, fail fast if critical runtime errors were seen
test.afterEach(async ({ consoleErrorWatcher }) => {
	const errs = consoleErrorWatcher?.messages ?? [];
	expect
		.soft(
			errs,
			"Critical console/page errors detected (possible removed module usage)"
		)
		.toEqual([]);
});
