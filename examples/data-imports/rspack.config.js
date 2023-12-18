const path = require("path");

const pluginName = "plugin";
/** @type {import('@rspack/cli').Configuration} */
module.exports = {
	entry: {
		main: "./index.js"
	},
	module: {
		rules: [
			{
				dependency: "url",
				scheme: /^data$/,
				type: "asset/resource"
			},
			{
				issuer: /\.js/,
				mimetype: /^image\/svg/,
				type: "asset/inline"
			},
			{
				mimetype: /^text\/bad-base64/,
				type: "asset/inline"
			}
		]
	},
	optimization: {
		minimize: false
	},
	experiments: {
		css: true,
		rspackFuture: {
			newTreeshaking: true
		}
	}
};
