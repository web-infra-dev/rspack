const Plugin = require("./plugin");

/** @type {import("webpack").Configuration} */
module.exports = {
	mode: "development",
	entry: {
		js: "./src/index.js",
		mjs: "./src/index.mjs"
	},
	target: "async-node",
	output: {
		filename: "[name].js"
	},
	mode: "production",
	optimization: {
		minimize: false,
		mangleExports: false,
		concatenateModules: false
	},
	plugins: [new Plugin(".js"), new Plugin(".mjs")]
};
