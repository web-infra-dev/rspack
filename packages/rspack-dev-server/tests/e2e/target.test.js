"use strict";

const webpack = require("@rspack/core");
const { RspackDevServer: Server } = require("@rspack/dev-server");
const config = require("../fixtures/client-config/webpack.config");
const runBrowser = require("../helpers/run-browser");
const port = require("../helpers/ports-map").target;

describe("target", () => {
	const targets = [
		false,
		"browserslist:defaults",
		"web",
		"webworker",
		"node",
		"async-node",
		"electron-main",
		"electron-preload",
		"electron-renderer",
		"nwjs",
		"node-webkit",
		"es5",
		["web", "es5"]
	];

	for (const target of targets) {
		it(`should work using "${target}" target`, async () => {
			const compiler = webpack({
				...config,
				target,
				...(target === false || target === "es5"
					? {
							output: { chunkFormat: "array-push", path: "/" }
						}
					: {})
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

				// TODO: check why require is defined in theses target
				// if (
				//   target === "node" ||
				//   target === "async-node" ||
				//   target === "electron-main" ||
				//   target === "electron-preload" ||
				//   target === "electron-renderer" ||
				//   target === "nwjs" ||
				//   target === "node-webkit"
				// ) {
				//   console.log(pageErrors);
				//   const hasRequireOrGlobalError =
				//     pageErrors.filter((pageError) =>
				//       /require is not defined|global is not defined/.test(pageError),
				//     ).length === 1;

				//   expect(hasRequireOrGlobalError).toBe(true);
				// } else {
				//   expect(pageErrors).toMatchSnapshot("page errors");
				// }
			} catch (error) {
				throw error;
			} finally {
				await browser.close();
				await server.stop();
			}
		});
	}
});
