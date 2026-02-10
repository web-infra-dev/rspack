const rspack = require("@rspack/core");

/** @type {import('@rspack/core').Configuration} */
module.exports = {
	builtins: {
		define: {
			"process.env.NODE_ENV": "'production'",
		}
	},
	optimization: {
		providedExports: true,
		usedExports: true,
		sideEffects: false,
	},
	plugins: [
		new rspack.BuiltinPlugin({
			name: "rspack.RsdoctorPlugin",
			options: {
				moduleGraphFeatures: ["sources"], // Enable module sources feature to collect JSON sizes
			}
		})
	]
};
