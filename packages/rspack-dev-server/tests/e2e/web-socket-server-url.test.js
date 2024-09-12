"use strict";

const express = require("express");
const webpack = require("@rspack/core");
const { createProxyMiddleware } = require("http-proxy-middleware");
const { RspackDevServer: Server } = require("@rspack/dev-server");
const config = require("../fixtures/client-config/webpack.config");
const runBrowser = require("../helpers/run-browser");
const sessionSubscribe = require("../helpers/session-subscribe");
const [port1, port2] = require("../helpers/ports-map")["web-socket-server-url"];

const webSocketServers = ["ws", "sockjs"];

describe("web socket server URL", () => {
	for (const webSocketServer of webSocketServers) {
		const websocketURLProtocol = webSocketServer === "ws" ? "ws" : "http";

		it(`should work behind proxy, when hostnames are same and ports are different ("${webSocketServer}")`, async () => {
			const devServerHost = "127.0.0.1";
			const devServerPort = port1;
			const proxyHost = devServerHost;
			const proxyPort = port2;

			const compiler = webpack(config);
			const devServerOptions = {
				webSocketServer,
				port: devServerPort,
				host: devServerHost,
				allowedHosts: "all"
			};
			const server = new Server(devServerOptions, compiler);

			await server.start();

			function startProxy(callback) {
				const app = express();
				app.use(
					"/",
					createProxyMiddleware({
						target: `http://${devServerHost}:${devServerPort}`,
						ws: true,
						changeOrigin: true,
						logLevel: "warn"
					})
				);

				return app.listen(proxyPort, proxyHost, callback);
			}

			const proxy = await new Promise(resolve => {
				const proxyCreated = startProxy(() => {
					resolve(proxyCreated);
				});
			});

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

				if (webSocketServer === "ws") {
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
				} else {
					page.on("request", request => {
						if (/\/ws\//.test(request.url())) {
							webSocketRequests.push({ url: request.url() });
						}
					});
				}

				await page.goto(`http://${proxyHost}:${proxyPort}/`, {
					waitUntil: "networkidle0"
				});
				const webSocketRequest = webSocketRequests[0];

				expect(webSocketRequest.url).toContain(
					`${websocketURLProtocol}://${devServerHost}:${devServerPort}/ws`
				);
				expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
					"console messages"
				);
				expect(pageErrors).toMatchSnapshot("page errors");
			} catch (error) {
				throw error;
			} finally {
				proxy.close();
				await browser.close();
				await server.stop();
			}
		});

		it(`should work behind proxy, when hostnames are different and ports are same ("${webSocketServer}")`, async () => {
			const devServerHost = "127.0.0.1";
			const devServerPort = port1;
			const proxyHost = Server.internalIPSync("v4");
			const proxyPort = port1;

			const compiler = webpack(config);
			const devServerOptions = {
				webSocketServer,
				port: devServerPort,
				host: devServerHost,
				allowedHosts: "all"
			};
			const server = new Server(devServerOptions, compiler);

			await server.start();

			function startProxy(callback) {
				const app = express();
				app.use(
					"/",
					createProxyMiddleware({
						target: `http://${devServerHost}:${devServerPort}`,
						ws: true,
						changeOrigin: true,
						logLevel: "warn"
					})
				);

				return app.listen(proxyPort, proxyHost, callback);
			}

			const proxy = await new Promise(resolve => {
				const proxyCreated = startProxy(() => {
					resolve(proxyCreated);
				});
			});

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

				if (webSocketServer === "ws") {
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
				} else {
					page.on("request", request => {
						if (/\/ws\//.test(request.url())) {
							webSocketRequests.push({ url: request.url() });
						}
					});
				}

				await page.goto(`http://${proxyHost}:${proxyPort}/`, {
					waitUntil: "networkidle0"
				});

				const webSocketRequest = webSocketRequests[0];

				expect(webSocketRequest.url).toContain(
					`${websocketURLProtocol}://${devServerHost}:${devServerPort}/ws`
				);
				expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
					"console messages"
				);
				expect(pageErrors).toMatchSnapshot("page errors");
			} catch (error) {
				throw error;
			} finally {
				proxy.close();
				await browser.close();
				await server.stop();
			}
		});

		it(`should work behind proxy, when hostnames are different and ports are different ("${webSocketServer}")`, async () => {
			const devServerHost = "127.0.0.1";
			const devServerPort = port1;
			const proxyHost = Server.internalIPSync("v4");
			const proxyPort = port2;

			const compiler = webpack(config);
			const devServerOptions = {
				client: {
					webSocketURL: {
						hostname: devServerHost
					}
				},
				webSocketServer,
				port: devServerPort,
				host: devServerHost,
				allowedHosts: "all"
			};
			const server = new Server(devServerOptions, compiler);

			await server.start();

			function startProxy(callback) {
				const app = express();
				app.use(
					"/",
					createProxyMiddleware({
						target: `http://${devServerHost}:${devServerPort}`,
						ws: true,
						changeOrigin: true,
						logLevel: "warn"
					})
				);

				return app.listen(proxyPort, proxyHost, callback);
			}

			const proxy = await new Promise(resolve => {
				const proxyCreated = startProxy(() => {
					resolve(proxyCreated);
				});
			});

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

				if (webSocketServer === "ws") {
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
				} else {
					page.on("request", request => {
						if (/\/ws\//.test(request.url())) {
							webSocketRequests.push({ url: request.url() });
						}
					});
				}
				await page.goto(`http://${proxyHost}:${proxyPort}/`, {
					waitUntil: "networkidle0"
				});

				const webSocketRequest = webSocketRequests[0];

				expect(webSocketRequest.url).toContain(
					`${websocketURLProtocol}://${devServerHost}:${devServerPort}/ws`
				);
				expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
					"console messages"
				);
				expect(pageErrors).toMatchSnapshot("page errors");
			} catch (error) {
				throw error;
			} finally {
				proxy.close();

				await browser.close();
				await server.stop();
			}
		});

		it(`should work behind proxy, when the "host" option is "local-ip" and the "port" option is "auto" ("${webSocketServer}")`, async () => {
			process.env.WEBPACK_DEV_SERVER_BASE_PORT = 40000;

			const proxyHost = Server.internalIPSync("v4");
			const proxyPort = port2;

			const compiler = webpack(config);
			const devServerOptions = {
				webSocketServer,
				port: "auto",
				host: "local-ip",
				allowedHosts: "all"
			};
			const server = new Server(devServerOptions, compiler);

			await server.start();

			const resolvedHost = server.options.host;
			const resolvedPort = server.options.port;

			function startProxy(callback) {
				const app = express();

				app.use(
					"/",
					createProxyMiddleware({
						target: `http://${resolvedHost}:${resolvedPort}`,
						ws: true,
						changeOrigin: true,
						logLevel: "warn"
					})
				);

				return app.listen(proxyPort, proxyHost, callback);
			}

			const proxy = await new Promise(resolve => {
				const proxyCreated = startProxy(() => {
					resolve(proxyCreated);
				});
			});

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

				if (webSocketServer === "ws") {
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
				} else {
					page.on("request", request => {
						if (/\/ws\//.test(request.url())) {
							webSocketRequests.push({ url: request.url() });
						}
					});
				}

				await page.goto(`http://${proxyHost}:${proxyPort}/`, {
					waitUntil: "networkidle0"
				});

				const webSocketRequest = webSocketRequests[0];

				expect(webSocketRequest.url).toContain(
					`${websocketURLProtocol}://${resolvedHost}:${resolvedPort}/ws`
				);
				expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
					"console messages"
				);
				expect(pageErrors).toMatchSnapshot("page errors");
			} catch (error) {
				throw error;
			} finally {
				proxy.close();

				await browser.close();
				await server.stop();

				delete process.env.WEBPACK_DEV_SERVER_BASE_PORT;
			}
		});

		it(`should work with the "client.webSocketURL.protocol" option ("${webSocketServer}")`, async () => {
			const compiler = webpack(config);
			const devServerOptions = {
				client: {
					webSocketURL: {
						protocol: "ws:"
					}
				},
				webSocketServer,
				port: port1,
				host: "0.0.0.0",
				allowedHosts: "all"
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

				if (webSocketServer === "ws") {
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
				} else {
					page.on("request", request => {
						if (/\/ws\//.test(request.url())) {
							webSocketRequests.push({ url: request.url() });
						}
					});
				}

				await page.goto(`http://localhost:${port1}/`, {
					waitUntil: "networkidle0"
				});

				const webSocketRequest = webSocketRequests[0];

				expect(webSocketRequest.url).toContain(
					`${websocketURLProtocol}://localhost:${port1}/ws`
				);
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

		it(`should work with the "client.webSocketURL.protocol" option using "auto:" value ("${webSocketServer}")`, async () => {
			const compiler = webpack(config);
			const devServerOptions = {
				client: {
					webSocketURL: {
						protocol: "auto:"
					}
				},
				webSocketServer,
				port: port1,
				host: "0.0.0.0",
				allowedHosts: "all"
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

				if (webSocketServer === "ws") {
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
				} else {
					page.on("request", request => {
						if (/\/ws\//.test(request.url())) {
							webSocketRequests.push({ url: request.url() });
						}
					});
				}

				await page.goto(`http://localhost:${port1}/`, {
					waitUntil: "networkidle0"
				});

				const webSocketRequest = webSocketRequests[0];

				expect(webSocketRequest.url).toContain(
					`${websocketURLProtocol}://localhost:${port1}/ws`
				);
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

		it(`should work with the "client.webSocketURL.protocol" option using "http:" value and covert to "ws:" ("${webSocketServer}")`, async () => {
			const compiler = webpack(config);
			const devServerOptions = {
				client: {
					webSocketURL: {
						protocol: "http:"
					}
				},
				webSocketServer,
				port: port1,
				host: "0.0.0.0",
				allowedHosts: "all"
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

				if (webSocketServer === "ws") {
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
				} else {
					page.on("request", request => {
						if (/\/ws\//.test(request.url())) {
							webSocketRequests.push({ url: request.url() });
						}
					});
				}

				await page.goto(`http://localhost:${port1}/`, {
					waitUntil: "networkidle0"
				});

				const webSocketRequest = webSocketRequests[0];

				expect(webSocketRequest.url).toContain(
					`${websocketURLProtocol}://localhost:${port1}/ws`
				);
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

		it(`should work with the "client.webSocketURL.host" option ("${webSocketServer}")`, async () => {
			const compiler = webpack(config);
			const devServerOptions = {
				client: {
					webSocketURL: {
						hostname: "127.0.0.1"
					}
				},
				webSocketServer,
				port: port1,
				host: "0.0.0.0",
				allowedHosts: "all"
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

				if (webSocketServer === "ws") {
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
				} else {
					page.on("request", request => {
						if (/\/ws\//.test(request.url())) {
							webSocketRequests.push({ url: request.url() });
						}
					});
				}

				await page.goto(`http://127.0.0.1:${port1}/`, {
					waitUntil: "networkidle0"
				});

				const webSocketRequest = webSocketRequests[0];

				expect(webSocketRequest.url).toContain(
					`${websocketURLProtocol}://127.0.0.1:${port1}/ws`
				);
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

		it(`should work with the "client.webSocketURL.host" option using "0.0.0.0" value ("${webSocketServer}")`, async () => {
			const compiler = webpack(config);
			const devServerOptions = {
				client: {
					webSocketURL: {
						hostname: "0.0.0.0"
					}
				},
				webSocketServer,
				port: port1,
				host: "0.0.0.0",
				allowedHosts: "all"
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

				if (webSocketServer === "ws") {
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
				} else {
					page.on("request", request => {
						if (/\/ws\//.test(request.url())) {
							webSocketRequests.push({ url: request.url() });
						}
					});
				}
				await page.goto(`http://127.0.0.1:${port1}/`, {
					waitUntil: "networkidle0"
				});

				const webSocketRequest = webSocketRequests[0];

				expect(webSocketRequest.url).toContain(
					`${websocketURLProtocol}://127.0.0.1:${port1}/ws`
				);
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

		it(`should work with the "client.webSocketURL.port" option ("${webSocketServer}")`, async () => {
			const compiler = webpack(config);
			const devServerOptions = {
				client: {
					webSocketURL: {
						port: port1
					}
				},
				webSocketServer,
				port: port1,
				host: "0.0.0.0",
				allowedHosts: "all"
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

				if (webSocketServer === "ws") {
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
				} else {
					page.on("request", request => {
						if (/\/ws\//.test(request.url())) {
							webSocketRequests.push({ url: request.url() });
						}
					});
				}

				await page.goto(`http://127.0.0.1:${port1}/`, {
					waitUntil: "networkidle0"
				});

				const webSocketRequest = webSocketRequests[0];

				expect(webSocketRequest.url).toContain(
					`${websocketURLProtocol}://127.0.0.1:${port1}/ws`
				);
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

		it(`should work with the "client.webSocketURL.port" option as string ("${webSocketServer}")`, async () => {
			const compiler = webpack(config);
			const devServerOptions = {
				client: {
					webSocketURL: {
						port: `${port1}`
					}
				},
				webSocketServer,
				port: port1,
				host: "0.0.0.0",
				allowedHosts: "all"
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

				if (webSocketServer === "ws") {
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
				} else {
					page.on("request", request => {
						if (/\/ws\//.test(request.url())) {
							webSocketRequests.push({ url: request.url() });
						}
					});
				}

				await page.goto(`http://127.0.0.1:${port1}/`, {
					waitUntil: "networkidle0"
				});

				const webSocketRequest = webSocketRequests[0];

				expect(webSocketRequest.url).toContain(
					`${websocketURLProtocol}://127.0.0.1:${port1}/ws`
				);
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

		it(`should work with "client.webSocketURL.port" and "webSocketServer.options.port" options as string ("${webSocketServer}")`, async () => {
			const compiler = webpack(config);
			const devServerOptions = {
				webSocketServer: {
					type: webSocketServer,
					options: {
						host: "0.0.0.0",
						// "sockjs" doesn't support external server
						port: webSocketServer === "sockjs" ? `${port1}` : `${port2}`
					}
				},
				port: port1,
				host: "0.0.0.0",
				client: {
					webSocketURL: {
						port: webSocketServer === "sockjs" ? `${port1}` : `${port2}`
					}
				},
				allowedHosts: "all"
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

				if (webSocketServer === "ws") {
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
				} else {
					page.on("request", request => {
						if (/\/ws\//.test(request.url())) {
							webSocketRequests.push({ url: request.url() });
						}
					});
				}

				await page.goto(`http://127.0.0.1:${port1}/`, {
					waitUntil: "networkidle0"
				});

				const webSocketRequest = webSocketRequests[0];

				expect(webSocketRequest.url).toContain(
					webSocketServer === "sockjs"
						? `${websocketURLProtocol}://127.0.0.1:${port1}/ws`
						: `${websocketURLProtocol}://127.0.0.1:${port2}/ws`
				);
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

		it(`should work with the "client.webSocketURL.port" option using "0" value ("${webSocketServer}")`, async () => {
			const compiler = webpack(config);
			const devServerOptions = {
				client: {
					webSocketURL: {
						port: 0
					}
				},
				webSocketServer,
				port: port1,
				host: "0.0.0.0",
				allowedHosts: "all"
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

				if (webSocketServer === "ws") {
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
				} else {
					page.on("request", request => {
						if (/\/ws\//.test(request.url())) {
							webSocketRequests.push({ url: request.url() });
						}
					});
				}
				await page.goto(`http://127.0.0.1:${port1}/`, {
					waitUntil: "networkidle0"
				});

				const webSocketRequest = webSocketRequests[0];

				expect(webSocketRequest.url).toContain(
					`${websocketURLProtocol}://127.0.0.1:${port1}/ws`
				);
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

		it(`should work with the "client.webSocketURL.pathname" option ("${webSocketServer}")`, async () => {
			const compiler = webpack(config);
			const devServerOptions = {
				client: {
					webSocketURL: {
						pathname: "/ws"
					}
				},
				webSocketServer,
				port: port1,
				host: "0.0.0.0",
				allowedHosts: "all"
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

				if (webSocketServer === "ws") {
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
				} else {
					page.on("request", request => {
						if (/\/ws\//.test(request.url())) {
							webSocketRequests.push({ url: request.url() });
						}
					});
				}

				await page.goto(`http://127.0.0.1:${port1}/`, {
					waitUntil: "networkidle0"
				});

				const webSocketRequest = webSocketRequests[0];

				expect(webSocketRequest.url).toContain(
					`${websocketURLProtocol}://127.0.0.1:${port1}/ws`
				);
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

		it(`should work with default "/ws" value of the "client.webSocketURL.pathname" option ("${webSocketServer}")`, async () => {
			const compiler = webpack(config);
			const devServerOptions = {
				webSocketServer,
				port: port1,
				host: "0.0.0.0",
				allowedHosts: "all"
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

				if (webSocketServer === "ws") {
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
				} else {
					page.on("request", request => {
						if (/\/ws\//.test(request.url())) {
							webSocketRequests.push({ url: request.url() });
						}
					});
				}

				await page.goto(`http://127.0.0.1:${port1}/`, {
					waitUntil: "networkidle0"
				});

				const webSocketRequest = webSocketRequests[0];

				expect(webSocketRequest.url).toContain(
					`${websocketURLProtocol}://127.0.0.1:${port1}/ws`
				);
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

		it(`should work with the "client.webSocketURL.username" option ("${webSocketServer}")`, async () => {
			const compiler = webpack(config);
			const devServerOptions = {
				client: {
					webSocketURL: {
						username: "zenitsu"
					}
				},
				webSocketServer,
				port: port1,
				host: "0.0.0.0",
				allowedHosts: "all"
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

				if (webSocketServer === "ws") {
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
				} else {
					page.on("request", request => {
						if (/\/ws\//.test(request.url())) {
							webSocketRequests.push({ url: request.url() });
						}
					});
				}
				await page.goto(`http://127.0.0.1:${port1}/`, {
					waitUntil: "networkidle0"
				});

				const webSocketRequest = webSocketRequests[0];

				expect(webSocketRequest.url).toContain(
					`${websocketURLProtocol}://zenitsu@127.0.0.1:${port1}/ws`
				);
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

		it(`should work with the "client.webSocketURL.password" option ("${webSocketServer}")`, async () => {
			const compiler = webpack(config);
			const devServerOptions = {
				client: {
					webSocketURL:
						webSocketServer === "ws"
							? {
									username: "foo",
									password: "chuntaro"
								}
							: {}
				},
				webSocketServer,
				port: port1,
				host: "0.0.0.0",
				allowedHosts: "all"
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

				if (webSocketServer === "ws") {
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
				} else {
					page.on("request", request => {
						if (/\/ws\//.test(request.url())) {
							webSocketRequests.push({ url: request.url() });
						}
					});
				}
				await page.goto(`http://127.0.0.1:${port1}/`, {
					waitUntil: "networkidle0"
				});

				const webSocketRequest = webSocketRequests[0];

				expect(webSocketRequest.url).toContain(
					// "sockjs" has bug with parsing URL
					webSocketServer === "ws"
						? `${websocketURLProtocol}://foo:chuntaro@127.0.0.1:${port1}/ws`
						: `${websocketURLProtocol}://127.0.0.1:${port1}/ws`
				);
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

		it(`should work with the "client.webSocketURL.username" and "client.webSocketURL.password" option ("${webSocketServer}")`, async () => {
			const compiler = webpack(config);
			const devServerOptions = {
				client: {
					webSocketURL: {
						username: "zenitsu",
						password: "chuntaro"
					}
				},
				webSocketServer,
				port: port1,
				host: "0.0.0.0",
				allowedHosts: "all"
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

				if (webSocketServer === "ws") {
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
				} else {
					page.on("request", request => {
						if (/\/ws\//.test(request.url())) {
							webSocketRequests.push({ url: request.url() });
						}
					});
				}

				await page.goto(`http://127.0.0.1:${port1}/`, {
					waitUntil: "networkidle0"
				});

				const webSocketRequest = webSocketRequests[0];

				expect(webSocketRequest.url).toContain(
					`${websocketURLProtocol}://zenitsu:chuntaro@127.0.0.1:${port1}/ws`
				);
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

		it(`should work with the custom web socket server "path" ("${webSocketServer}")`, async () => {
			const compiler = webpack(config);
			const devServerOptions = {
				webSocketServer: {
					type: webSocketServer,
					options: {
						path: "/custom-ws/foo/bar"
					}
				},
				port: port1,
				host: "0.0.0.0",
				allowedHosts: "all"
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

				if (webSocketServer === "ws") {
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
				} else {
					page.on("request", request => {
						if (/\/custom-ws\/foo\/bar/.test(request.url())) {
							webSocketRequests.push({ url: request.url() });
						}
					});
				}

				await page.goto(`http://127.0.0.1:${port1}/`, {
					waitUntil: "networkidle0"
				});

				const webSocketRequest = webSocketRequests[0];

				expect(webSocketRequest.url).toContain(
					`${websocketURLProtocol}://127.0.0.1:${port1}/custom-ws/foo/bar`
				);
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

		// Only works for "ws" server
		it(`should work with the custom web socket server "path" using empty value ("${webSocketServer}")`, async () => {
			const compiler = webpack(config);
			const devServerOptions = {
				webSocketServer: {
					type: webSocketServer,
					options: {
						path: webSocketServer === "ws" ? "" : "/custom-ws"
					}
				},
				port: port1,
				host: "0.0.0.0",
				allowedHosts: "all"
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

				if (webSocketServer === "ws") {
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
				} else {
					page.on("request", request => {
						if (/\/custom-ws\//.test(request.url())) {
							webSocketRequests.push({ url: request.url() });
						}
					});
				}

				await page.goto(`http://127.0.0.1:${port1}/`, {
					waitUntil: "networkidle0"
				});

				const webSocketRequest = webSocketRequests[0];

				expect(webSocketRequest.url).toContain(
					webSocketServer === "ws"
						? `${websocketURLProtocol}://127.0.0.1:${port1}`
						: `${websocketURLProtocol}://127.0.0.1:${port1}/custom-ws`
				);
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

		it(`should work with the "client.webSocketURL.pathname" option and the custom web socket server "path" ("${webSocketServer}")`, async () => {
			const compiler = webpack(config);
			const devServerOptions = {
				client: {
					webSocketURL: {
						pathname: "/custom-ws/foo/bar"
					}
				},
				webSocketServer: {
					type: webSocketServer,
					options: {
						path: "/custom-ws/foo/bar"
					}
				},
				port: port1,
				host: "0.0.0.0",
				allowedHosts: "all"
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

				if (webSocketServer === "ws") {
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
				} else {
					page.on("request", request => {
						if (/\/custom-ws\/foo\/bar/.test(request.url())) {
							webSocketRequests.push({ url: request.url() });
						}
					});
				}

				await page.goto(`http://127.0.0.1:${port1}/`, {
					waitUntil: "networkidle0"
				});

				const webSocketRequest = webSocketRequests[0];

				expect(webSocketRequest.url).toContain(
					`${websocketURLProtocol}://127.0.0.1:${port1}/custom-ws/foo/bar`
				);
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

		it(`should work with the "client.webSocketURL.pathname" option and the custom web socket server "path" ending without slash ("${webSocketServer}")`, async () => {
			const compiler = webpack(config);
			const devServerOptions = {
				client: {
					webSocketURL: {
						pathname: "/custom-ws"
					}
				},
				webSocketServer: {
					type: webSocketServer,
					options: {
						path: "/custom-ws"
					}
				},
				port: port1,
				host: "0.0.0.0",
				allowedHosts: "all"
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

				if (webSocketServer === "ws") {
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
				} else {
					page.on("request", request => {
						if (/\/custom-ws\//.test(request.url())) {
							webSocketRequests.push({ url: request.url() });
						}
					});
				}

				await page.goto(`http://127.0.0.1:${port1}/`, {
					waitUntil: "networkidle0"
				});

				const webSocketRequest = webSocketRequests[0];

				expect(webSocketRequest.url).toContain(
					`${websocketURLProtocol}://127.0.0.1:${port1}/custom-ws`
				);
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

		// Only works for "ws" server, "sockjs" adds "/" be default, because need do requests like "/custom-ws/info?t=1624462615772"
		it(`should work with the "client.webSocketURL.pathname" option and the custom web socket server "path" ending with slash ("${webSocketServer}")`, async () => {
			const compiler = webpack(config);
			const devServerOptions = {
				client: {
					webSocketURL: {
						pathname: webSocketServer === "ws" ? "/custom-ws/" : "/custom-ws"
					}
				},
				webSocketServer: {
					type: webSocketServer,
					options: {
						path: webSocketServer === "ws" ? "/custom-ws/" : "/custom-ws"
					}
				},
				port: port1,
				host: "0.0.0.0",
				allowedHosts: "all"
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

				if (webSocketServer === "ws") {
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
				} else {
					page.on("request", request => {
						if (/\/custom-ws\//.test(request.url())) {
							webSocketRequests.push({ url: request.url() });
						}
					});
				}
				await page.goto(`http://127.0.0.1:${port1}/`, {
					waitUntil: "networkidle0"
				});

				const webSocketRequest = webSocketRequests[0];

				expect(webSocketRequest.url).toContain(
					`${websocketURLProtocol}://127.0.0.1:${port1}/custom-ws/`
				);
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

		// Only works for "ws" server
		it(`should work with the "client.webSocketURL.pathname" option and the custom web socket server "path" using empty value ("${webSocketServer}")`, async () => {
			const compiler = webpack(config);
			const devServerOptions = {
				client: {
					webSocketURL: {
						pathname: webSocketServer === "ws" ? "" : "/custom-ws"
					}
				},
				webSocketServer: {
					type: webSocketServer,
					options: {
						path: webSocketServer === "ws" ? "" : "/custom-ws"
					}
				},
				port: port1,
				host: "0.0.0.0",
				allowedHosts: "all"
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

				if (webSocketServer === "ws") {
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
				} else {
					page.on("request", request => {
						if (/\/custom-ws\//.test(request.url())) {
							webSocketRequests.push({ url: request.url() });
						}
					});
				}

				await page.goto(`http://127.0.0.1:${port1}/`, {
					waitUntil: "networkidle0"
				});

				const webSocketRequest = webSocketRequests[0];

				expect(webSocketRequest.url).toContain(
					webSocketServer === "ws"
						? `${websocketURLProtocol}://127.0.0.1:${port1}`
						: `${websocketURLProtocol}://127.0.0.1:${port1}/custom-ws`
				);
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

		// Only works for "sockjs" server
		it(`should work with the "client.webSocketURL.pathname" option and the custom web socket server "prefix" for compatibility with "sockjs" ("${webSocketServer}")`, async () => {
			const compiler = webpack(config);
			const devServerOptions = {
				client: {
					webSocketURL: {
						pathname: "/custom-ws"
					}
				},
				webSocketServer: {
					type: webSocketServer,
					options:
						webSocketServer === "ws"
							? { path: "/custom-ws" }
							: { prefix: "/custom-ws" }
				},
				port: port1,
				host: "0.0.0.0",
				allowedHosts: "all"
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

				if (webSocketServer === "ws") {
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
				} else {
					page.on("request", request => {
						if (/\/custom-ws/.test(request.url())) {
							webSocketRequests.push({ url: request.url() });
						}
					});
				}

				await page.goto(`http://127.0.0.1:${port1}/`, {
					waitUntil: "networkidle0"
				});

				const webSocketRequest = webSocketRequests[0];

				expect(webSocketRequest.url).toContain(
					`${websocketURLProtocol}://127.0.0.1:${port1}/custom-ws`
				);
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

		it(`should work when "host" option is IPv4 ("${webSocketServer}")`, async () => {
			const hostname = Server.internalIPSync("v4");
			const compiler = webpack(config);
			const devServerOptions = {
				webSocketServer,
				port: port1,
				host: hostname
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

				if (webSocketServer === "ws") {
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
				} else {
					page.on("request", request => {
						if (/\/ws\//.test(request.url())) {
							webSocketRequests.push({ url: request.url() });
						}
					});
				}
				await page.goto(`http://${hostname}:${port1}/`, {
					waitUntil: "networkidle0"
				});

				const webSocketRequest = webSocketRequests[0];

				expect(webSocketRequest.url).toContain(
					`${websocketURLProtocol}://${hostname}:${port1}/ws`
				);
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

		it(`should work when "host" option is "local-ip" ("${webSocketServer}")`, async () => {
			const hostname = Server.internalIPSync("v4");
			const compiler = webpack(config);
			const devServerOptions = {
				webSocketServer,
				port: port1,
				host: "local-ip"
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

				if (webSocketServer === "ws") {
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
				} else {
					page.on("request", request => {
						if (/\/ws\//.test(request.url())) {
							webSocketRequests.push({ url: request.url() });
						}
					});
				}

				await page.goto(`http://${hostname}:${port1}/`, {
					waitUntil: "networkidle0"
				});

				const webSocketRequest = webSocketRequests[0];

				expect(webSocketRequest.url).toContain(
					`${websocketURLProtocol}://${hostname}:${port1}/ws`
				);
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

		it(`should work when "host" option is "local-ipv4" ("${webSocketServer}")`, async () => {
			const hostname = Server.internalIPSync("v4");
			const compiler = webpack(config);
			const devServerOptions = {
				webSocketServer,
				port: port1,
				host: "local-ipv4"
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

				if (webSocketServer === "ws") {
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
				} else {
					page.on("request", request => {
						if (/\/ws\//.test(request.url())) {
							webSocketRequests.push({ url: request.url() });
						}
					});
				}
				await page.goto(`http://${hostname}:${port1}/`, {
					waitUntil: "networkidle0"
				});

				const webSocketRequest = webSocketRequests[0];

				expect(webSocketRequest.url).toContain(
					`${websocketURLProtocol}://${hostname}:${port1}/ws`
				);
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

		it(`should work with "server: 'https'" option ("${webSocketServer}")`, async () => {
			const hostname = "127.0.0.1";
			const compiler = webpack(config);
			const devServerOptions = {
				webSocketServer,
				port: port1,
				server: "https"
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

				if (webSocketServer === "ws") {
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
				} else {
					page.on("request", request => {
						if (/\/ws\//.test(request.url())) {
							webSocketRequests.push({ url: request.url() });
						}
					});
				}

				await page.goto(`https://${hostname}:${port1}/`, {
					waitUntil: "networkidle0"
				});

				const webSocketRequest = webSocketRequests[0];

				if (webSocketServer === "ws") {
					expect(webSocketRequest.url).toContain(
						`wss://${hostname}:${port1}/ws`
					);
				} else {
					expect(webSocketRequest.url).toContain(
						`https://${hostname}:${port1}/ws`
					);
				}

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

		it(`should work with "server: 'spdy'" option ("${webSocketServer}")`, async () => {
			const hostname = "127.0.0.1";
			const compiler = webpack(config);
			const devServerOptions = {
				webSocketServer,
				port: port1,
				server: "spdy"
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

				if (webSocketServer === "ws") {
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
				} else {
					page.on("request", request => {
						if (/\/ws\//.test(request.url())) {
							webSocketRequests.push({ url: request.url() });
						}
					});
				}

				await page.goto(`https://${hostname}:${port1}/`, {
					waitUntil: "networkidle0"
				});

				const webSocketRequest = webSocketRequests[0];

				if (webSocketServer === "ws") {
					expect(webSocketRequest.url).toContain(
						`wss://${hostname}:${port1}/ws`
					);
				} else {
					expect(webSocketRequest.url).toContain(
						`https://${hostname}:${port1}/ws`
					);
				}

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

		it(`should work when "port" option is "auto" ("${webSocketServer}")`, async () => {
			process.env.WEBPACK_DEV_SERVER_BASE_PORT = 50000;

			const compiler = webpack(config);
			const devServerOptions = {
				webSocketServer,
				port: "auto",
				host: "0.0.0.0"
			};
			const server = new Server(devServerOptions, compiler);

			await server.start();

			const resolvedFreePort = server.options.port;

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

				if (webSocketServer === "ws") {
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
				} else {
					page.on("request", request => {
						if (/\/ws\//.test(request.url())) {
							webSocketRequests.push({ url: request.url() });
						}
					});
				}

				await page.goto(`http://127.0.0.1:${resolvedFreePort}/`, {
					waitUntil: "networkidle0"
				});

				const webSocketRequest = webSocketRequests[0];

				expect(webSocketRequest.url).toContain(
					`${websocketURLProtocol}://127.0.0.1:${resolvedFreePort}/ws`
				);
				expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
					"console messages"
				);
				expect(pageErrors).toMatchSnapshot("page errors");
			} catch (error) {
				throw error;
			} finally {
				await browser.close();
				await server.stop();

				delete process.env.WEBPACK_DEV_SERVER_BASE_PORT;
			}
		});

		it(`should work with "client.webSocketURL.*" options ("${webSocketServer}")`, async () => {
			const compiler = webpack(config);
			const devServerOptions = {
				client: {
					webSocketURL: {
						protocol: "ws:",
						hostname: "127.0.0.1",
						port: port1,
						pathname: "/ws"
					}
				},
				webSocketServer,
				port: port1,
				host: "0.0.0.0",
				allowedHosts: "all"
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

				if (webSocketServer === "ws") {
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
				} else {
					page.on("request", request => {
						if (/\/ws\//.test(request.url())) {
							webSocketRequests.push({ url: request.url() });
						}
					});
				}

				await page.goto(`http://127.0.0.1:${port1}/`, {
					waitUntil: "networkidle0"
				});

				const webSocketRequest = webSocketRequests[0];

				expect(webSocketRequest.url).toContain(
					`${websocketURLProtocol}://127.0.0.1:${port1}/ws`
				);
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

		it(`should work with the "client.webSocketURL" option as "string" ("${webSocketServer}")`, async () => {
			const compiler = webpack(config);
			const devServerOptions = {
				client: {
					webSocketURL: `ws://127.0.0.1:${port1}/ws`
				},
				webSocketServer,
				port: port1,
				host: "0.0.0.0",
				allowedHosts: "all"
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

				if (webSocketServer === "ws") {
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
				} else {
					page.on("request", request => {
						if (/\/ws\//.test(request.url())) {
							webSocketRequests.push({ url: request.url() });
						}
					});
				}

				await page.goto(`http://127.0.0.1:${port1}/`, {
					waitUntil: "networkidle0"
				});

				const webSocketRequest = webSocketRequests[0];

				expect(webSocketRequest.url).toContain(
					`${websocketURLProtocol}://127.0.0.1:${port1}/ws`
				);
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

		it(`should work and throw an error on invalid web socket URL ("${webSocketServer}")`, async () => {
			const compiler = webpack(config);
			const devServerOptions = {
				client: {
					webSocketURL: "unknown://unknown.unknown/unknown"
				},
				webSocketServer,
				port: port1,
				host: "0.0.0.0",
				allowedHosts: "all"
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

				await page.goto(`http://localhost:${port1}/`, {
					waitUntil: "networkidle0"
				});

				expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
					"console messages"
				);
				expect(
					pageErrors.map(pageError => pageError.message.split("\n")[0])
				).toMatchSnapshot("page errors");
			} catch (error) {
				throw error;
			} finally {
				await browser.close();
				await server.stop();
			}
		});

		it(`should not work and output disconnect wrong web socket URL ("${webSocketServer}")`, async () => {
			const compiler = webpack(config);
			const devServerOptions = {
				client: {
					webSocketURL: "ws://unknown.unknown/unknown"
				},
				webSocketServer,
				port: port1,
				host: "0.0.0.0",
				allowedHosts: "all"
			};
			const server = new Server(devServerOptions, compiler);

			await server.start();

			const { page, browser } = await runBrowser();

			try {
				const pageErrors = [];
				const consoleMessages = [];

				let isDisconnected = false;

				page
					.on("console", message => {
						const text = message.text();

						if (!isDisconnected) {
							isDisconnected = /Disconnected!/.test(text);
							consoleMessages.push(text.replace(/:[\d]+/g, ":<port>"));
						}
					})
					.on("pageerror", error => {
						pageErrors.push(error);
					});

				await page.goto(`http://localhost:${port1}/`, {
					waitUntil: "networkidle0"
				});

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

				// TODO: not stable on lynx linux ci
				// expect(consoleMessages).toMatchSnapshot("console messages");
				expect(
					pageErrors.map(pageError => pageError.message.split("\n")[0])
				).toMatchSnapshot("page errors");
			} catch (error) {
				throw error;
			} finally {
				await browser.close();
				await server.stop();
			}
		});
	}
});
