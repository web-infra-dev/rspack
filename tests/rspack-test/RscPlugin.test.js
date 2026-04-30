const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { experiments, rspack } = require("@rspack/core");

const { createPlugins, Layers } = experiments.rsc;

const runCompiler = compiler =>
	new Promise((resolve, reject) => {
		compiler.run((err, stats) => {
			if (err) {
				reject(err);
				return;
			}
			if (stats?.hasErrors()) {
				reject(
					new Error(
						stats.toString({
							all: false,
							errors: true,
							errorDetails: true
						})
					)
				);
				return;
			}
			resolve(stats);
		});
	});

const closeCompiler = compiler =>
	new Promise((resolve, reject) => {
		compiler.close(err => {
			if (err) {
				reject(err);
				return;
			}
			resolve();
		});
	});

describe("RscPlugin", () => {
	it("supports void and Promise<void> onServerComponentChanges callbacks", async () => {
		const tmp = fs.mkdtempSync(
			path.join(os.tmpdir(), "rspack-rsc-callback-")
		);
		const src = path.join(tmp, "src");
		const dist = path.join(tmp, "dist");
		const logFile = path.join(tmp, "on-server-component-changes.log");
		const { ServerPlugin, ClientPlugin } = createPlugins();
		let changeCount = 0;

		const readLog = () =>
			fs.existsSync(logFile) ? fs.readFileSync(logFile, "utf-8") : "";
		const writeApp = step => {
			fs.writeFileSync(
				path.join(src, "App.js"),
				`"use server-entry";\n\nexport const App = () => "step ${step}";\n`
			);
		};

		fs.mkdirSync(path.join(src, "framework"), { recursive: true });
		writeApp(0);
		fs.writeFileSync(
			path.join(src, "framework/entry.rsc.js"),
			"import { App } from '../App';\n\nexport const value = App();\n"
		);
		fs.writeFileSync(
			path.join(src, "framework/entry.ssr.js"),
			"import { value } from './entry.rsc';\n\nexport default value;\n"
		);
		fs.writeFileSync(path.join(src, "framework/entry.client.js"), "export {};\n");

		const swcLoaderRule = {
			test: /\.jsx?$/,
			use: [
				{
					loader: "builtin:swc-loader",
					options: {
						detectSyntax: "auto",
						rspackExperiments: {
							reactServerComponents: true
						}
					}
				}
			]
		};
		const sharedOptions = {
			mode: "development",
			cache: false,
			context: src,
			resolve: {
				extensions: ["...", ".jsx"],
				modules: [
					path.resolve(__dirname, "node_modules"),
					path.resolve(__dirname, "../../node_modules"),
					"node_modules"
				]
			},
			module: {
				rules: [
					swcLoaderRule,
					{
						resource: /[\\/]framework[\\/]entry\.ssr\.js$/,
						layer: Layers.ssr
					},
					{
						resource: /[\\/]framework[\\/]entry\.rsc\.js$/,
						layer: Layers.rsc,
						resolve: {
							conditionNames: ["react-server", "..."]
						}
					},
					{
						issuerLayer: Layers.rsc,
						resolve: {
							conditionNames: ["react-server", "..."]
						}
					}
				]
			}
		};
		const compiler = rspack([
			{
				...sharedOptions,
				name: "server",
				target: "node",
				entry: {
					main: {
						import: "./framework/entry.ssr.js"
					}
				},
				output: {
					path: dist,
					filename: "server-[name].js"
				},
				plugins: [
					new ServerPlugin({
						onServerComponentChanges() {
							changeCount += 1;

							if (changeCount === 1) {
								fs.appendFileSync(logFile, "callback returned void\n");
								return undefined;
							}

							return new Promise(resolve => {
								setTimeout(() => {
									fs.appendFileSync(logFile, "callback resolved promise\n");
									resolve();
								}, 20);
							});
						}
					})
				]
			},
			{
				...sharedOptions,
				name: "client",
				target: "web",
				entry: {
					main: {
						import: "./framework/entry.client.js"
					}
				},
				output: {
					path: dist,
					filename: "client-[name].js"
				},
				plugins: [new ClientPlugin()]
			}
		]);

		try {
			await runCompiler(compiler);
			expect(readLog()).toBe("");

			writeApp(1);
			await runCompiler(compiler);
			expect(readLog()).toBe("callback returned void\n");

			writeApp(2);
			await runCompiler(compiler);
			expect(readLog()).toBe(
				"callback returned void\ncallback resolved promise\n"
			);
		} finally {
			await closeCompiler(compiler);
			fs.rmSync(tmp, { recursive: true, force: true });
		}
	}, 120000);
});
