const { ModuleFederationPlugin } = require("@rspack/core").container;

const common = {
	entry: {
		main: "./index.js"
	},
	optimization: {
		runtimeChunk: "single"
	},
	experiments: {
		rspackFuture: {
			newTreeshaking: true
		}
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

/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
	{
		...common,
		output: {
			filename: "[name].js",
			uniqueName: "mf-with-shareScope"
		},
		plugins: [
			new ModuleFederationPlugin({
				name: "container2",
				library: { type: "commonjs-module" },
				filename: "container.js",
				remotes: {
					containerA: "../0-container-full/container.js",
					containerB: "./container.js"
				},
				...commonMF
			})
		]
	}
	// {
	// 	...common,
	// 	experiments: {
	// 		outputModule: true
	// 	},
	// 	output: {
	// 		filename: "module/[name].mjs",
	// 		uniqueName: "mf-with-shareScope-mjs"
	// 	},
	// 	plugins: [
	// 		new ModuleFederationPlugin({
	// 			name: "container2",
	// 			library: { type: "module" },
	// 			filename: "module/container.mjs",
	// 			remotes: {
	// 				containerA: "../../../0-container-full/dist/module/container.mjs",
	// 				containerB: "./container.mjs"
	// 			},
	// 			...commonMF
	// 		})
	// 	],
	// 	target: "node14"
	// }
];
