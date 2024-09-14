"use strict";

const ModuleFederationPlugin =
	require("@rspack/core").container.ModuleFederationPlugin;

module.exports = {
	mode: "development",
	target: "node",
	stats: "none",
	context: __dirname,
	entry: ["./entry1.js"],
	plugins: [
		new ModuleFederationPlugin({
			name: "app1",
			library: { type: "var", name: "app1" },
			filename: "remoteEntry.js",
			exposes: {
				"./entry1": "./entry1"
			}
		})
	],
	infrastructureLogging: {
		level: "warn"
	}
};
