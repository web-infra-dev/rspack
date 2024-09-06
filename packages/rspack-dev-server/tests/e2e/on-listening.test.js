"use strict";

const webpack = require("@rspack/core");
const { RspackDevServer: Server } = require("@rspack/dev-server");
const config = require("../fixtures/client-config/webpack.config");
const runBrowser = require("../helpers/run-browser");
const port = require("../helpers/ports-map")["on-listening-option"];

describe("onListening option", () => {
	let compiler;
	let server;
	let page;
	let browser;
	let pageErrors;
	let consoleMessages;
	let onListeningIsRunning = false;

	beforeEach(async () => {
		compiler = webpack(config);
		server = new Server(
			{
				onListening: devServer => {
					if (!devServer) {
						throw new Error("webpack-dev-server is not defined");
					}

					onListeningIsRunning = true;

					devServer.app.get("/listening/some/path", (_, response) => {
						response.send("listening");
					});

					devServer.app.post("/listening/some/path", (_, response) => {
						response.send("listening POST");
					});
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

	it("should handle GET request to /listening/some/path route", async () => {
		page
			.on("console", message => {
				consoleMessages.push(message);
			})
			.on("pageerror", error => {
				pageErrors.push(error);
			});

		const response = await page.goto(
			`http://127.0.0.1:${port}/listening/some/path`,
			{
				waitUntil: "networkidle0"
			}
		);

		expect(onListeningIsRunning).toBe(true);

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

	it("should handle POST request to /listening/some/path route", async () => {
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
			`http://127.0.0.1:${port}/listening/some/path`,
			{
				waitUntil: "networkidle0"
			}
		);

		expect(onListeningIsRunning).toBe(true);

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
