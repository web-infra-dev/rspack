"use strict";

const webpack = require("@rspack/core");
const { RspackDevServer: Server } = require("@rspack/dev-server");
const config = require("../fixtures/client-config/webpack.config");
const runBrowser = require("../helpers/run-browser");
const port = require("../helpers/ports-map")["setup-middlewares-option"];

describe("setupMiddlewares option", () => {
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
				setupMiddlewares: (middlewares, devServer) => {
					if (!devServer) {
						throw new Error("webpack-dev-server is not defined");
					}

					devServer.app.get("/setup-middleware/some/path", (_, response) => {
						response.send("setup-middlewares option GET");
					});

					devServer.app.post("/setup-middleware/some/path", (_, response) => {
						response.send("setup-middlewares option POST");
					});

					middlewares.push({
						name: "hello-world-test-two",
						middleware: (req, res, next) => {
							if (req.path !== "/foo/bar/baz") {
								next();

								return;
							}

							res.send("Hello World without path!");
						}
					});
					middlewares.push({
						name: "hello-world-test-one",
						path: "/foo/bar",
						middleware: (req, res) => {
							res.send("Hello World with path!");
						}
					});
					middlewares.push((req, res) => {
						res.send("Hello World as function!");
					});

					return middlewares;
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

	it("should handle GET request to /setup-middleware/some/path route", async () => {
		page
			.on("console", message => {
				consoleMessages.push(message);
			})
			.on("pageerror", error => {
				pageErrors.push(error);
			});

		const response = await page.goto(
			`http://127.0.0.1:${port}/setup-middleware/some/path`,
			{
				waitUntil: "networkidle0"
			}
		);

		expect(response.headers()["content-type"]).toMatchSnapshot(
			"response headers content-type"
		);
		expect(response.status()).toMatchSnapshot("response status");
		expect(await response.text()).toMatchSnapshot("response text");

		const response1 = await page.goto(`http://127.0.0.1:${port}/foo/bar`, {
			waitUntil: "networkidle0"
		});

		expect(response1.headers()["content-type"]).toMatchSnapshot(
			"response headers content-type"
		);
		expect(response1.status()).toMatchSnapshot("response status");
		expect(await response1.text()).toMatchSnapshot("response text");

		const response2 = await page.goto(`http://127.0.0.1:${port}/foo/bar/baz`, {
			waitUntil: "networkidle0"
		});

		expect(response2.headers()["content-type"]).toMatchSnapshot(
			"response headers content-type"
		);
		expect(response2.status()).toMatchSnapshot("response status");
		expect(await response2.text()).toMatchSnapshot("response text");

		const response3 = await page.goto(
			`http://127.0.0.1:${port}/setup-middleware/unknown`,
			{
				waitUntil: "networkidle0"
			}
		);

		expect(response3.headers()["content-type"]).toMatchSnapshot(
			"response headers content-type"
		);
		expect(response3.status()).toMatchSnapshot("response status");
		expect(await response3.text()).toMatchSnapshot("response text");

		expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
			"console messages"
		);
		expect(pageErrors).toMatchSnapshot("page errors");
	});

	it("should handle POST request to /setup-middleware/some/path route", async () => {
		await page.setRequestInterception(true);

		page
			.on("console", message => {
				consoleMessages.push(message);
			})
			.on("pageerror", error => {
				pageErrors.push(error);
			})
			.on("request", interceptedRequest => {
				if (interceptedRequest.isInterceptResolutionHandled()) return;

				interceptedRequest.continue({ method: "POST" });
			});

		const response = await page.goto(
			`http://127.0.0.1:${port}/setup-middleware/some/path`,
			{
				waitUntil: "networkidle0"
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
