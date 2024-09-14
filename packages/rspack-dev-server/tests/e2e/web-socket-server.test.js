"use strict";

const webpack = require("@rspack/core");
const { RspackDevServer: Server } = require("@rspack/dev-server");
const config = require("../fixtures/client-config/webpack.config");
const runBrowser = require("../helpers/run-browser");
const sessionSubscribe = require("../helpers/session-subscribe");
const port = require("../helpers/ports-map")["web-socket-server-test"];

describe("web socket server", () => {
	it("should work allow to disable", async () => {
		const devServerPort = port;

		const compiler = webpack(config);
		const devServerOptions = {
			webSocketServer: false,
			port: devServerPort
		};
		const server = new Server(devServerOptions, compiler);

		await server.start();

		const { page, browser } = await runBrowser();

		try {
			const pageErrors = [];
			const consoleMessages = [];

			page
				.on("console", message => {
					consoleMessages.push(message);
				})
				.on("pageerror", error => {
					pageErrors.push(error);
				});

			const webSocketRequests = [];
			const session = await page.target().createCDPSession();

			session.on("Network.webSocketCreated", test => {
				webSocketRequests.push(test);
			});

			await session.send("Target.setAutoAttach", {
				autoAttach: true,
				flatten: true,
				waitForDebuggerOnStart: true
			});

			sessionSubscribe(session);

			await page.goto(`http://127.0.0.1:${port}/`, {
				waitUntil: "networkidle0"
			});

			expect(webSocketRequests).toHaveLength(0);
			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);
			expect(pageErrors).toMatchSnapshot("page errors");
		} catch (error) {
			throw error;
		} finally {
			await browser.close();
			await server.stop();
		}
	});
});
