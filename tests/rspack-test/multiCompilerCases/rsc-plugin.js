const fs = require("node:fs");
const path = require("node:path");
const { experiments } = require("@rspack/core");

const { createPlugins, Layers } = experiments.rsc;

const TEST_STATE = "rsc-plugin:on-server-component-changes";
const SYNC_CHANGE_EVENT = "server component change callback completed synchronously";
const ASYNC_CHANGE_EVENT =
	"server component change callback completed asynchronously";

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

const getTestState = context => {
	const state = context.getValue(TEST_STATE);
	if (!state) {
		throw new Error("RSC plugin test state has not been initialized");
	}
	return state;
};

const writeApp = (source, step) => {
	fs.writeFileSync(
		path.join(source, "App.js"),
		`"use server-entry";\n\nexport const App = () => "step ${step}";\n`
	);
};

const createFixture = context => {
	const root = context.getDist("rsc-plugin/on-server-component-changes");
	const source = path.join(root, "src");
	const output = path.join(root, "dist");

	fs.rmSync(root, { recursive: true, force: true });
	fs.mkdirSync(path.join(source, "framework"), { recursive: true });
	writeApp(source, 0);
	fs.writeFileSync(
		path.join(source, "framework/entry.rsc.js"),
		"import { App } from '../App';\n\nexport const value = App();\n"
	);
	fs.writeFileSync(
		path.join(source, "framework/entry.ssr.js"),
		"import { value } from './entry.rsc';\n\nexport default value;\n"
	);
	fs.writeFileSync(path.join(source, "framework/entry.client.js"), "export {};\n");

	return { output, source };
};

const createSharedOptions = fixture => ({
	mode: "development",
	cache: false,
	context: fixture.source,
	resolve: {
		extensions: ["...", ".jsx"],
		modules: [
			path.resolve(__dirname, "../node_modules"),
			path.resolve(__dirname, "../../../node_modules"),
			"node_modules"
		]
	},
	module: {
		rules: [
			{
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
			},
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
});

/** @type {import('@rspack/test-tools').TMultiCompilerCaseConfig} */
module.exports = {
	description:
		"should accept void and Promise<void> from onServerComponentChanges",
	options(context) {
		const fixture = createFixture(context);
		const events = [];
		const { ServerPlugin, ClientPlugin } = createPlugins();
		const sharedOptions = createSharedOptions(fixture);

		context.setValue(TEST_STATE, { events, fixture });

		return [
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
					path: fixture.output,
					filename: "server-[name].js"
				},
				plugins: [
					new ServerPlugin({
						onServerComponentChanges() {
							if (events.length === 0) {
								events.push(SYNC_CHANGE_EVENT);
								return undefined;
							}

							return new Promise(resolve => {
								setTimeout(() => {
									events.push(ASYNC_CHANGE_EVENT);
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
					path: fixture.output,
					filename: "client-[name].js"
				},
				plugins: [new ClientPlugin()]
			}
		];
	},
	async build(context, compiler) {
		const { events, fixture } = getTestState(context);

		try {
			await runCompiler(compiler);
			expect(events).toEqual([]);

			writeApp(fixture.source, 1);
			await runCompiler(compiler);
			expect(events).toEqual([SYNC_CHANGE_EVENT]);

			writeApp(fixture.source, 2);
			await runCompiler(compiler);
			expect(events).toEqual([SYNC_CHANGE_EVENT, ASYNC_CHANGE_EVENT]);
		} finally {
			await closeCompiler(compiler);
		}
	}
};
