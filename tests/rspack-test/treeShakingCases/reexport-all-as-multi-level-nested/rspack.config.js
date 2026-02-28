const { DefinePlugin } = require("@rspack/core");
/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		main: {
			import: ["./src/index.js"]
		}
	},
	optimization: {
		sideEffects: true
	},
	plugins: [
		new DefinePlugin({
			"process.env.NODE_ENV": "'development'"
		})
	],
};
