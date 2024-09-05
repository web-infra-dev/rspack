"use strict";

const path = require("path");
const webpack = require("@rspack/core");
const { RspackDevServer: Server } = require("@rspack/dev-server");
const config = require("../fixtures/client-config/webpack.config");
const runBrowser = require("../helpers/run-browser");
const port = require("../helpers/ports-map").entry;

const HOT_ENABLED_MESSAGE =
	"[webpack-dev-server] Server started: Hot Module Replacement enabled, Live Reloading enabled, Progress disabled, Overlay enabled.";

const waitForConsoleLogFinished = async consoleLogs => {
	await new Promise(resolve => {
		const interval = setInterval(() => {
			if (consoleLogs.includes(HOT_ENABLED_MESSAGE)) {
				clearInterval(interval);

				resolve();
			}
		}, 100);
	});
};

describe("entry", () => {
	const entryFirst = path.resolve(
		__dirname,
		"../fixtures/client-config/foo.js"
	);
	const entrySecond = path.resolve(
		__dirname,
		"../fixtures/client-config/bar.js"
	);

	it("should work with single entry", async () => {
		const compiler = webpack({ ...config, entry: entryFirst });
		const devServerOptions = {
			port
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

	it("should work with single array entry", async () => {
		const compiler = webpack({ ...config, entry: [entryFirst, entrySecond] });
		const devServerOptions = {
			port
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

	it("should work with object entry", async () => {
		const compiler = webpack({
			...config,
			entry: {
				main: { import: entryFirst }
			}
		});
		const devServerOptions = {
			port
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

	it("should work with dynamic entry", async () => {
		const compiler = webpack({ ...config, entry: () => entryFirst });
		const devServerOptions = {
			port
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

	it("should work with dynamic async entry", async () => {
		const compiler = webpack({
			...config,
			entry: () => new Promise(resolve => resolve([entryFirst]))
		});
		const devServerOptions = {
			port
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

	it("should work with multiple entries", async () => {
		const compiler = webpack({
			...config,
			entry: {
				foo: entryFirst,
				bar: entrySecond
			},
			optimization: {
				runtimeChunk: {
					name: "runtime"
				}
			}
		});
		const devServerOptions = {
			port
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

			await page.goto(`http://127.0.0.1:${port}/test.html`, {
				waitUntil: "networkidle0"
			});
			await page.addScriptTag({ url: `http://127.0.0.1:${port}/runtime.js` });
			await page.addScriptTag({ url: `http://127.0.0.1:${port}/foo.js` });
			await waitForConsoleLogFinished(consoleMessages);

			expect(consoleMessages).toMatchSnapshot("console messages");
			expect(pageErrors).toMatchSnapshot("page errors");
		} catch (error) {
			throw error;
		} finally {
			await browser.close();
			await server.stop();
		}
	});

	it("should work with multiple entries #2", async () => {
		const compiler = webpack({
			...config,
			entry: {
				foo: entryFirst,
				bar: entrySecond
			},
			optimization: {
				runtimeChunk: {
					name: "runtime"
				}
			}
		});
		const devServerOptions = {
			port
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

			await page.goto(`http://127.0.0.1:${port}/test.html`, {
				waitUntil: "networkidle0"
			});
			await page.addScriptTag({ url: `http://127.0.0.1:${port}/runtime.js` });
			await page.addScriptTag({ url: `http://127.0.0.1:${port}/bar.js` });
			await waitForConsoleLogFinished(consoleMessages);

			expect(consoleMessages).toMatchSnapshot("console messages");
			expect(pageErrors).toMatchSnapshot("page errors");
		} catch (error) {
			throw error;
		} finally {
			await browser.close();
			await server.stop();
		}
	});

	it('should work with multiple entries and "dependOn"', async () => {
		const compiler = webpack({
			...config,
			entry: {
				foo: {
					import: entryFirst,
					dependOn: "bar"
				},
				bar: entrySecond
			}
		});
		const devServerOptions = {
			port
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

			await page.goto(`http://127.0.0.1:${port}/test.html`, {
				waitUntil: "networkidle0"
			});
			await page.addScriptTag({ url: `http://127.0.0.1:${port}/bar.js` });
			await page.addScriptTag({ url: `http://127.0.0.1:${port}/foo.js` });
			await waitForConsoleLogFinished(consoleMessages);

			expect(consoleMessages).toMatchSnapshot("console messages");
			expect(pageErrors).toMatchSnapshot("page errors");
		} catch (error) {
			throw error;
		} finally {
			await browser.close();
			await server.stop();
		}
	});

	it("should work with empty", async () => {
		const compiler = webpack({
			...config,
			entry: {}
		});

		new webpack.EntryPlugin(compiler.context, entryFirst, {
			name: "main"
		}).apply(compiler);

		const devServerOptions = {
			port
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
