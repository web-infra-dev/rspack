import { RspackDevServer } from "@rspack/dev-server";
import { createCompiler } from "@rspack/core";
import { initFixture, installDeps } from "../helpers/tempDir";
import { editFile } from "../helpers/emitFile";
import path from "path";
import runBrowser from "../helpers/runBrowser";
import type { Browser, Page } from "puppeteer";

// FIXME: A concurrency problem occurs when executing on ci
// describe("reload and hot should works", () => {
// 	let browser: Browser;
// 	let server: RspackDevServer;

// 	afterEach(async () => {
// 		if (browser) {
// 			await browser.close();
// 		}
// 		if (server) {
// 			await server.stop();
// 		}
// 	});

// 	it("reload should works", async () => {
// 		console.log("=== first start ===");
// 		const tempDir = await initFixture("react");
// 		await installDeps(tempDir);
// 		const config = require(path.resolve(tempDir, "./webpack.config.js"));
// 		const compiler = createCompiler({
// 			...config,
// 			context: tempDir,
// 			devServer: {
// 				hot: false,
// 				liveReload: true
// 			}
// 		});
// 		server = new RspackDevServer(compiler);
// 		await server.start();
// 		const launched = await runBrowser();
// 		({ browser } = launched);
// 		const { page } = launched;

// 		const consoleMessages: string[] = [];
// 		page.on("console", message => {
// 			const text = message.text();
// 			consoleMessages.push(text);
// 		});

// 		await page.goto(`http://localhost:${server.options.port}`);
// 		await page.click(".test-button");
// 		expect(await getText(page, ".test-button-content")).toBe("1");
// 		expect(await getText(page, ".placeholder")).toBe("__PLACE_HOLDER__");
// 		await editFile(path.resolve(tempDir, "./app.jsx"), code =>
// 			code.replace("__PLACE_HOLDER__", "update")
// 		);
// 		expect(await getText(page, ".test-button-content")).toBe("0");
// 		await page.click(".test-button");
// 		expect(await getText(page, ".placeholder")).toBe("update");
// 		await editFile(path.resolve(tempDir, "./index.css"), code =>
// 			code.replace("rgba(0, 0, 0, 0)", "rgba(255, 0, 0, 0)")
// 		);
// 		expect(await getText(page, ".test-button-content")).toBe("0");
// 		expect((await getComputedStyle(page, "body")).backgroundColor).toBe(
// 			"rgba(255, 0, 0, 0)"
// 		);
// 		console.log("=== first end ===");
// 	});

// 	it("hot should works", async () => {
// 		console.log("=== second start ===");
// 		const tempDir = await initFixture("react");
// 		await installDeps(tempDir);
// 		const config = require(path.resolve(tempDir, "./webpack.config.js"));
// 		const compiler = createCompiler({
// 			...config,
// 			context: tempDir
// 		});
// 		server = new RspackDevServer(compiler);
// 		await server.start();

// 		const launched = await runBrowser();
// 		({ browser } = launched);
// 		const { page } = launched;

// 		const consoleMessages: string[] = [];
// 		page.on("console", message => {
// 			const text = message.text();
// 			consoleMessages.push(text);
// 		});

// 		await page.goto(`http://localhost:${server.options.port}`);

// 		await page.click(".test-button");
// 		expect(await getText(page, ".test-button-content")).toBe("1");
// 		expect(await getText(page, ".placeholder")).toBe("__PLACE_HOLDER__");
// 		await editFile(path.resolve(tempDir, "./app.jsx"), code =>
// 			code.replace("__PLACE_HOLDER__", "update")
// 		);
// 		expect(await getText(page, ".placeholder")).toBe("update");
// 		await editFile(path.resolve(tempDir, "./index.css"), code =>
// 			code.replace("rgba(0, 0, 0, 0)", "rgba(255, 0, 0, 0)")
// 		);
// 		expect((await getComputedStyle(page, "body")).backgroundColor).toBe(
// 			"rgba(255, 0, 0, 0)"
// 		);
// 		expect(await getText(page, ".test-button-content")).toBe("1");
// 		expect(consoleMessages).toContain("App hot update...");
// 		console.log("=== second end ===");
// 	});
// });

describe("reload and hot should works", () => {
	it("reload and hot should works", async () => {
		// reload should works
		let tempDir = await initFixture("react");
		await installDeps(tempDir);
		let config = require(path.resolve(tempDir, "./webpack.config.js"));
		let compiler = createCompiler({
			...config,
			context: tempDir,
			devServer: {
				hot: false,
				liveReload: true
			}
		});
		let server = new RspackDevServer(compiler);
		await server.start();
		let launched = await runBrowser();
		let { browser, page } = launched;

		await page.goto(`http://localhost:${server.options.port}`);
		await page.click(".test-button");
		expect(await getText(page, ".test-button-content")).toBe("1");
		expect(await getText(page, ".placeholder")).toBe("__PLACE_HOLDER__");
		await editFile(path.resolve(tempDir, "./app.jsx"), code =>
			code.replace("__PLACE_HOLDER__", "update")
		);
		expect(await getText(page, ".test-button-content")).toBe("0");
		await page.click(".test-button");
		expect(await getText(page, ".placeholder")).toBe("update");
		await editFile(path.resolve(tempDir, "./index.css"), code =>
			code.replace("rgba(0, 0, 0, 0)", "rgba(255, 0, 0, 0)")
		);
		expect(await getText(page, ".test-button-content")).toBe("0");
		expect((await getComputedStyle(page, "body")).backgroundColor).toBe(
			"rgba(255, 0, 0, 0)"
		);
		console.log("=== first end ===");
		await server.stop();
		await browser.close();

		// hot should works;
		console.log("=== second start ===");
		tempDir = await initFixture("react");
		await installDeps(tempDir);
		config = require(path.resolve(tempDir, "./webpack.config.js"));
		compiler = createCompiler({
			...config,
			context: tempDir
		});
		server = new RspackDevServer(compiler);
		await server.start();
		launched = await runBrowser();
		({ browser, page } = launched);
		const consoleMessages: string[] = [];
		page.on("console", message => {
			const text = message.text();
			consoleMessages.push(text);
		});
		await page.goto(`http://localhost:${server.options.port}`);
		await page.click(".test-button");
		expect(await getText(page, ".test-button-content")).toBe("1");
		expect(await getText(page, ".placeholder")).toBe("__PLACE_HOLDER__");
		await editFile(path.resolve(tempDir, "./app.jsx"), code =>
			code.replace("__PLACE_HOLDER__", "update")
		);
		expect(await getText(page, ".placeholder")).toBe("update");
		await editFile(path.resolve(tempDir, "./index.css"), code =>
			code.replace("rgba(0, 0, 0, 0)", "rgba(255, 0, 0, 0)")
		);
		expect((await getComputedStyle(page, "body")).backgroundColor).toBe(
			"rgba(255, 0, 0, 0)"
		);
		expect(await getText(page, ".test-button-content")).toBe("1");
		expect(consoleMessages).toContain("App hot update...");
		console.log("=== second end ===");
	});
});

async function getComputedStyle(
	page: Page,
	selector: string
): Promise<CSSStyleDeclaration> {
	await page.waitForSelector(selector);
	return await page.$eval(selector, ele =>
		JSON.parse(JSON.stringify(window.getComputedStyle(ele)))
	);
}

async function getText(page: Page, selector: string) {
	await page.waitForSelector(selector);
	return await page.$eval(selector, ele => ele.textContent);
}
