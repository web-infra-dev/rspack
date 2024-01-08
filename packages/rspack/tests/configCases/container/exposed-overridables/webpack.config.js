// eslint-disable-next-line node/no-unpublished-require
const { ModuleFederationPlugin } = require("../../../../").container;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		new ModuleFederationPlugin({
			name: "container",
			filename: "container.js",
			exposes: {
				"./Button": "./Button"
			},
			shared: {
				react: {
					eager: true
				}
			}
		})
	]
};
