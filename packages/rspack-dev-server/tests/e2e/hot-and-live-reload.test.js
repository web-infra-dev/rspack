/**
 * @jest-environment node
 */

"use strict";

const path = require("path");
const WebSocket = require("ws");
const SockJS = require("sockjs-client");
const webpack = require("@rspack/core");
const fs = require("graceful-fs");
const { RspackDevServer: Server } = require("@rspack/dev-server");
const HTMLGeneratorPlugin = require("../helpers/html-generator-plugin");
const reloadConfig = require("../fixtures/reload-config/webpack.config");
const runBrowser = require("../helpers/run-browser");
const port = require("../helpers/ports-map")["hot-and-live-reload"];
const config = require("../fixtures/client-config/webpack.config");
const multiCompilerConfig = require("../fixtures/multi-compiler-one-configuration/webpack.config");

const cssFilePath = path.resolve(
	__dirname,
	"../fixtures/reload-config/main.css"
);

const INVALID_MESSAGE = "[webpack-dev-server] App updated. Recompiling...";

describe("hot and live reload", () => {
	// "sockjs" client cannot add additional headers
	const modes = [
		{
			title: "should work and refresh content using hot module replacement"
		},
		{
			title: "should work and do nothing when web socket server disabled",
			options: {
				webSocketServer: false
			}
		},
		// Default web socket serve ("ws")
		{
			title:
				"should work and refresh content using hot module replacement when hot enabled",
			options: {
				hot: true
			}
		},
		{
			title:
				"should work and refresh content using hot module replacement when live reload enabled",
			options: {
				liveReload: true
			}
		},
		{
			title: "should not refresh content when hot and no live reload disabled",
			options: {
				hot: false,
				liveReload: false
			}
		},
		{
			title:
				"should work and refresh content using hot module replacement when live reload disabled and hot enabled",
			options: {
				liveReload: false,
				hot: true
			}
		},
		{
			title: "should work and refresh content using live reload",
			options: {
				liveReload: true,
				hot: false
			}
		},
		{
			title:
				"should work and refresh content using hot module replacement when live reload enabled and hot disabled",
			options: {
				liveReload: true,
				hot: true
			}
		},
		// "ws" web socket serve
		{
			title:
				"should work and refresh content using hot module replacement when hot enabled",
			options: {
				webSocketServer: "ws",
				hot: true
			}
		},
		{
			title:
				"should work and refresh content using hot module replacement when live reload enabled",
			options: {
				webSocketServer: "ws",
				liveReload: true
			}
		},
		{
			title: "should not refresh content when hot and no live reload disabled",
			options: {
				webSocketServer: "ws",
				hot: false,
				liveReload: false
			}
		},
		{
			title:
				"should work and refresh content using hot module replacement when live reload disabled and hot enabled",
			options: {
				webSocketServer: "ws",
				liveReload: false,
				hot: true
			}
		},
		{
			title:
				"should work and refresh content using live reload when live reload enabled and hot disabled",
			options: {
				webSocketServer: "ws",
				liveReload: true,
				hot: false
			}
		},
		{
			title:
				"should work and refresh content using hot module replacement when live reload and hot enabled",
			options: {
				webSocketServer: "ws",
				liveReload: true,
				hot: true
			}
		},
		// "sockjs" web socket serve
		{
			title:
				"should work and refresh content using hot module replacement when hot enabled",
			options: {
				allowedHosts: "all",

				webSocketServer: "sockjs",
				hot: true
			}
		},
		{
			title:
				"should work and refresh content using hot module replacement when live reload enabled",
			options: {
				allowedHosts: "all",

				webSocketServer: "sockjs",
				liveReload: true
			}
		},
		{
			title: "should not refresh content when hot and no live reload disabled",
			options: {
				allowedHosts: "all",

				webSocketServer: "sockjs",
				hot: false,
				liveReload: false
			}
		},
		{
			title:
				"should work and refresh content using hot module replacement when live reload disabled and hot enabled",
			options: {
				allowedHosts: "all",

				webSocketServer: "sockjs",
				liveReload: false,
				hot: true
			}
		},
		{
			title:
				"should work and refresh content using live reload when live reload disabled and hot enabled",
			options: {
				allowedHosts: "all",

				webSocketServer: "sockjs",
				liveReload: true,
				hot: false
			}
		},
		{
			title:
				"should work and refresh content using hot module replacement when live reload and hot enabled",
			options: {
				allowedHosts: "all",

				webSocketServer: "sockjs",
				liveReload: true,
				hot: true
			}
		},
		{
			title:
				'should work and allow to disable hot module replacement using the "webpack-dev-server-hot=false"',
			query: "?webpack-dev-server-hot=false",
			options: {
				liveReload: true,
				hot: true
			}
		},
		{
			title:
				'should work and allow to disable live reload using the "webpack-dev-server-live-reload=false"',
			query: "?webpack-dev-server-live-reload=false",
			options: {
				liveReload: true,
				hot: false
			}
		},
		{
			title:
				'should work and allow to disable hot module replacement and live reload using the "webpack-dev-server-hot=false&webpack-dev-server-live-reload=false"',
			query:
				"?webpack-dev-server-hot=false&webpack-dev-server-live-reload=false",
			options: {
				liveReload: true,
				hot: true
			}
		},
		{
			title: "should work with manual client setup",
			webpackOptions: {
				entry: [
					require.resolve("@rspack/dev-server/client/index.js"),
					require.resolve("../fixtures/reload-config/foo.js")
				]
			},
			options: {
				client: false,
				liveReload: true,
				hot: true
			}
		},
		// TODO we still output logs from webpack, need to improve this
		{
			title:
				"should work with manual client setup and allow to enable hot module replacement",
			webpackOptions: {
				entry: [
					"@rspack/core/hot/dev-server",
					`${require.resolve("@rspack/dev-server/client/index.js")}?hot=true`,
					require.resolve("../fixtures/reload-config/foo.js")
				],
				plugins: [
					new webpack.HotModuleReplacementPlugin(),
					new HTMLGeneratorPlugin()
				]
			},
			options: {
				client: false,
				liveReload: false,
				hot: false
			}
		},
		{
			title:
				"should work with manual client setup and allow to disable hot module replacement",
			webpackOptions: {
				entry: [
					`${require.resolve("@rspack/dev-server/client/index.js")}?hot=false`,
					require.resolve("../fixtures/reload-config/foo.js")
				]
			},
			options: {
				client: false,
				liveReload: true,
				hot: true
			}
		},
		{
			title:
				"should work with manual client setup and allow to enable live reload",
			webpackOptions: {
				entry: [
					`${require.resolve("@rspack/dev-server/client/index.js")}?live-reload=true`,
					require.resolve("../fixtures/reload-config/foo.js")
				]
			},
			options: {
				client: false,
				liveReload: false,
				hot: false
			}
		},
		{
			title:
				"should work with manual client setup and allow to disable live reload",
			webpackOptions: {
				entry: [
					`${require.resolve("@rspack/dev-server/client/index.js")}?live-reload=false`,
					require.resolve("../fixtures/reload-config/foo.js")
				]
			},
			options: {
				client: false,
				liveReload: true,
				hot: false
			}
		}
	];

	let browser;
	let server;

	beforeEach(() => {
		fs.writeFileSync(cssFilePath, "body { background-color: rgb(0, 0, 255); }");
	});

	afterEach(async () => {
		if (browser) {
			await browser.close();
		}

		if (server) {
			await server.stop();
		}

		fs.unlinkSync(cssFilePath);
	});

	modes.forEach(mode => {
		const webSocketServerTitle =
			mode.options && mode.options.webSocketServer
				? mode.options.webSocketServer
				: "default";

		it(`${mode.title} (${webSocketServerTitle})`, async () => {
			const webpackOptions = { ...reloadConfig, ...mode.webpackOptions };
			const compiler = webpack(webpackOptions);
			const testDevServerOptions = mode.options || {};
			const devServerOptions = { port, ...testDevServerOptions };

			server = new Server(devServerOptions, compiler);

			await server.start();

			const webSocketServerLaunched =
				testDevServerOptions.webSocketServer !== false;

			await new Promise(resolve => {
				const webSocketTransport =
					typeof testDevServerOptions.webSocketServer !== "undefined" &&
					testDevServerOptions.webSocketServer !== false
						? testDevServerOptions.webSocketServer
						: "ws";

				if (webSocketTransport === "ws") {
					const ws = new WebSocket(
						`ws://127.0.0.1:${devServerOptions.port}/ws`,
						{
							headers: {
								host: `127.0.0.1:${devServerOptions.port}`,
								origin: `http://127.0.0.1:${devServerOptions.port}`
							}
						}
					);

					let opened = false;
					let received = false;
					let errored = false;

					ws.on("error", error => {
						if (!webSocketServerLaunched && /404/.test(error)) {
							errored = true;
						} else {
							errored = true;
						}

						ws.close();
					});

					ws.on("open", () => {
						opened = true;
					});

					ws.on("message", data => {
						const message = JSON.parse(data.toString());

						if (message.type === "ok") {
							received = true;

							ws.close();
						}
					});

					ws.on("close", () => {
						if (opened && received && !errored) {
							resolve();
						} else if (!webSocketServerLaunched && errored) {
							resolve();
						}
					});
				} else {
					const sockjs = new SockJS(
						`http://127.0.0.1:${devServerOptions.port}/ws`
					);

					let opened = false;
					let received = false;
					let errored = false;

					sockjs.onerror = () => {
						errored = true;
					};

					sockjs.onopen = () => {
						opened = true;
					};

					sockjs.onmessage = ({ data }) => {
						const message = JSON.parse(data.toString());

						if (message.type === "ok") {
							received = true;

							sockjs.close();
						}
					};

					sockjs.onclose = event => {
						if (opened && received && !errored) {
							resolve();
						} else if (event && event.reason === "Cannot connect to server") {
							resolve();
						}
					};
				}
			});

			const launched = await runBrowser();

			({ browser } = launched);

			const page = launched.page;

			const consoleMessages = [];
			const pageErrors = [];

			let doneHotUpdate = false;
			let hasDisconnectedMessage = false;

			page
				.on("console", message => {
					if (!hasDisconnectedMessage) {
						const text = message.text();

						hasDisconnectedMessage = /Disconnected!/.test(text);
						consoleMessages.push(text);
					}
				})
				.on("pageerror", error => {
					pageErrors.push(error);
				})
				.on("request", requestObj => {
					if (/\.hot-update\.json$/.test(requestObj.url())) {
						doneHotUpdate = true;
					}
				});

			await page.goto(`http://localhost:${port}/${mode.query || ""}`, {
				waitUntil: "networkidle0"
			});

			const backgroundColorBefore = await page.evaluate(() => {
				const body = document.body;

				return getComputedStyle(body)["background-color"];
			});

			expect(backgroundColorBefore).toEqual("rgb(0, 0, 255)");

			fs.writeFileSync(
				cssFilePath,
				"body { background-color: rgb(255, 0, 0); }"
			);

			let waitHot =
				typeof testDevServerOptions.hot !== "undefined"
					? testDevServerOptions.hot
					: true;
			let waitLiveReload =
				typeof testDevServerOptions.liveReload !== "undefined"
					? testDevServerOptions.liveReload
					: true;

			if (webSocketServerLaunched === false) {
				waitHot = false;
				waitLiveReload = false;
			}

			if (Array.isArray(webpackOptions.entry)) {
				if (webpackOptions.entry.some(item => item.includes("hot=true"))) {
					waitHot = true;
				} else if (
					webpackOptions.entry.some(item => item.includes("hot=false"))
				) {
					waitHot = false;
				}
			}

			if (Array.isArray(webpackOptions.entry)) {
				if (
					webpackOptions.entry.some(item => item.includes("live-reload=true"))
				) {
					waitLiveReload = true;
				} else if (
					webpackOptions.entry.some(item => item.includes("live-reload=false"))
				) {
					waitLiveReload = false;
				}
			}

			const query = mode.query || "";

			if (query.includes("webpack-dev-server-hot=false")) {
				waitHot = false;
			}

			if (query.includes("webpack-dev-server-live-reload=false")) {
				waitLiveReload = false;
			}

			if (waitHot) {
				await page.waitForFunction(
					() =>
						getComputedStyle(document.body)["background-color"] ===
						"rgb(255, 0, 0)"
				);

				expect(doneHotUpdate).toBe(true);
			} else if (waitLiveReload) {
				await page.waitForNavigation({
					waitUntil: "networkidle0"
				});
			} else if (webSocketServerLaunched) {
				await new Promise(resolve => {
					const interval = setInterval(() => {
						if (consoleMessages.includes(INVALID_MESSAGE)) {
							clearInterval(interval);

							resolve();
						}
					}, 100);
				});
			}

			const backgroundColorAfter = await page.evaluate(() => {
				const body = document.body;

				return getComputedStyle(body)["background-color"];
			});

			if (!waitHot && !waitLiveReload) {
				expect(backgroundColorAfter).toEqual("rgb(0, 0, 255)");
			} else {
				expect(backgroundColorAfter).toEqual("rgb(255, 0, 0)");
			}

			expect(consoleMessages).toMatchSnapshot("console messages");
			expect(pageErrors).toMatchSnapshot("page errors");
		});
	});
});

