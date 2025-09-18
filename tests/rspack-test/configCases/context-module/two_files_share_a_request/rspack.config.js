/** @type {import("@rspack/core").Configuration} */
module.exports = {
  resolve: {
    extensions: [".ts", "..."],
  },
	optimization: {
		minimize: false,
		moduleIds:"named",
	},
	module: {
		rules: [
			{
				test: /\.ts$/,
				type: "javascript/auto"
			}]
	},
};
