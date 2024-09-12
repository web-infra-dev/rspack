"use strict";

const webpack = require("@rspack/core");
const { RspackDevServer: Server } = require("@rspack/dev-server");
const config = require("../fixtures/simple-config/webpack.config");
const runBrowser = require("../helpers/run-browser");
const port = require("../helpers/ports-map")["setup-exit-signals-option"];

describe("setupExitSignals option", () => {
	describe("should handle 'SIGINT' and 'SIGTERM' signals", () => {
		let compiler;
		let server;
		let page;
		let browser;
		let pageErrors;
		let consoleMessages;
		let doExit;
		let exitSpy;
		let stopCallbackSpy;
		let stdinResumeSpy;
		let closeCallbackSpy;

		const signals = ["SIGINT", "SIGTERM"];

		beforeEach(async () => {
			compiler = webpack(config);

			server = new Server(
				{
					setupExitSignals: true,
					port
				},
				compiler
			);

			await server.start();

			({ page, browser } = await runBrowser());

			pageErrors = [];
			consoleMessages = [];
			doExit = false;

			exitSpy = jest.spyOn(process, "exit").mockImplementation(() => {
				doExit = true;
			});

			stdinResumeSpy = jest
				.spyOn(process.stdin, "resume")
				.mockImplementation(() => {});

			stopCallbackSpy = jest.spyOn(server, "stopCallback");

			if (server.compiler.close) {
				closeCallbackSpy = jest.spyOn(server.compiler, "close");
			}
		});

		afterEach(async () => {
			exitSpy.mockReset();
			stdinResumeSpy.mockReset();
			signals.forEach(signal => {
				process.removeAllListeners(signal);
			});
			process.stdin.removeAllListeners("end");
			await browser.close();
			await server.stop();
		});

		it.each(signals)("should close and exit on %s", async signal => {
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

			expect(response.status()).toMatchSnapshot("response status");

			process.emit(signal);

			await new Promise(resolve => {
				const interval = setInterval(() => {
					if (doExit) {
						expect(stopCallbackSpy.mock.calls.length).toEqual(1);

						if (server.compiler.close) {
							expect(closeCallbackSpy.mock.calls.length).toEqual(1);
						}

						clearInterval(interval);

						resolve();
					}
				}, 100);
			});

			consoleMessages = consoleMessages.filter(
				message =>
					!(
						message.text().includes("Trying to reconnect...") ||
						message.text().includes("Disconnected")
					)
			);

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");
		});
	});
});
