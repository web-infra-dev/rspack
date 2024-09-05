"use strict";

const webpack = require("@rspack/core");
const { RspackDevServer: Server } = require("@rspack/dev-server");
const config = require("../fixtures/simple-config/webpack.config");
const runBrowser = require("../helpers/run-browser");
const port = require("../helpers/ports-map")["client-reconnect-option"];

describe("client.reconnect option", () => {
	describe("specified as true", () => {
		let compiler;
		let server;
		let page;
		let browser;
		let pageErrors;
		let consoleMessages;

		beforeEach(async () => {
			compiler = webpack(config);

			server = new Server({ port, client: { reconnect: true } }, compiler);

			await server.start();

			({ page, browser } = await runBrowser());

			pageErrors = [];
			consoleMessages = [];
		});

		afterEach(async () => {
			await browser.close();
		});

		it("should try to reconnect unlimited times", async () => {
			page
				.on("console", message => {
					consoleMessages.push(message);
				})
				.on("pageerror", error => {
					pageErrors.push(error);
				});

			const response = await page.goto(`http://127.0.0.1:${port}/`, {
				waitUntil: "networkidle0"
			});

			try {
				expect(response.status()).toMatchSnapshot("response status");
			} catch (error) {
				throw error;
			} finally {
				await server.stop();
			}

			let interval;

			await new Promise(resolve => {
				interval = setInterval(() => {
					const retryingMessages = consoleMessages.filter(message =>
						message.text().includes("Trying to reconnect...")
					);

					if (retryingMessages.length >= 5) {
						clearInterval(interval);

						resolve();
					}
				}, 1000);
			});

			expect(pageErrors).toMatchSnapshot("page errors");
		});
	});

	describe("specified as false", () => {
		let compiler;
		let server;
		let page;
		let browser;
		let pageErrors;
		let consoleMessages;

		beforeEach(async () => {
			compiler = webpack(config);

			server = new Server({ port, client: { reconnect: false } }, compiler);

			await server.start();

			({ page, browser } = await runBrowser());

			pageErrors = [];
			consoleMessages = [];
		});

		afterEach(async () => {
			await browser.close();
		});

		it("should not try to reconnect", async () => {
			page
				.on("console", message => {
					consoleMessages.push(message);
				})
				.on("pageerror", error => {
					pageErrors.push(error);
				});

			const response = await page.goto(`http://127.0.0.1:${port}/`, {
				waitUntil: "networkidle0"
			});

			try {
				expect(response.status()).toMatchSnapshot("response status");
			} catch (error) {
				throw error;
			} finally {
				await server.stop();
			}

			// Can't wait to check for unlimited times so wait only for couple retries
			await new Promise(resolve =>
				setTimeout(
					() => {
						resolve();
					},
					// eslint-disable-next-line no-restricted-properties
					1000 * Math.pow(2, 3)
				)
			);

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");
		});
	});

	describe("specified as number", () => {
		let compiler;
		let server;
		let page;
		let browser;
		let pageErrors;
		let consoleMessages;

		beforeEach(async () => {
			compiler = webpack(config);

			server = new Server({ port, client: { reconnect: 2 } }, compiler);

			await server.start();

			({ page, browser } = await runBrowser());

			pageErrors = [];
			consoleMessages = [];
		});

		afterEach(async () => {
			await browser.close();
		});

		it("should try to reconnect 2 times", async () => {
			page
				.on("console", message => {
					consoleMessages.push(message);
				})
				.on("pageerror", error => {
					pageErrors.push(error);
				});

			const response = await page.goto(`http://127.0.0.1:${port}/`, {
				waitUntil: "networkidle0"
			});

			try {
				expect(response.status()).toMatchSnapshot("response status");
			} catch (error) {
				throw error;
			} finally {
				await server.stop();
			}

			// Can't wait to check for unlimited times so wait only for couple retries
			await new Promise(resolve =>
				setTimeout(
					() => {
						resolve();
					},
					// eslint-disable-next-line no-restricted-properties
					1000 * Math.pow(2, 3)
				)
			);

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");
		});
	});
});
