const { rspack } = require("@rspack/core");
/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		library: {
			type: "amd",
			amdContainer: "window['clientContainer']"
		}
	},
	node: {
		__dirname: false,
		__filename: false
	},
	plugins: [
		new rspack.BannerPlugin({
			raw: true,
			banner:
				"function define(fn) { fn(); }\nconst window = {};\nwindow['clientContainer'] = { define };\n"
		})
	]
};

