const { ModuleFederationPlugin } = require("@rspack/core").container;

function createConfig() {
	return {
		output: {
			filename: "[name].js"
		},
		plugins: [
			new ModuleFederationPlugin({
				name: "container",
				library: { type: "commonjs-module" },
				exposes: ["./a"],
				manifest:false,
				remotes: {
					container2:
						"promise Promise.resolve().then(() => require('./container2.js'))"
				}
			}),
			new ModuleFederationPlugin({
				name: "container2",
				library: { type: "commonjs-module" },
				exposes: ["./b"],
				manifest:false,
				remotes: {
					container:
						"promise Promise.resolve().then(() => require('./container.js'))"
				}
			})
		]
	};
}

module.exports = createConfig();
