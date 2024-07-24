"use strict";

var NoEmitOnErrorsPlugin = require("@rspack/core").NoEmitOnErrorsPlugin;
var TestChildCompilationFailurePlugin = require("./TestChildCompilationFailurePlugin");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index",
	output: {
		filename: "bundle.js"
	},
	plugins: [
		new NoEmitOnErrorsPlugin(),
		new TestChildCompilationFailurePlugin({
			filename: "child.js"
		})
	]
};