// the following cases check to make sure that the HMR
// plugin is actually added

// describe("simple hot config HMR plugin", () => {
//   let compiler;
//   let server;
//   let page;
//   let browser;
//   let pageErrors;
//   let consoleMessages;

//   beforeEach(async () => {
//     compiler = webpack(config);

//     ({ page, browser } = await runBrowser());

//     pageErrors = [];
//     consoleMessages = [];
//   });

//   afterEach(async () => {
//     await browser.close();
//     await server.stop();
//   });

//   it("should register the HMR plugin before compilation is complete", async () => {
//     let pluginFound = false;

//     compiler.hooks.compilation.intercept({
//       register: (tapInfo) => {
//         if (tapInfo.name === "HotModuleReplacementPlugin") {
//           pluginFound = true;
//         }

//         return tapInfo;
//       },
//     });

//     server = new Server({ port }, compiler);

//     await server.start();

//     expect(pluginFound).toBe(true);

//     page
//       .on("console", (message) => {
//         consoleMessages.push(message);
//       })
//       .on("pageerror", (error) => {
//         pageErrors.push(error);
//       });

//     const response = await page.goto(`http://127.0.0.1:${port}/`, {
//       waitUntil: "networkidle0",
//     });

//     expect(response.status()).toMatchSnapshot("response status");

