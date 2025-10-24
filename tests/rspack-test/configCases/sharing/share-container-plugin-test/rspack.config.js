const path = require("path");

const { sharing } = require("@rspack/core");

const { ShareContainerPlugin,ConsumeSharedPlugin } = sharing;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		new ShareContainerPlugin({
			library: {
				name:'ui_lib',
				type:'commonjs2'
			},
			mfName: "host",
			shareName: "ui-lib",
			version: "1.0.0",
			request: path.resolve(__dirname, "node_modules/ui-lib/index.js")
		}),

		new ConsumeSharedPlugin({
			consumes: [
				['ui-lib', {
					import: 'ui-lib',
					shareScope: 'ui-lib',
				}],
				['ui-lib-dep', {
					import: 'ui-lib-dep',
					shareScope: 'ui-lib-dep',
				}],
			]
		})
	]
};
