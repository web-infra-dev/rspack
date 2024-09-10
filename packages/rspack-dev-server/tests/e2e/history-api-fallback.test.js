"use strict";

const path = require("path");
const webpack = require("@rspack/core");
const { RspackDevServer: Server } = require("@rspack/dev-server");
const config = require("../fixtures/historyapifallback-config/webpack.config");
const config2 = require("../fixtures/historyapifallback-2-config/webpack.config");
const config3 = require("../fixtures/historyapifallback-3-config/webpack.config");
const runBrowser = require("../helpers/run-browser");
const port = require("../helpers/ports-map")["history-api-fallback-option"];

describe("historyApiFallback option", () => {
	describe("as boolean", () => {
		let compiler;
		let server;
		let page;
		let browser;
		let pageErrors;
		let consoleMessages;

		beforeEach(async () => {
			compiler = webpack(config);

			server = new Server(
				{
					historyApiFallback: true,
					port
				},
				compiler
			);

			await server.start();

			({ page, browser } = await runBrowser());

			pageErrors = [];
			consoleMessages = [];
		});

		afterEach(async () => {
			await browser.close();
			await server.stop();
		});

		it("should handle GET request to directory", async () => {
			page
				.on("console", message => {
					consoleMessages.push(message);
				})
				.on("pageerror", error => {
					pageErrors.push(error);
				});

			const response = await page.goto(`http://127.0.0.1:${port}/foo`, {
				waitUntil: "networkidle0"
			});

			expect(response.headers()["content-type"]).toMatchSnapshot(
				"response headers content-type"
			);

			expect(response.status()).toMatchSnapshot("response status");

			expect(await response.text()).toMatchSnapshot("response text");

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");
		});
	});

	describe("as object", () => {
		let compiler;
		let server;
		let page;
		let browser;
		let pageErrors;
		let consoleMessages;

		beforeEach(async () => {
			compiler = webpack(config);

			server = new Server(
				{
					historyApiFallback: {
						index: "/bar.html"
					},
					port
				},
				compiler
			);

			await server.start();

			({ page, browser } = await runBrowser());

			pageErrors = [];
			consoleMessages = [];
		});

		afterEach(async () => {
			await browser.close();
			await server.stop();
		});

		it("should handle GET request to directory", async () => {
			page
				.on("console", message => {
					consoleMessages.push(message);
				})
				.on("pageerror", error => {
					pageErrors.push(error);
				});

			const response = await page.goto(`http://127.0.0.1:${port}/foo`, {
				waitUntil: "networkidle0"
			});

			expect(response.headers()["content-type"]).toMatchSnapshot(
				"response headers content-type"
			);

			expect(response.status()).toMatchSnapshot("response status");

			expect(await response.text()).toMatchSnapshot("response text");

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");
		});
	});

	describe("as object with static", () => {
		let compiler;
		let server;
		let page;
		let browser;
		let pageErrors;
		let consoleMessages;

		beforeEach(async () => {
			compiler = webpack(config2);

			server = new Server(
				{
					static: path.resolve(
						__dirname,
						"../fixtures/historyapifallback-2-config"
					),
					historyApiFallback: {
						index: "/bar.html"
					},
					port
				},
				compiler
			);

			await server.start();

			({ page, browser } = await runBrowser());

			pageErrors = [];
			consoleMessages = [];
		});

		afterEach(async () => {
			await browser.close();
			await server.stop();
		});

		it("should handle GET request to directory", async () => {
			page
				.on("console", message => {
					consoleMessages.push(message);
				})
				.on("pageerror", error => {
					pageErrors.push(error);
				});

			const response = await page.goto(`http://127.0.0.1:${port}/foo`, {
				waitUntil: "networkidle0"
			});

			expect(response.headers()["content-type"]).toMatchSnapshot(
				"response headers content-type"
			);

			expect(response.status()).toMatchSnapshot("response status");

			expect(await response.text()).toMatchSnapshot("response text");

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");
		});

		it("should prefer static file over historyApiFallback", async () => {
			page
				.on("console", message => {
					consoleMessages.push(message);
				})
				.on("pageerror", error => {
					pageErrors.push(error);
				});

			const response = await page.goto(
				`http://127.0.0.1:${port}/random-file.txt`,
				{
					waitUntil: "networkidle2"
				}
			);

			expect(response.headers()["content-type"]).toMatchSnapshot(
				"response headers content-type"
			);

			expect(response.status()).toMatchSnapshot("response status");

			expect(await response.text()).toMatchSnapshot("response text");

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");
		});
	});

	describe("as object with static set to false", () => {
		let compiler;
		let server;
		let page;
		let browser;
		let pageErrors;
		let consoleMessages;

		beforeEach(async () => {
			compiler = webpack(config3);

			server = new Server(
				{
					static: false,
					historyApiFallback: {
						index: "/bar.html"
					},
					port
				},
				compiler
			);

			await server.start();

			({ page, browser } = await runBrowser());

			pageErrors = [];
			consoleMessages = [];
		});

		afterEach(async () => {
			await browser.close();
			await server.stop();
		});

		it("historyApiFallback should work and ignore static content", async () => {
			page
				.on("console", message => {
					consoleMessages.push(message);
				})
				.on("pageerror", error => {
					pageErrors.push(error);
				});

			const response = await page.goto(`http://127.0.0.1:${port}/index.html`, {
				waitUntil: "networkidle0"
			});

			expect(response.headers()["content-type"]).toMatchSnapshot(
				"response headers content-type"
			);

			expect(response.status()).toMatchSnapshot("response status");

			expect(await response.text()).toMatchSnapshot("response text");

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");
		});
	});

	describe("as object with static and rewrites", () => {
		let compiler;
		let server;
		let page;
		let browser;
		let pageErrors;
		let consoleMessages;

		beforeEach(async () => {
			compiler = webpack(config2);

			server = new Server(
				{
					port,
					static: path.resolve(
						__dirname,
						"../fixtures/historyapifallback-2-config"
					),
					historyApiFallback: {
						rewrites: [
							{
								from: /other/,
								to: "/other.html"
							},
							{
								from: /.*/,
								to: "/bar.html"
							}
						]
					}
				},
				compiler
			);

			await server.start();

			({ page, browser } = await runBrowser());

			pageErrors = [];
			consoleMessages = [];
		});

		afterEach(async () => {
			await browser.close();
			await server.stop();
		});

		it("historyApiFallback respect rewrites for index", async () => {
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

			expect(response.headers()["content-type"]).toMatchSnapshot(
				"response headers content-type"
			);

			expect(response.status()).toMatchSnapshot("response status");

			expect(await response.text()).toMatchSnapshot("response text");

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");
		});

		it("historyApiFallback respect rewrites and shows index for unknown urls", async () => {
			page
				.on("console", message => {
					consoleMessages.push(message);
				})
				.on("pageerror", error => {
					pageErrors.push(error);
				});

			const response = await page.goto(`http://127.0.0.1:${port}/acme`, {
				waitUntil: "networkidle0"
			});

			expect(response.headers()["content-type"]).toMatchSnapshot(
				"response headers content-type"
			);

			expect(response.status()).toMatchSnapshot("response status");

			expect(await response.text()).toMatchSnapshot("response text");

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");
		});

		it("historyApiFallback respect any other specified rewrites", async () => {
			page
				.on("console", message => {
					consoleMessages.push(message);
				})
				.on("pageerror", error => {
					pageErrors.push(error);
				});

			const response = await page.goto(`http://127.0.0.1:${port}/other`, {
				waitUntil: "networkidle0"
			});

			expect(response.headers()["content-type"]).toMatchSnapshot(
				"response headers content-type"
			);

			expect(response.status()).toMatchSnapshot("response status");

			expect(await response.text()).toMatchSnapshot("response text");

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");
		});
	});

	describe('as object with the "verbose" option', () => {
		let compiler;
		let server;
		let page;
		let browser;
		let pageErrors;
		let consoleMessages;
		let consoleSpy;

		beforeEach(async () => {
			consoleSpy = jest.spyOn(global.console, "log");

			compiler = webpack(config);

			server = new Server(
				{
					historyApiFallback: {
						index: "/bar.html",
						verbose: true
					},
					port
				},
				compiler
			);

			await server.start();

			({ page, browser } = await runBrowser());

			pageErrors = [];
			consoleMessages = [];
		});

		afterEach(async () => {
			consoleSpy.mockRestore();
			await browser.close();
			await server.stop();
		});

		it("request to directory and log", async () => {
			page
				.on("console", message => {
					consoleMessages.push(message);
				})
				.on("pageerror", error => {
					pageErrors.push(error);
				});

			const response = await page.goto(`http://127.0.0.1:${port}/foo`, {
				waitUntil: "networkidle0"
			});

			expect(response.headers()["content-type"]).toMatchSnapshot(
				"response headers content-type"
			);

			expect(response.status()).toMatchSnapshot("response status");

			expect(await response.text()).toMatchSnapshot("response text");

			expect(consoleSpy).toHaveBeenCalledWith(
				"Rewriting",
				"GET",
				"/foo",
				"to",
				"/bar.html"
			);

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");
		});
	});

	describe('as object with the "logger" option', () => {
		let compiler;
		let server;
		let page;
		let browser;
		let pageErrors;
		let consoleMessages;
		let consoleSpy;

		beforeEach(async () => {
			consoleSpy = jest.spyOn(global.console, "log");

			compiler = webpack(config);

			server = new Server(
				{
					historyApiFallback: {
						index: "/bar.html",
						logger: consoleSpy
					},
					port
				},
				compiler
			);

			await server.start();

			({ page, browser } = await runBrowser());

			pageErrors = [];
			consoleMessages = [];
		});

		afterEach(async () => {
			consoleSpy.mockRestore();
			await browser.close();
			await server.stop();
		});

		it("request to directory and log", async () => {
			page
				.on("console", message => {
					consoleMessages.push(message);
				})
				.on("pageerror", error => {
					pageErrors.push(error);
				});

			const response = await page.goto(`http://127.0.0.1:${port}/foo`, {
				waitUntil: "networkidle0"
			});

			expect(response.headers()["content-type"]).toMatchSnapshot(
				"response headers content-type"
			);

			expect(response.status()).toMatchSnapshot("response status");

			expect(await response.text()).toMatchSnapshot("response text");

			expect(consoleSpy).toHaveBeenCalledWith(
				"Rewriting",
				"GET",
				"/foo",
				"to",
				"/bar.html"
			);

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");
		});
	});

	describe("in-memory files", () => {
		let compiler;
		let server;
		let page;
		let browser;
		let pageErrors;
		let consoleMessages;

		beforeEach(async () => {
			compiler = webpack(config3);

			server = new Server(
				{
					static: path.resolve(
						__dirname,
						"../fixtures/historyapifallback-3-config"
					),
					historyApiFallback: true,
					port
				},
				compiler
			);

			await server.start();

			({ page, browser } = await runBrowser());

			pageErrors = [];
			consoleMessages = [];
		});

		afterEach(async () => {
			await browser.close();
			await server.stop();
		});

		it("should take precedence over static files", async () => {
			page
				.on("console", message => {
					consoleMessages.push(message);
				})
				.on("pageerror", error => {
					pageErrors.push(error);
				});

			const response = await page.goto(`http://127.0.0.1:${port}/foo`, {
				waitUntil: "networkidle0"
			});

			expect(response.headers()["content-type"]).toMatchSnapshot(
				"response headers content-type"
			);

			expect(response.status()).toMatchSnapshot("response status");

			expect(await response.text()).toMatchSnapshot("response text");

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");
		});

		it("should perform HEAD request in same way as GET", async () => {
			await page.goto(`http://127.0.0.1:${port}/foo`, {
				waitUntil: "networkidle0"
			});

			const responseGet = await page.evaluate(async () => {
				const response = await fetch("/foo", { method: "GET" });

				return {
					contentType: response.headers.get("content-type"),
					statusText: response.statusText,
					text: await response.text()
				};
			});

			expect(responseGet.contentType).toMatchSnapshot(
				"response headers content-type"
			);

			expect(responseGet.statusText).toMatchSnapshot("response status");

			expect(responseGet.text).toMatchSnapshot("response text");

			const responseHead = await page.evaluate(async () => {
				const response = await fetch("/foo", { method: "HEAD" });

				return {
					contentType: response.headers.get("content-type"),
					statusText: response.statusText,
					text: await response.text()
				};
			});

			expect(responseHead).toMatchObject({
				...responseGet,
				// HEAD response has an empty body
				text: ""
			});
		});
	});
});
