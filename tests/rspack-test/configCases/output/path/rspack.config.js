/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	target: "node",
	// 故意不设置 output.path，测试默认值
	output: {
		filename: "bundle.js"
	}
};
