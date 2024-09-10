"use strict";

const os = require("os");
const webpack = require("@rspack/core");
const { RspackDevServer: Server } = require("@rspack/dev-server");
const config = require("../fixtures/simple-config/webpack.config");
const runBrowser = require("../helpers/run-browser");
const port = require("../helpers/ports-map").bonjour;

describe("bonjour option", () => {
	let mockPublish;
	let mockUnpublishAll;
	let mockDestroy;

	beforeEach(() => {
		mockPublish = jest.fn();
		mockUnpublishAll = jest.fn(callback => {
			callback();
		});
		mockDestroy = jest.fn();
	});

	describe("as true", () => {
		let compiler;
		let server;
		let page;
		let browser;
		let pageErrors;
		let consoleMessages;

		beforeEach(async () => {
			jest.mock("bonjour-service", () => {
				return {
					Bonjour: jest.fn().mockImplementation(() => {
						return {
							publish: mockPublish,
							unpublishAll: mockUnpublishAll,
							destroy: mockDestroy
						};
					})
				};
			});

			compiler = webpack(config);

			server = new Server({ port, bonjour: true }, compiler);

			await server.start();

			({ page, browser } = await runBrowser());

			pageErrors = [];
			consoleMessages = [];
		});

		afterEach(async () => {
			await browser.close();
			await server.stop();

			mockPublish.mockReset();
			mockUnpublishAll.mockReset();
			mockDestroy.mockReset();
		});

		it("should call bonjour with correct params", async () => {
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

			expect(mockPublish).toHaveBeenCalledTimes(1);

			expect(mockPublish).toHaveBeenCalledWith({
				name: `Webpack Dev Server ${os.hostname()}:${port}`,
				port,
				type: "http",
				subtypes: ["webpack"]
			});

			expect(mockUnpublishAll).toHaveBeenCalledTimes(0);
			expect(mockDestroy).toHaveBeenCalledTimes(0);

			expect(response.status()).toMatchSnapshot("response status");

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");
		});
	});

	describe("with 'server' option", () => {
		let compiler;
		let server;
		let page;
		let browser;
		let pageErrors;
		let consoleMessages;

		beforeEach(async () => {
			jest.mock("bonjour-service", () => {
				return {
					Bonjour: jest.fn().mockImplementation(() => {
						return {
							publish: mockPublish,
							unpublishAll: mockUnpublishAll,
							destroy: mockDestroy
						};
					})
				};
			});

			compiler = webpack(config);

			server = new Server({ bonjour: true, port, server: "https" }, compiler);

			await server.start();

			({ page, browser } = await runBrowser());

			pageErrors = [];
			consoleMessages = [];
		});

		afterEach(async () => {
			await browser.close();
			await server.stop();
		});

		it("should call bonjour with 'https' type", async () => {
			page
				.on("console", message => {
					consoleMessages.push(message);
				})
				.on("pageerror", error => {
					pageErrors.push(error);
				});

			const response = await page.goto(`https://127.0.0.1:${port}/`, {
				waitUntil: "networkidle0"
			});

			expect(mockPublish).toHaveBeenCalledTimes(1);

			expect(mockPublish).toHaveBeenCalledWith({
				name: `Webpack Dev Server ${os.hostname()}:${port}`,
				port,
				type: "https",
				subtypes: ["webpack"]
			});

			expect(mockUnpublishAll).toHaveBeenCalledTimes(0);
			expect(mockDestroy).toHaveBeenCalledTimes(0);

			expect(response.status()).toMatchSnapshot("response status");

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
			jest.mock("bonjour-service", () => {
				return {
					Bonjour: jest.fn().mockImplementation(() => {
						return {
							publish: mockPublish,
							unpublishAll: mockUnpublishAll,
							destroy: mockDestroy
						};
					})
				};
			});

			compiler = webpack(config);

			server = new Server(
				{
					port,
					bonjour: {
						type: "https",
						protocol: "udp"
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

		it("should apply bonjour options", async () => {
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

			expect(mockPublish).toHaveBeenCalledTimes(1);

			expect(mockPublish).toHaveBeenCalledWith({
				name: `Webpack Dev Server ${os.hostname()}:${port}`,
				port,
				type: "https",
				protocol: "udp",
				subtypes: ["webpack"]
			});

			expect(mockUnpublishAll).toHaveBeenCalledTimes(0);
			expect(mockDestroy).toHaveBeenCalledTimes(0);

			expect(response.status()).toMatchSnapshot("response status");

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");
		});
	});

	describe("bonjour object and 'server' option", () => {
		let compiler;
		let server;
		let page;
		let browser;
		let pageErrors;
		let consoleMessages;

		beforeEach(async () => {
			jest.mock("bonjour-service", () => {
				return {
					Bonjour: jest.fn().mockImplementation(() => {
						return {
							publish: mockPublish,
							unpublishAll: mockUnpublishAll,
							destroy: mockDestroy
						};
					})
				};
			});

			compiler = webpack(config);

			server = new Server(
				{
					port,
					bonjour: {
						type: "http",
						protocol: "udp"
					},
					server: {
						type: "https"
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

		it("should apply bonjour options", async () => {
			page
				.on("console", message => {
					consoleMessages.push(message);
				})
				.on("pageerror", error => {
					pageErrors.push(error);
				});

			const response = await page.goto(`https://127.0.0.1:${port}/`, {
				waitUntil: "networkidle0"
			});

			expect(mockPublish).toHaveBeenCalledTimes(1);

			expect(mockPublish).toHaveBeenCalledWith({
				name: `Webpack Dev Server ${os.hostname()}:${port}`,
				port,
				type: "http",
				protocol: "udp",
				subtypes: ["webpack"]
			});

			expect(mockUnpublishAll).toHaveBeenCalledTimes(0);
			expect(mockDestroy).toHaveBeenCalledTimes(0);

			expect(response.status()).toMatchSnapshot("response status");

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");
		});
	});
});
