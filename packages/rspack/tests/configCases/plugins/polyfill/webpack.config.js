const polyfillNodePlugin = require("@rspack/plugin-node-polyfill");
module.exports = {
	target: ["webworker"],
	plugins: [new polyfillNodePlugin()],
	output: {
		chunkLoading: "jsonp"
	}
};
