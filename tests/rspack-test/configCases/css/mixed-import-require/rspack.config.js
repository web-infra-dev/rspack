/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	entry: "./index.js",
	experiments: {
		css: true
	},
  target: "web"
};