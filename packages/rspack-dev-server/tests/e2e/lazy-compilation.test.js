"use strict";

const webpack = require("@rspack/core");
const { RspackDevServer: Server } = require("@rspack/dev-server");
const lazyCompilationSingleEntryConfig = require("../fixtures/lazy-compilation-single-entry/webpack.config");
const lazyCompilationMultipleEntriesConfig = require("../fixtures/lazy-compilation-multiple-entries/webpack.config");
const runBrowser = require("../helpers/run-browser");
const port = require("../helpers/ports-map")["lazy-compilation"];

describe("lazy compilation", () => {
	// TODO jest freeze due webpack do not close `eventsource`, we should uncomment this after fix it on webpack side
	it.skip(`should work with single entry`, async () => {
		const compiler = webpack(lazyCompilationSingleEntryConfig);
		const server = new Server({ port }, compiler);

		await server.start();

		const { page, browser } = await runBrowser();

		try {
			const pageErrors = [];
			const consoleMessages = [];

			page
				.on("console", message => {
					consoleMessages.push(message.text());
				})
				.on("pageerror", error => {
					pageErrors.push(error);
				});

			await page.goto(`http://127.0.0.1:${port}/test.html`, {
				waitUntil: "domcontentloaded"
			});
			await new Promise(resolve => {
				const interval = setInterval(() => {
					if (consoleMessages.includes("Hey.")) {
						clearInterval(interval);

						resolve();
					}
				}, 100);
			});

			expect(consoleMessages).toMatchSnapshot("console messages");
			expect(pageErrors).toMatchSnapshot("page errors");
		} catch (error) {
			throw error;
		} finally {
			await browser.close();
			await server.stop();
		}
	});

	it.skip(`should work with multiple entries`, async () => {
		const compiler = webpack(lazyCompilationMultipleEntriesConfig);
		const server = new Server({ port }, compiler);

		await server.start();

		const { page, browser } = await runBrowser();

		try {
			const pageErrors = [];
			const consoleMessages = [];

			page
				.on("console", message => {
					consoleMessages.push(message.text());
				})
				.on("pageerror", error => {
					pageErrors.push(error);
				});

			await page.goto(`http://127.0.0.1:${port}/test-one.html`, {
				waitUntil: "domcontentloaded"
			});
			await new Promise(resolve => {
				const interval = setInterval(() => {
					console.log(consoleMessages);
					if (consoleMessages.includes("One.")) {
						clearInterval(interval);

						resolve();
					}
				}, 100);
			});

			await page.goto(`http://127.0.0.1:${port}/test-two.html`, {
				waitUntil: "domcontentloaded"
			});
			await new Promise(resolve => {
				const interval = setInterval(() => {
					console.log(consoleMessages);
					if (consoleMessages.includes("Two.")) {
						clearInterval(interval);

						resolve();
					}
				}, 100);
			});

			expect(consoleMessages).toMatchSnapshot("console messages");
			expect(pageErrors).toMatchSnapshot("page errors");
		} catch (error) {
			throw error;
		} finally {
			await browser.close();
			await server.stop();
		}
	});
});
