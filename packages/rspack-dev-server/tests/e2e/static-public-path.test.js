"use strict";

const path = require("path");
const webpack = require("@rspack/core");
const { RspackDevServer: Server } = require("@rspack/dev-server");
const config = require("../fixtures/static-config/webpack.config");
const port = require("../helpers/ports-map")["static-public-path-option"];
const runBrowser = require("../helpers/run-browser");

const staticDirectory = path.resolve(__dirname, "../fixtures/static-config");
const publicDirectory = path.resolve(staticDirectory, "public");
const otherPublicDirectory = path.resolve(staticDirectory, "other");
const staticPublicPath = "/serve-content-at-this-url";
const otherStaticPublicPath = "/serve-other-content-at-this-url";

describe("static.publicPath option", () => {
	describe("to directory", () => {
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
					static: {
						directory: publicDirectory,
						publicPath: staticPublicPath,
						watch: true
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

		it("should handle request to index", async () => {
			page
				.on("console", message => {
					consoleMessages.push(message);
				})
				.on("pageerror", error => {
					pageErrors.push(error);
				});

			const response = await page.goto(
				`http://127.0.0.1:${port}${staticPublicPath}/`,
				{
					waitUntil: "networkidle0"
				}
			);

			expect(response.status()).toMatchSnapshot("response status");

			expect(await response.text()).toMatchSnapshot("response text");

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");
		});

		it("should handle request to other file", async () => {
			page
				.on("console", message => {
					consoleMessages.push(message);
				})
				.on("pageerror", error => {
					pageErrors.push(error);
				});

			const response = await page.goto(
				`http://127.0.0.1:${port}${staticPublicPath}/other.html`,
				{
					waitUntil: "networkidle0"
				}
			);

			expect(response.status()).toMatchSnapshot("response status");

			expect(await response.text()).toMatchSnapshot("response text");

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");
		});
	});

	describe("test listing files in folders without index.html using the option static.serveIndex: false", () => {
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
					static: {
						directory: publicDirectory,
						publicPath: staticPublicPath,
						watch: true,
						serveIndex: false
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

		it("shouldn't list the files inside the assets folder (404)", async () => {
			page
				.on("console", message => {
					consoleMessages.push(message);
				})
				.on("pageerror", error => {
					pageErrors.push(error);
				});

			const response = await page.goto(
				`http://127.0.0.1:${port}${staticPublicPath}/assets`,
				{
					waitUntil: "networkidle0"
				}
			);

			expect(response.status()).toMatchSnapshot("response status");

			expect(await response.text()).toMatchSnapshot("response text");

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");
		});

		it("should show Heyo. because bar has index.html inside it (200)", async () => {
			page
				.on("console", message => {
					consoleMessages.push(message);
				})
				.on("pageerror", error => {
					pageErrors.push(error);
				});

			const response = await page.goto(
				`http://127.0.0.1:${port}${staticPublicPath}/bar`,
				{
					waitUntil: "networkidle0"
				}
			);

			expect(response.status()).toMatchSnapshot("response status");

			expect(await response.text()).toMatchSnapshot("response text");

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");
		});
	});

	describe("test listing files in folders without index.html using the option static.serveIndex: true", () => {
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
					static: {
						directory: publicDirectory,
						publicPath: staticPublicPath,
						watch: true,
						serveIndex: true
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

		it("should list the files inside the assets folder (200)", async () => {
			page
				.on("console", message => {
					consoleMessages.push(message);
				})
				.on("pageerror", error => {
					pageErrors.push(error);
				});

			const response = await page.goto(
				`http://127.0.0.1:${port}${staticPublicPath}/assets`,
				{
					waitUntil: "networkidle0"
				}
			);

			expect(response.status()).toMatchSnapshot("response status");

			expect(await response.text()).toContain("other.txt");

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");
		});

		it("should show Heyo. because bar has index.html inside it (200)", async () => {
			page
				.on("console", message => {
					consoleMessages.push(message);
				})
				.on("pageerror", error => {
					pageErrors.push(error);
				});

			const response = await page.goto(
				`http://127.0.0.1:${port}${staticPublicPath}/bar`,
				{
					waitUntil: "networkidle0"
				}
			);

			expect(response.status()).toMatchSnapshot("response status");

			expect(await response.text()).toMatchSnapshot("response text");

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");
		});
	});

	describe("test listing files in folders without index.html using the option static.serveIndex default (true)", () => {
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
					static: {
						directory: publicDirectory,
						publicPath: staticPublicPath,
						watch: true,
						serveIndex: true
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

		it("should list the files inside the assets folder (200)", async () => {
			page
				.on("console", message => {
					consoleMessages.push(message);
				})
				.on("pageerror", error => {
					pageErrors.push(error);
				});

			const response = await page.goto(
				`http://127.0.0.1:${port}${staticPublicPath}/assets`,
				{
					waitUntil: "networkidle0"
				}
			);

			expect(response.status()).toMatchSnapshot("response status");

			expect(await response.text()).toContain("other.txt");

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");
		});

		it("should show Heyo. because bar has index.html inside it (200)", async () => {
			page
				.on("console", message => {
					consoleMessages.push(message);
				})
				.on("pageerror", error => {
					pageErrors.push(error);
				});

			const response = await page.goto(
				`http://127.0.0.1:${port}${staticPublicPath}/bar`,
				{
					waitUntil: "networkidle0"
				}
			);

			expect(response.status()).toMatchSnapshot("response status");

			expect(await response.text()).toMatchSnapshot("response text");

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");
		});
	});

	describe("to multiple directories", () => {
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
					static: [
						{
							directory: publicDirectory,
							publicPath: staticPublicPath
						},
						{
							directory: otherPublicDirectory,
							publicPath: staticPublicPath
						}
					],
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

		it("should handle request to first directory", async () => {
			page
				.on("console", message => {
					consoleMessages.push(message);
				})
				.on("pageerror", error => {
					pageErrors.push(error);
				});

			const response = await page.goto(
				`http://127.0.0.1:${port}${staticPublicPath}/`,
				{
					waitUntil: "networkidle0"
				}
			);

			expect(response.status()).toMatchSnapshot("response status");

			expect(await response.text()).toMatchSnapshot("response text");

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");
		});

		it("should handle request to second directory", async () => {
			page
				.on("console", message => {
					consoleMessages.push(message);
				})
				.on("pageerror", error => {
					pageErrors.push(error);
				});

			const response = await page.goto(
				`http://127.0.0.1:${port}${staticPublicPath}/foo.html`,
				{
					waitUntil: "networkidle0"
				}
			);

			expect(response.status()).toMatchSnapshot("response status");

			expect(await response.text()).toMatchSnapshot("response text");

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");
		});
	});

	describe("defaults to CWD", () => {
		let cwdSpy;
		let compiler;
		let server;
		let page;
		let browser;
		let pageErrors;
		let consoleMessages;

		beforeEach(async () => {
			cwdSpy = jest
				.spyOn(process, "cwd")
				.mockImplementation(() => staticDirectory);

			compiler = webpack(config);

			server = new Server(
				{
					static: {
						publicPath: staticPublicPath
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
			cwdSpy.mockRestore();

			await browser.close();
			await server.stop();
		});

		it("should handle request to page", async () => {
			page
				.on("console", message => {
					consoleMessages.push(message);
				})
				.on("pageerror", error => {
					pageErrors.push(error);
				});

			const response = await page.goto(
				`http://127.0.0.1:${port}${staticPublicPath}/index.html`,
				{
					waitUntil: "networkidle0"
				}
			);

			expect(response.status()).toMatchSnapshot("response status");

			expect(await response.text()).toMatchSnapshot("response text");

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");
		});
	});

	describe("Content type", () => {
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
					static: {
						directory: publicDirectory,
						publicPath: staticPublicPath
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

		it("should handle request to example.txt", async () => {
			page
				.on("console", message => {
					consoleMessages.push(message);
				})
				.on("pageerror", error => {
					pageErrors.push(error);
				});

			const response = await page.goto(
				`http://127.0.0.1:${port}${staticPublicPath}/assets/example.txt`,
				{
					waitUntil: "networkidle0"
				}
			);

			expect(response.status()).toMatchSnapshot("response status");

			expect(response.headers()["content-type"]).toMatchSnapshot(
				"response header content-type"
			);

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");
		});
	});

	describe("should ignore methods other than GET and HEAD", () => {
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
					static: {
						directory: publicDirectory,
						publicPath: staticPublicPath
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

		it("should handle GET request", async () => {
			page
				.on("console", message => {
					consoleMessages.push(message);
				})
				.on("pageerror", error => {
					pageErrors.push(error);
				});

			const response = await page.goto(
				`http://127.0.0.1:${port}${staticPublicPath}/`,
				{
					waitUntil: "networkidle0"
				}
			);

			expect(response.status()).toMatchSnapshot("response status");

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");
		});

		it("should handle HEAD request", async () => {
			page
				.on("console", message => {
					consoleMessages.push(message);
				})
				.on("pageerror", error => {
					pageErrors.push(error);
				})
				.on("request", interceptedRequest => {
					if (interceptedRequest.isInterceptResolutionHandled()) return;

					interceptedRequest.continue({ method: "HEAD" });
				});

			const response = await page.goto(
				`http://127.0.0.1:${port}${staticPublicPath}/`,
				{
					waitUntil: "networkidle0"
				}
			);

			expect(response.status()).toMatchSnapshot("response status");

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");
		});

		it("should not handle POST request", async () => {
			await page.setRequestInterception(true);

			page
				.on("console", message => {
					consoleMessages.push(message);
				})
				.on("pageerror", error => {
					pageErrors.push(error);
				})
				.on("request", interceptedRequest => {
					interceptedRequest.continue({ method: "POST" });
				});

			const response = await page.goto(
				`http://127.0.0.1:${port}${staticPublicPath}/`,
				{
					waitUntil: "networkidle0"
				}
			);

			expect(response.status()).toMatchSnapshot("response status");

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");
		});

		it("should not handle PUT request", async () => {
			await page.setRequestInterception(true);

			page
				.on("console", message => {
					consoleMessages.push(message);
				})
				.on("pageerror", error => {
					pageErrors.push(error);
				})
				.on("request", interceptedRequest => {
					interceptedRequest.continue({ method: "PUT" });
				});

			const response = await page.goto(
				`http://127.0.0.1:${port}${staticPublicPath}/`,
				{
					waitUntil: "networkidle0"
				}
			);

			expect(response.status()).toMatchSnapshot("response status");

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");
		});

		it("should not handle DELETE request", async () => {
			await page.setRequestInterception(true);

			page
				.on("console", message => {
					consoleMessages.push(message);
				})
				.on("pageerror", error => {
					pageErrors.push(error);
				})
				.on("request", interceptedRequest => {
					interceptedRequest.continue({ method: "DELETE" });
				});

			const response = await page.goto(
				`http://127.0.0.1:${port}${staticPublicPath}/`,
				{
					waitUntil: "networkidle0"
				}
			);

			expect(response.status()).toMatchSnapshot("response status");

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");
		});

		it("should not handle PATCH request", async () => {
			await page.setRequestInterception(true);

			page
				.on("console", message => {
					consoleMessages.push(message);
				})
				.on("pageerror", error => {
					pageErrors.push(error);
				})
				.on("request", interceptedRequest => {
					interceptedRequest.continue({ method: "PATCH" });
				});

			const response = await page.goto(
				`http://127.0.0.1:${port}${staticPublicPath}/`,
				{
					waitUntil: "networkidle0"
				}
			);

			expect(response.status()).toMatchSnapshot("response status");

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");
		});
	});

	describe("multiple static.publicPath entries", () => {
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
					static: [
						{
							directory: publicDirectory,
							publicPath: staticPublicPath,
							watch: true
						},
						{
							directory: otherPublicDirectory,
							publicPath: otherStaticPublicPath,
							watch: true
						}
					],
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

		it("should handle request to the index of first path", async () => {
			page
				.on("console", message => {
					consoleMessages.push(message);
				})
				.on("pageerror", error => {
					pageErrors.push(error);
				});

			const response = await page.goto(
				`http://127.0.0.1:${port}${staticPublicPath}/`,
				{
					waitUntil: "networkidle0"
				}
			);

			expect(response.status()).toMatchSnapshot("response status");

			expect(await response.text()).toMatchSnapshot("response text");

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");
		});

		it("should handle request to the other file of first path", async () => {
			page
				.on("console", message => {
					consoleMessages.push(message);
				})
				.on("pageerror", error => {
					pageErrors.push(error);
				});

			const response = await page.goto(
				`http://127.0.0.1:${port}${staticPublicPath}/other.html`,
				{
					waitUntil: "networkidle0"
				}
			);

			expect(response.status()).toMatchSnapshot("response status");

			expect(await response.text()).toMatchSnapshot("response text");

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");
		});

		it("should handle request to the /foo route of second path", async () => {
			page
				.on("console", message => {
					consoleMessages.push(message);
				})
				.on("pageerror", error => {
					pageErrors.push(error);
				});

			const response = await page.goto(
				`http://127.0.0.1:${port}${otherStaticPublicPath}/foo.html`,
				{
					waitUntil: "networkidle0"
				}
			);

			expect(response.status()).toMatchSnapshot("response status");

			expect(await response.text()).toMatchSnapshot("response text");

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");
		});
	});

	describe("multiple static.publicPath entries with publicPath array", () => {
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
					static: [
						{
							directory: publicDirectory,
							publicPath: staticPublicPath,
							watch: true
						},
						{
							directory: otherPublicDirectory,
							publicPath: [staticPublicPath, otherStaticPublicPath],
							watch: true
						}
					],
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

		it("should handle request to the index of first path", async () => {
			page
				.on("console", message => {
					consoleMessages.push(message);
				})
				.on("pageerror", error => {
					pageErrors.push(error);
				});

			const response = await page.goto(
				`http://127.0.0.1:${port}${staticPublicPath}/`,
				{
					waitUntil: "networkidle0"
				}
			);

			expect(response.status()).toMatchSnapshot("response status");

			expect(await response.text()).toMatchSnapshot("response text");

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");
		});

		it("should handle request to the other file of first path", async () => {
			page
				.on("console", message => {
					consoleMessages.push(message);
				})
				.on("pageerror", error => {
					pageErrors.push(error);
				});

			const response = await page.goto(
				`http://127.0.0.1:${port}${staticPublicPath}/other.html`,
				{
					waitUntil: "networkidle0"
				}
			);

			expect(response.status()).toMatchSnapshot("response status");

			expect(await response.text()).toMatchSnapshot("response text");

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");
		});

		it("should handle request to the /foo route of first path", async () => {
			page
				.on("console", message => {
					consoleMessages.push(message);
				})
				.on("pageerror", error => {
					pageErrors.push(error);
				});

			const response = await page.goto(
				`http://127.0.0.1:${port}${staticPublicPath}/foo.html`,
				{
					waitUntil: "networkidle0"
				}
			);

			expect(response.status()).toMatchSnapshot("response status");

			expect(await response.text()).toMatchSnapshot("response text");

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");
		});

		it("should handle request to the /foo route of second path", async () => {
			page
				.on("console", message => {
					consoleMessages.push(message);
				})
				.on("pageerror", error => {
					pageErrors.push(error);
				});

			const response = await page.goto(
				`http://127.0.0.1:${port}${otherStaticPublicPath}/foo.html`,
				{
					waitUntil: "networkidle0"
				}
			);

			expect(response.status()).toMatchSnapshot("response status");

			expect(await response.text()).toMatchSnapshot("response text");

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");
		});
	});
});