//     expect(consoleMessages.map((message) => message.text())).toMatchSnapshot(
//       "console messages",
//     );

//     expect(pageErrors).toMatchSnapshot("page errors");
//   });
// });

// describe("simple hot config HMR plugin with already added HMR plugin", () => {
//   let compiler;
//   let server;
//   let page;
//   let browser;
//   let pageErrors;
//   let consoleMessages;

//   beforeEach(async () => {
//     compiler = webpack({
//       ...config,
//       plugins: [...config.plugins, new webpack.HotModuleReplacementPlugin()],
//     });

//     ({ page, browser } = await runBrowser());

//     pageErrors = [];
//     consoleMessages = [];
//   });

//   afterEach(async () => {
//     await browser.close();
//     await server.stop();
//   });

//   it("should register the HMR plugin before compilation is complete", async () => {
//     let pluginFound = false;

//     compiler.hooks.compilation.intercept({
//       register: (tapInfo) => {
//         if (tapInfo.name === "HotModuleReplacementPlugin") {
//           pluginFound = true;
//         }

//         return tapInfo;
//       },
//     });

//     server = new Server({ port }, compiler);

//     await server.start();

//     expect(compiler.options.plugins).toHaveLength(2);
//     expect(pluginFound).toBe(true);

//     page
//       .on("console", (message) => {
//         consoleMessages.push(message);
//       })
//       .on("pageerror", (error) => {
//         pageErrors.push(error);
//       });

