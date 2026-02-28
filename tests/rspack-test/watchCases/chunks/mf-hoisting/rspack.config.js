const rspack = require("@rspack/core");
const ReactRefreshPlugin = require("@rspack/plugin-react-refresh");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "web",
	plugins: [
		new rspack.container.ModuleFederationPlugin({
			name: "test",
			shareStrategy: "loaded-first"
		}),
		new ReactRefreshPlugin() // Need this to trigger hoisting (hoist_container_references_plugin.rs)
	]
};
