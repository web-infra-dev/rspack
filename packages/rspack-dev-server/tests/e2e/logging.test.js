"use strict";

const path = require("path");
const fs = require("graceful-fs");
const webpack = require("@rspack/core");
const { RspackDevServer: Server } = require("@rspack/dev-server");
const HTMLGeneratorPlugin = require("../helpers/html-generator-plugin");
const config = require("../fixtures/client-config/webpack.config");
const runBrowser = require("../helpers/run-browser");
const port = require("../helpers/ports-map").logging;
require("../helpers/normalize");

describe("logging", () => {
	const webSocketServers = [
		{ webSocketServer: "ws" },
		{ webSocketServer: "sockjs" }
	];

	const cases = [
		{
			title: "should work and log message about live reloading is enabled",
			devServerOptions: {
				hot: false
			}
		},
		{
			title:
				"should work and log messages about hot and live reloading is enabled",
			devServerOptions: {
				hot: true
			}
		},
		{
			title: "should work and log messages about hot is enabled",
			devServerOptions: {
				liveReload: false
			}
		},
		{
			title:
				"should work and log messages about hot and live reloading is enabled",
			devServerOptions: {
				liveReload: true
			}
		},
		{
			title:
				"should work and do not log messages about hot and live reloading is enabled",
			devServerOptions: {
				liveReload: false,
				hot: false
			}
		},
		{
			title:
				"should work and log messages about hot and live reloading is enabled",
			devServerOptions: {
				liveReload: true,
				hot: true
			}
		},
		{
			title: "should work and log warnings by default",
			webpackOptions: {
				plugins: [
					{
						apply(compiler) {
							compiler.hooks.thisCompilation.tap(
								"warnings-webpack-plugin",
								compilation => {
									compilation.warnings.push(
										new Error("Warning from compilation")
									);
								}
							);
						}
					},
					new HTMLGeneratorPlugin()
				]
			}
		},
		{
			title: "should work and log errors by default",
			webpackOptions: {
				plugins: [
					{
						apply(compiler) {
							compiler.hooks.thisCompilation.tap(
								"warnings-webpack-plugin",
								compilation => {
									compilation.errors.push(new Error("Error from compilation"));
								}
							);
						}
					},
					new HTMLGeneratorPlugin()
				]
			}
		},
		{
			title: 'should work when the "client.logging" is "info"',
			devServerOptions: {
				client: {
					logging: "info"
				}
			}
		},
		{
			title: 'should work when the "client.logging" is "log"',
			devServerOptions: {
				client: {
					logging: "log"
				}
			}
		},
		{
			title: 'should work when the "client.logging" is "verbose"',
			devServerOptions: {
				client: {
					logging: "verbose"
				}
			}
		},
		{
			title: 'should work when the "client.logging" is "none"',
			devServerOptions: {
				client: {
					logging: "none"
				}
			}
		},
		{
			title: "should work and log only error",
			webpackOptions: {
				plugins: [
					{
						apply(compiler) {
							compiler.hooks.thisCompilation.tap(
								"warnings-webpack-plugin",
								compilation => {
									compilation.warnings.push(
										new Error("Warning from compilation")
									);
									compilation.errors.push(new Error("Error from compilation"));
								}
							);
						}
					},
					new HTMLGeneratorPlugin()
				]
			},
			devServerOptions: {
				client: {
					logging: "error"
				}
			}
		},
		{
			title: "should work and log warning and errors",
			webpackOptions: {
				plugins: [
					{
						apply(compiler) {
							compiler.hooks.thisCompilation.tap(
								"warnings-webpack-plugin",
								compilation => {
									compilation.warnings.push(
										new Error("Warning from compilation")
									);
									compilation.errors.push(new Error("Error from compilation"));
								}
							);
						}
					},
					new HTMLGeneratorPlugin()
				]
			},
			devServerOptions: {
				client: {
					logging: "warn"
				}
			}
		},
		{
			title: "should work and log static changes",
			devServerOptions: {
				static: path.resolve(__dirname, "../fixtures/client-config/static")
			}
		}
	];

	webSocketServers.forEach(webSocketServer => {
		cases.forEach(testCase => {
			it(`${testCase.title} (${
				webSocketServer.webSocketServer || "default"
			})`, async () => {
				const compiler = webpack({ ...config, ...testCase.webpackOptions });
				const devServerOptions = {
					port,
					...testCase.devServerOptions
				};
				const server = new Server(devServerOptions, compiler);

				await server.start();

				const { page, browser } = await runBrowser();

				try {
					const consoleMessages = [];

					page.on("console", message => {
						consoleMessages.push(message);
					});

					await page.goto(`http://localhost:${port}/`, {
						waitUntil: "networkidle0"
					});

					if (testCase.devServerOptions && testCase.devServerOptions.static) {
						fs.writeFileSync(
							path.join(testCase.devServerOptions.static, "./foo.txt"),
							"Text"
						);

						await page.waitForNavigation({
							waitUntil: "networkidle0"
						});
					}

					expect(
						consoleMessages.map(message => message.text().replace(/\\/g, "/"))
					).toMatchSnapshot();
				} catch (error) {
					throw error;
				} finally {
					await browser.close();
					await server.stop();
				}
			});
		});
	});
});
