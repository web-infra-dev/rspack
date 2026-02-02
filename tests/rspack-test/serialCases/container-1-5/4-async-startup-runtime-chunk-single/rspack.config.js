const path = require("path");
const distRoot = path.resolve(
	__dirname,
	"../../../js/serial/container-1-5/4-async-startup-runtime-chunk-single"
);
const remoteOut = path.join(distRoot, "0-container-full");
const remoteContext = path.resolve(__dirname, "../0-container-full");

// Reuse the real remote container config so the case exercises emitted remotes.
const remoteConfigs = require("../0-container-full/rspack.config.js").map(
	config => {
		const isModule = config.output && config.output.module;
		return {
			...config,
			context: remoteContext,
			output: {
				...config.output,
				path: remoteOut,
				filename: isModule ? "module/[name].mjs" : "[name].js",
				chunkFilename: isModule ? "module/[id].mjs" : "[id].js"
			}
		};
	}
);
// eslint-disable-next-line node/no-unpublished-require
const { ModuleFederationPlugin } = require("@rspack/core").container;

const common = {
	entry: {
		main: "./index.js"
	},
	optimization: {
		runtimeChunk: "single"
	}
};

/** @type {ConstructorParameters<typeof ModuleFederationPlugin>[0]} */
const commonMF = {
	runtime: false,
	exposes: {
		"./ComponentB": "./ComponentB",
		"./ComponentC": "./ComponentC"
	},
	shared: ["mocked-react"]
};

/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
	...remoteConfigs,
	// Host bundles under test (CJS + ESM)
	{
		...common,
		target: "async-node",
		output: {
			filename: "[name].js",
			uniqueName: "4-async-startup-runtime-chunk-single",
			chunkLoading: "async-node"
		},
		plugins: [
			new ModuleFederationPlugin({
				name: "container",
				library: { type: "commonjs-module" },
				filename: "container.js",
				remotes: {
					containerA: "./0-container-full/container.js",
					containerB: "./container.js"
				},
				...commonMF,
				experiments: {
					asyncStartup: true
				}
			})
		]
	},
	{
		...common,
		output: {
			filename: "module/[name].mjs",
			uniqueName: "4-async-startup-runtime-chunk-single-mjs"
		},
		plugins: [
			new ModuleFederationPlugin({
				name: "container",
				library: { type: "module" },
				filename: "module/container.mjs",
				remotes: {
					// NOTE: this resolves from the host ESM output directory (module/)
					// so we need a single ../ to reach the collocated remote outputs.
					containerA: "../0-container-full/module/container.mjs",
					containerB: "./container.mjs"
				},
				...commonMF,
				experiments: {
					asyncStartup: true
				}
			})
		],
		target: "node14"
	}
];
