// eslint-disable-next-line node/no-unpublished-require
const { ModuleFederationPlugin } = require("@rspack/core").container;

const common = {
	entry: {
		main: "./index.js"
	},
	target: 'async-node',
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
	shared: ["react"]
};

/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
	{
		...common,
		output: {
			filename: "[name].js",
			uniqueName: "1-container-full"
		},
		plugins: [
			new ModuleFederationPlugin({
				name: "container",
				library: { type: "commonjs-module" },
				runtimePlugins: [require.resolve('./runtimePlugin.js')],
				filename: "container.js",
				remotes: {
					containerA: "../0-container-full/container.js",
					containerB: "./container.js"
				},
				...commonMF
			})
		]
	}
];