//     const response = await page.goto(`http://127.0.0.1:${port}/`, {
//       waitUntil: "networkidle0",
//     });

//     expect(response.status()).toMatchSnapshot("response status");

//     expect(consoleMessages.map((message) => message.text())).toMatchSnapshot(
//       "console messages",
//     );

//     expect(pageErrors).toMatchSnapshot("page errors");
//   });
// });

// describe("simple config with already added HMR plugin", () => {
//   let loggerWarnSpy;
//   let getInfrastructureLoggerSpy;
//   let compiler;
//   let server;

//   beforeEach(() => {
//     compiler = webpack({
//       ...config,
//       devServer: { hot: false },
//       plugins: [...config.plugins, new webpack.HotModuleReplacementPlugin()],
//     });

//     loggerWarnSpy = jest.fn();

//     getInfrastructureLoggerSpy = jest
//       .spyOn(compiler, "getInfrastructureLogger")
//       .mockImplementation(() => {
//         return {
//           warn: loggerWarnSpy,
//           info: () => { },
//           log: () => { },
//         };
//       });
//   });

//   afterEach(() => {
//     getInfrastructureLoggerSpy.mockRestore();
//     loggerWarnSpy.mockRestore();
//   });

//   it("should show warning with hot normalized as true", async () => {
//     server = new Server({ port }, compiler);

