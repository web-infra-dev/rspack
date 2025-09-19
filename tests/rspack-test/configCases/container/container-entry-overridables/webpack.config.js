const { ContainerPlugin } = require("@rspack/core").container;
const { ConsumeSharedPlugin } = require("@rspack/core").sharing;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		new ContainerPlugin({
			name: "container",
			filename: "container-file.js",
			library: {
				type: "commonjs-module"
			},
			exposes: {
				"./test": "./test"
			}
		}),
		new ConsumeSharedPlugin({
			consumes: {
				"./value": {
					shareKey: "value"
				}
			}
		})
	]
};
