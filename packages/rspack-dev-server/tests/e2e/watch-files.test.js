"use strict";

const path = require("path");
const chokidar = require("chokidar");
const fs = require("graceful-fs");
const webpack = require("@rspack/core");
const { RspackDevServer: Server } = require("@rspack/dev-server");
const config = require("../fixtures/watch-files-config/webpack.config");
const runBrowser = require("../helpers/run-browser");
const port = require("../helpers/ports-map")["watch-files-option"];

const watchDir = path.resolve(
	__dirname,
	"../fixtures/watch-files-config/public"
);

describe("watchFiles option", () => {
	describe("should work with string and path to file", () => {
		const file = path.join(watchDir, "assets/example.txt");
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
					watchFiles: file,
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
			fs.truncateSync(file);
		});

		it("should reload when file content is changed", async () => {
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

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");

			// change file content
			fs.writeFileSync(file, "Kurosaki Ichigo", "utf8");

			await new Promise(resolve => {
				server.staticWatchers[0].on("change", async changedPath => {
					// page reload
					await page.waitForNavigation({ waitUntil: "networkidle0" });

					expect(changedPath).toBe(file);

					resolve();
				});
			});
		});
	});

	describe("should work with string and path to directory", () => {
		const file = path.join(watchDir, "assets/example.txt");
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
					watchFiles: watchDir,
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
			fs.truncateSync(file);
		});

		it("should reload when file content is changed", async () => {
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

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");

			// change file content
			fs.writeFileSync(file, "Kurosaki Ichigo", "utf8");

			await new Promise(resolve => {
				server.staticWatchers[0].on("change", async changedPath => {
					// page reload
					await page.waitForNavigation({ waitUntil: "networkidle0" });

					expect(changedPath).toBe(file);

					resolve();
				});
			});
		});
	});

	describe("should work with string and glob", () => {
		const file = path.join(watchDir, "assets/example.txt");
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
					watchFiles: `${watchDir}/**/*`,
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
			fs.truncateSync(file);
		});

		it("should reload when file content is changed", async () => {
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

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");

			// change file content
			fs.writeFileSync(file, "Kurosaki Ichigo", "utf8");

			await new Promise(resolve => {
				server.staticWatchers[0].on("change", async changedPath => {
					// page reload
					await page.waitForNavigation({ waitUntil: "networkidle0" });

					expect(changedPath).toBe(file);

					resolve();
				});
			});
		});
	});

	describe("should not crash if file doesn't exist", () => {
		const nonExistFile = path.join(watchDir, "assets/non-exist.txt");
		let compiler;
		let server;
		let page;
		let browser;
		let pageErrors;
		let consoleMessages;

		beforeEach(async () => {
			try {
				fs.unlinkSync(nonExistFile);
			} catch (error) {
				// ignore
			}

			compiler = webpack(config);

			server = new Server(
				{
					watchFiles: nonExistFile,
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

		it("should reload when file content is changed", async () => {
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

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");

			await new Promise(resolve => {
				server.staticWatchers[0].on("change", async changedPath => {
					// page reload
					await page.waitForNavigation({ waitUntil: "networkidle0" });

					expect(changedPath).toBe(nonExistFile);
					resolve();
				});

				// create file content
				setTimeout(() => {
					fs.writeFileSync(nonExistFile, "Kurosaki Ichigo", "utf8");
					// change file content
					setTimeout(() => {
						fs.writeFileSync(nonExistFile, "Kurosaki Ichigo", "utf8");
					}, 1000);
				}, 1000);
			});
		});
	});

	describe("should work with object with single path", () => {
		const file = path.join(watchDir, "assets/example.txt");
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
					watchFiles: { paths: file },
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
			fs.truncateSync(file);
		});

		it("should reload when file content is changed", async () => {
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

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");

			// change file content
			fs.writeFileSync(file, "Kurosaki Ichigo", "utf8");

			await new Promise(resolve => {
				server.staticWatchers[0].on("change", async changedPath => {
					// page reload
					await page.waitForNavigation({ waitUntil: "networkidle0" });

					expect(changedPath).toBe(file);

					resolve();
				});
			});
		});
	});

	describe("should work with object with multiple paths", () => {
		const file = path.join(watchDir, "assets/example.txt");
		const other = path.join(watchDir, "assets/other.txt");
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
					watchFiles: { paths: [file, other] },
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
			fs.truncateSync(file);
			fs.truncateSync(other);
		});

		it("should reload when file content is changed", async () => {
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

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");

			// change file content
			fs.writeFileSync(file, "foo", "utf8");
			fs.writeFileSync(other, "bar", "utf8");

			await new Promise(resolve => {
				const expected = [file, other];
				let changed = 0;

				server.staticWatchers[0].on("change", async changedPath => {
					// page reload
					await page.waitForNavigation({ waitUntil: "networkidle0" });

					expect(expected.includes(changedPath)).toBeTruthy();

					changed += 1;

					if (changed === 2) {
						resolve();
					}
				});
			});
		});
	});

	describe("should work with array config", () => {
		const file = path.join(watchDir, "assets/example.txt");
		const other = path.join(watchDir, "assets/other.txt");
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
					watchFiles: [{ paths: [file] }, other],
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
			fs.truncateSync(file);
			fs.truncateSync(other);
		});

		it("should reload when file content is changed", async () => {
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

			expect(consoleMessages.map(message => message.text())).toMatchSnapshot(
				"console messages"
			);

			expect(pageErrors).toMatchSnapshot("page errors");

			// change file content
			fs.writeFileSync(file, "foo", "utf8");
			fs.writeFileSync(other, "bar", "utf8");

			await new Promise(resolve => {
				let changed = 0;

				server.staticWatchers[0].on("change", async changedPath => {
					// page reload
					await page.waitForNavigation({ waitUntil: "networkidle0" });

					expect(changedPath).toBe(file);

					changed += 1;

					if (changed === 2) {
						resolve();
					}
				});
				server.staticWatchers[1].on("change", async changedPath => {
					// page reload
					await page.waitForNavigation({ waitUntil: "networkidle0" });

					expect(changedPath).toBe(other);

					changed += 1;

					if (changed === 2) {
						resolve();
					}
				});
			});
		});
	});

	describe("should work with options", () => {
		const file = path.join(watchDir, "assets/example.txt");

		const chokidarMock = jest.spyOn(chokidar, "watch");

		const optionCases = [
			{
				poll: true
			},
			{
				poll: 200
			},
			{
				usePolling: true
			},
			{
				usePolling: true,
				poll: 200
			},
			{
				usePolling: false
			},
			{
				usePolling: false,
				poll: 200
			},
			{
				usePolling: false,
				poll: true
			},
			{
				interval: 400,
				poll: 200
			},
			{
				usePolling: true,
				interval: 200,
				poll: 400
			},
			{
				usePolling: false,
				interval: 200,
				poll: 400
			}
		];

		optionCases.forEach(optionCase => {
			describe(JSON.stringify(optionCase), () => {
				let compiler;
				let server;
				let page;
				let browser;
				let pageErrors;
				let consoleMessages;

				beforeEach(async () => {
					chokidarMock.mockClear();

					compiler = webpack(config);

					server = new Server(
						{
							watchFiles: {
								paths: file,
								options: optionCase
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
					await server.stop();
					await browser.close();
					fs.truncateSync(file);
				});

				it("should reload when file content is changed", async () => {
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

					// should pass correct options to chokidar config
					expect(chokidarMock.mock.calls[0][1]).toMatchSnapshot();

					expect(response.status()).toMatchSnapshot("response status");

					expect(
						consoleMessages.map(message => message.text())
					).toMatchSnapshot("console messages");

					expect(pageErrors).toMatchSnapshot("page errors");

					// change file content
					fs.writeFileSync(file, "Kurosaki Ichigo", "utf8");

					await new Promise(resolve => {
						server.staticWatchers[0].on("change", async changedPath => {
							// page reload
							await page.waitForNavigation({ waitUntil: "networkidle0" });

							expect(changedPath).toBe(file);

							resolve();
						});
					});
				});
			});
		});
	});
});