//     await server.start();

//     expect(loggerWarnSpy).toHaveBeenCalledWith(
//       `"hot: true" automatically applies HMR plugin, you don't have to add it manually to your webpack configuration.`,
//     );

//     await server.stop();
//   });

//   it(`should show warning with "hot: true"`, async () => {
//     server = new Server({ port, hot: true }, compiler);

//     await server.start();

//     expect(loggerWarnSpy).toHaveBeenCalledWith(
//       `"hot: true" automatically applies HMR plugin, you don't have to add it manually to your webpack configuration.`,
//     );

//     await server.stop();
//   });

//   it(`should not show warning with "hot: false"`, async () => {
//     server = new Server({ port, hot: false }, compiler);

//     await server.start();

//     expect(loggerWarnSpy).not.toHaveBeenCalledWith(
//       `"hot: true" automatically applies HMR plugin, you don't have to add it manually to your webpack configuration.`,
//     );

//     await server.stop();
//   });
// });

// describe("multi compiler hot config HMR plugin", () => {
//   let compiler;
//   let server;
//   let page;
//   let browser;
//   let pageErrors;
//   let consoleMessages;

//   beforeEach(async () => {
//     compiler = webpack(multiCompilerConfig);

//     ({ page, browser } = await runBrowser());

//     pageErrors = [];
//     consoleMessages = [];
//   });

//   afterEach(async () => {
//     await browser.close();
//     await server.stop();
//   });

//   it("should register the HMR plugin before compilation is complete", async () => {
//     let pluginFound = false;

//     compiler.compilers[0].hooks.compilation.intercept({
//       register: (tapInfo) => {
//         if (tapInfo.name === "HotModuleReplacementPlugin") {
//           pluginFound = true;
//         }

//         return tapInfo;
//       },
//     });

//     server = new Server({ port }, compiler);

//     await server.start();

//     expect(pluginFound).toBe(true);

//     page
//       .on("console", (message) => {
//         consoleMessages.push(message);
//       })
//       .on("pageerror", (error) => {
//         pageErrors.push(error);
//       });

//     const response = await page.goto(`http://127.0.0.1:${port}/`, {
//       waitUntil: "networkidle0",
//     });

//     expect(response.status()).toMatchSnapshot("response status");

//     expect(consoleMessages.map((message) => message.text())).toMatchSnapshot(
//       "console messages",
//     );

//     expect(pageErrors).toMatchSnapshot("page errors");
//   });
// });

// describe("hot disabled HMR plugin", () => {
//   let compiler;
//   let server;
//   let page;
//   let browser;
//   let pageErrors;
//   let consoleMessages;

//   beforeEach(async () => {
//     compiler = webpack(config);

//     ({ page, browser } = await runBrowser());

//     pageErrors = [];
//     consoleMessages = [];
//   });

//   afterEach(async () => {
//     await browser.close();
//     await server.stop();
//   });

//   it("should NOT register the HMR plugin before compilation is complete", async () => {
//     let pluginFound = false;

//     compiler.hooks.compilation.intercept({
//       register: (tapInfo) => {
//         if (tapInfo.name === "HotModuleReplacementPlugin") {
//           pluginFound = true;
//         }

//         return tapInfo;
//       },
//     });

//     server = new Server({ port, hot: false }, compiler);

//     await server.start();

//     expect(pluginFound).toBe(false);

//     page
//       .on("console", (message) => {
//         consoleMessages.push(message);
//       })
//       .on("pageerror", (error) => {
//         pageErrors.push(error);
//       });

//     const response = await page.goto(`http://127.0.0.1:${port}/`, {
//       waitUntil: "networkidle0",
//     });

//     expect(response.status()).toMatchSnapshot("response status");

//     expect(consoleMessages.map((message) => message.text())).toMatchSnapshot(
//       "console messages",
//     );

//     expect(pageErrors).toMatchSnapshot("page errors");
//   });
// });
