const { ModuleFederationPluginV1: ModuleFederationPlugin } =
	require("@rspack/core").container;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	plugins: [
		new ModuleFederationPlugin({
			name: "A",
			filename: "container-a.js",
			library: {
				type: "commonjs-module"
			},
			exposes: {
				".": "./a"
			},
			remoteType: "commonjs-module",
			remotes: {
				A: "./container-a.js"
			}
		}),
		function (compiler) {
			compiler.hooks.thisCompilation.tap(
				"ChangeDataRequest",
				(compilation, { normalModuleFactory }) => {
					normalModuleFactory.hooks.beforeResolve.tap(
						"ChangeDataRequest",
						data => {
							if (data.request === "myA") {
								data.request = "A";
							}
						}
					);
				}
			);
		}
	]
};
