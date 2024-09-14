"use strict";

const WebSocket = require("ws");
const webpack = require("@rspack/core");
const { RspackDevServer: Server } = require("@rspack/dev-server");
const WebsocketServer = require("webpack-dev-server/lib/servers/WebsocketServer");
const config = require("../fixtures/client-config/webpack.config");
const runBrowser = require("../helpers/run-browser");
const port = require("../helpers/ports-map")["web-socket-communication"];

jest.setTimeout(60000);

describe("web socket communication", () => {
	const webSocketServers = ["ws", "sockjs"];

	webSocketServers.forEach(websocketServer => {
		it(`should work and close web socket client connection when web socket server closed ("${websocketServer}")`, async () => {
			WebsocketServer.heartbeatInterval = 100;

			const compiler = webpack(config);
			const devServerOptions = {
				port,
				webSocketServer: websocketServer
			};
			const server = new Server(devServerOptions, compiler);

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

				await page.goto(`http://127.0.0.1:${port}/`, {
					waitUntil: "networkidle0"
				});

				await server.stop();
				await new Promise(resolve => {
					const interval = setInterval(() => {
						if (
							consoleMessages.includes("[webpack-dev-server] Disconnected!")
						) {
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
			}
		});

		it(`should work and terminate client that is not alive ("${websocketServer}")`, async () => {
			WebsocketServer.heartbeatInterval = 100;

			const compiler = webpack(config);
			const devServerOptions = {
				port,
				webSocketServer: websocketServer
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

				await page.goto(`http://127.0.0.1:${port}/`, {
					waitUntil: "networkidle0"
				});
				await browser.close();

				// Wait heartbeat
				await new Promise(resolve => {
					setTimeout(() => {
						resolve();
					}, 200);
				});

				expect(server.webSocketServer.clients.length).toBe(0);
				expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
					"console messages"
				);
				expect(pageErrors).toMatchSnapshot("page errors");
			} catch (error) {
				throw error;
			} finally {
				await server.stop();
			}
		});

		it(`should work and reconnect when the connection is lost ("${websocketServer}")`, async () => {
			WebsocketServer.heartbeatInterval = 100;

			const compiler = webpack(config);
			const devServerOptions = {
				port,
				webSocketServer: websocketServer
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

				await page.goto(`http://127.0.0.1:${port}/`, {
					waitUntil: "networkidle0"
				});

				await server.stop();
				await server.start();

				await page.waitForNavigation({
					waitUntil: "networkidle0"
				});

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

	it(`should work and do heartbeat using ("ws" web socket server)`, async () => {
		WebsocketServer.heartbeatInterval = 100;

		const compiler = webpack(config);
		const devServerOptions = {
			port,
			webSocketServer: "ws"
		};
		const server = new Server(devServerOptions, compiler);

		await server.start();

		server.webSocketServer.heartbeatInterval = 100;

		await new Promise((resolve, reject) => {
			const ws = new WebSocket(`ws://127.0.0.1:${devServerOptions.port}/ws`, {
				headers: {
					host: `127.0.0.1:${devServerOptions.port}`,
					origin: `http://127.0.0.1:${devServerOptions.port}`
				}
			});

			let opened = false;
			let received = false;

			ws.on("open", () => {
				opened = true;
			});

			ws.on("error", error => {
				reject(error);
			});

			ws.on("ping", () => {
				if (opened && received) {
					ws.close();
				}
			});

			ws.on("message", data => {
				const message = JSON.parse(data);

				if (message.type === "ok") {
					received = true;
				}
			});

			ws.on("close", () => {
				resolve();
			});
		});

		await server.stop();
	});
});
