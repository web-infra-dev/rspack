const Plugin = require("./plugin");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		js: "./src/index.js",
		mjs: "./src/index.mjs",
	},
	target: "async-node",
	output: {
		filename: "[name].js",
	},
	mode: "production",
	optimization: {
		minimize: false
	},
	plugins: [
		new Plugin(".js"),
		new Plugin(".mjs"),
	]
};
