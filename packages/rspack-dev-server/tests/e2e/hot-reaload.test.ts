import { RspackDevServer } from "@rspack/dev-server";
import { createCompiler } from "@rspack/core";
import { initFixture, installDeps } from "../helpers/tempDir";
import { editFile, waitingForBuild } from "../helpers/emitFile";
import path from "path";
import runBrowser from "../helpers/runBrowser";
import type { Browser, Page } from "puppeteer";

async function wait(time) {
	return new Promise(resolve => {
		setTimeout(() => {
			resolve(undefined);
		}, time);
	});
}

describe("reload and hot should works", () => {
	it.skip("reload and hot should works", async () => {
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
		let server = new RspackDevServer(
			compiler.options.devServer ?? {},
			compiler
		);
		await server.start();
		await waitingForBuild(server.options.port);
		console.log("=== before goto page ===");
		let { browser, page } = await runBrowser();

		await page.goto(`http://localhost:${server.options.port}`);
		await page.click(".test-button");
		expect(await getText(page, ".test-button-content")).toBe("1");
		expect(await getText(page, ".placeholder")).toBe("__PLACE_HOLDER__");

		const appFilePath = path.resolve(tempDir, "./app.jsx");
		await editFile(appFilePath, code =>
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
		await server.stop();
		await browser.close();
		console.log("=== first end ===");

		// hot should works;
		console.log("=== second start ===");
		tempDir = await initFixture("react");
		await installDeps(tempDir);
		config = require(path.resolve(tempDir, "./webpack.config.js"));
		compiler = createCompiler({
			...config,
			context: tempDir
		});
		server = new RspackDevServer(compiler.options.devServer ?? {}, compiler);
		await server.start();
		await waitingForBuild(server.options.port);
		console.log("=== before goto page ===");

		({ browser, page } = await runBrowser());
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
		await server.stop();
		await browser.close();
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
