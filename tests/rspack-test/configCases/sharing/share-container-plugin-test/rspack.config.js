const path = require("path");

const { sharing } = require("@rspack/core");

const { ShareContainerPlugin } = sharing;

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
		})
	]
};
