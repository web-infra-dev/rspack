const { ModuleFederationPlugin } = require("@rspack/core").container;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		new ModuleFederationPlugin({
			name: "container",
			filename: "remoteEntry.js",
			library: { type: "commonjs-module" },
			remotes: {
				remote:
					"promise Promise.resolve().then(() => ({ get: () => Promise.resolve(() => 'remote'), init: () => {} }))"
			},
			exposes: {
				"./exposed": "./exposed"
			},
			experiments: {
				asyncStartup: true
			}
		})
	]
};
