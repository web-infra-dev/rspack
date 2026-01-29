const { ModuleFederationPlugin } = require("@rspack/core").container;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization: {
		chunkIds: "named",
		moduleIds: "named",
		concatenateModules: false,
		// inlineExports will inline shared.js into b.js, and 2-transitive-overriding will check
		// the __webpack_modules__ of this container, so disable inlineExports to avoid test fail
		inlineExports: false
	},
	output: {
		uniqueName: "1-transitive-overriding"
	},
	plugins: [
		new ModuleFederationPlugin({
			name: "container-no-shared",
			library: { type: "commonjs-module" },
			filename: "container-no-shared.js",
			exposes: ["./a", "./b", "./modules", "./modules-from-remote"],
			remotes: {
				"container-with-shared":
					"../0-transitive-overriding/container-with-shared.js",
				"container-no-shared": "./container-no-shared.js"
			}
		})
	],
};
