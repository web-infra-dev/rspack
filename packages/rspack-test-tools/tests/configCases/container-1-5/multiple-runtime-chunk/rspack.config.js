const { ModuleFederationPlugin } = require("@rspack/core").container;

const common = {
	entry: {
		main: {
			import: "./index.js",
			runtime: "other"
		},
		another: {
			import: "./index.js",
			runtime: "webpack"
		}
	},
	optimization: {
		runtimeChunk: "single"
	}
};

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
		mode: "production",
		...common,
		output: {
			filename: "[name].js",
			uniqueName: "1-container-full"
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
			uniqueName: "1-container-full-mjs"
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
				...commonMF
			})
		],
		target: "node14"
	}
];
