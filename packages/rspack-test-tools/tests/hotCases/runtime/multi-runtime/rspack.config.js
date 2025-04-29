/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		main: "./a.js",
		b: "./b.js"
	},
	output: {
		filename: "[name].js",
		hotUpdateGlobal: "webpackHotUpdate_[runtime]"
	},
	experiments: {
		css: true
	}
};
