// eslint-disable-next-line node/no-unpublished-require
const { ModuleFederationPluginV1: ModuleFederationPlugin } = require("../../../../").container;

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
	shared: ["react"],
	shareScope: "test-scope"
};

/** @type {import("../../../../types").Configuration[]} */
module.exports = [
	{
		...common,
		output: {
			filename: "[name].js",
			uniqueName: "mf-with-shareScope"
		},
		plugins: [
			new ModuleFederationPlugin({
				name: "containerCjs",
				library: { type: "commonjs-module" },
				filename: "container.js",
				remotes: {
					containerA: "../0-container-full/container.js",
					containerB: "./container.js"
				},
				...commonMF
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
			uniqueName: "mf-with-shareScope-mjs"
		},
		plugins: [
			new ModuleFederationPlugin({
				name: "containerEsm",
				library: { type: "module" },
				filename: "module/container.mjs",
				remotes: {
					containerA: "../../0-container-full/module/container.mjs",
					containerB: "./container.mjs"
				},
				...commonMF
			})
		],
		target: "node14"
	}
];
