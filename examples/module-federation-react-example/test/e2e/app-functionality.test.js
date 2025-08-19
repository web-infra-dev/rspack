import fs from "node:fs";
import path from "node:path";
import puppeteer from "puppeteer";
import { preview } from "vite";
import { beforeAll, describe, expect, it } from "vitest";

// Skip e2e tests due to esbuild environment issue with vite preview
describe.skip("E2E: Module Federation Apps", () => {
	let browser;
	let hostServer;
	let remoteServer;
	let hostPage;
	let remotePage;

	beforeAll(async () => {
		// Start preview servers for host and remote
		hostServer = await preview({
			root: path.resolve(__dirname, "../../host"),
			server: { port: 3001 }
		});

		remoteServer = await preview({
			root: path.resolve(__dirname, "../../remote"),
			server: { port: 3002 }
		});

		// Launch browser
		browser = await puppeteer.launch({
			headless: true,
			args: ["--no-sandbox", "--disable-setuid-sandbox"]
		});
	});

	afterAll(async () => {
		await browser?.close();
		await hostServer?.httpServer.close();
		await remoteServer?.httpServer.close();
	});

	describe("Host Application", () => {
		beforeAll(async () => {
			hostPage = await browser.newPage();
			await hostPage.goto("http://localhost:3001");
		});

		it("should load successfully", async () => {
			const title = await hostPage.title();
			expect(title).toContain("Host");

			// Check for main content
			const appElement = await hostPage.$("#app");
			expect(appElement).toBeTruthy();
		});

		it("should execute processItems function", async () => {
			// Check console logs for function execution
			const consoleLogs = [];
			hostPage.on("console", msg => consoleLogs.push(msg.text()));

			// Reload page to capture logs
			await hostPage.reload();
			await hostPage.waitForTimeout(1000);

			const hasProcessItems = consoleLogs.some(
				log => log.includes("processItems") || log.includes("Processed")
			);

			expect(hasProcessItems).toBe(true);
		});

		it("should load remote Button component", async () => {
			// Wait for remote component to load
			await hostPage.waitForSelector("button", { timeout: 5000 });

			const button = await hostPage.$("button");
			expect(button).toBeTruthy();

			const buttonText = await hostPage.$eval("button", el => el.textContent);
			expect(buttonText).toContain("Remote Button");
		});
	});

	describe("Remote Application", () => {
		beforeAll(async () => {
			remotePage = await browser.newPage();
			await remotePage.goto("http://localhost:3002");
		});

		it("should load successfully", async () => {
			const title = await remotePage.title();
			expect(title).toContain("Remote");

			const appElement = await remotePage.$("#app");
			expect(appElement).toBeTruthy();
		});

		it("should render Button component", async () => {
			const button = await remotePage.$("button");
			expect(button).toBeTruthy();

			const buttonText = await remotePage.$eval("button", el => el.textContent);
			expect(buttonText).toContain("Remote Button");
		});

		it("should handle button clicks", async () => {
			const button = await remotePage.$("button");

			// Click the button
			await button.click();
			await remotePage.waitForTimeout(100);

			// Check if click was handled (look for console logs or DOM changes)
			const consoleLogs = [];
			remotePage.on("console", msg => consoleLogs.push(msg.text()));

			await button.click();

			// Verify click handling
			const hasClickLog = consoleLogs.some(
				log => log.includes("clicked") || log.includes("Remote button")
			);

			expect(hasClickLog).toBe(true);
		});
	});

	describe("Optimization Impact", () => {
		it("should not break app functionality after optimization", async () => {
			// Check if optimized files exist
			const hostDistPath = path.resolve(__dirname, "../../host/dist");
			const optimizedChunk = fs
				.readdirSync(hostDistPath)
				.find(
					file => file.includes("lodash-es") && !file.endsWith(".original")
				);

			expect(optimizedChunk).toBeDefined();

			// Verify app still works with optimized chunk
			const testPage = await browser.newPage();
			await testPage.goto("http://localhost:3001");

			// Wait for app to load
			await testPage.waitForSelector("#app", { timeout: 5000 });

			// Check for errors
			const errors = [];
			testPage.on("pageerror", error => errors.push(error.message));

			await testPage.reload();
			await testPage.waitForTimeout(1000);

			expect(errors).toHaveLength(0);
		});
	});
});
