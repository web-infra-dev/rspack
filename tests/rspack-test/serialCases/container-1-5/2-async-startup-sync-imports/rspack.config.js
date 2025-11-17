// eslint-disable-next-line node/no-unpublished-require
const { ModuleFederationPlugin } = require("@rspack/core").container;

const common = {
	entry: {
		main: "./index.js"
	}
};

/** @type {ConstructorParameters<typeof ModuleFederationPlugin>[0]} */
const commonMF = {
	runtime: false,
	exposes: {
		"./ComponentB": "./ComponentB",
		"./ComponentC": "./ComponentC"
	},
	shared: ["react"]
};

/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
	{
		...common,
		output: {
			filename: "[name].js",
			uniqueName: "2-async-startup-sync-imports"
		},
		plugins: [
			new ModuleFederationPlugin({
				name: "container",
				library: { type: "commonjs-module" },
				filename: "container.js",
				remotes: {
					containerA: "../0-container-full/container.js",
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
		experiments: {
			outputModule: true
		},
		output: {
			filename: "module/[name].mjs",
			uniqueName: "2-async-startup-sync-imports-mjs"
		},
		plugins: [
			new ModuleFederationPlugin({
				name: "container",
				library: { type: "module" },
				filename: "module/container.mjs",
				remotes: {
					containerA: "../../0-container-full/module/container.mjs",
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
