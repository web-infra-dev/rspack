const { ModuleFederationPlugin } = require("@rspack/core").container;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		uniqueName: "2-container-full"
	},
	plugins: [
		new ModuleFederationPlugin({
			name: "main",
			library: { type: "commonjs-module" },
			remotes: {
				containerB: "../1-container-full/container.js",
				self: [
					"var undefined",
					"var (() => { throw new Error(); })()",
					"var { then: (a, b) => b(new Error()) }",
					"./bundle0.js"
				]
			},
			exposes: ["./Self"],
			shared: {
				xreact: "xreact",
				"old-react": {
					import: false,
					shareKey: "xreact",
					requiredVersion: "^2"
				},
				"old-react-singleton": {
					import: false,
					shareKey: "xreact",
					requiredVersion: "^2",
					singleton: true
				}
			}
		})
	]
};
