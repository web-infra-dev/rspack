const { CircularDependencyRspackPlugin } = require("@rspack/core");

module.exports = {
	entry: {
		aa: "./require-circular/d.js",
		bb: "./import-circular/index.js",
		cc: "./no-cycle/index.js",
		dd: "./ignore-circular/a.js"
	},
	plugins: [
		new CircularDependencyRspackPlugin({
			failOnError: false,
			exclude: /ignore-circular/,
			onStart(_compilation) {
				console.log("[Circular Dependency check] start right now");
				// compilation.warnings.push(new Error("[Circular Dependency check] start right now"))
			},
			onEnd(_compilation) {
				console.log("[Circular Dependency check] end right now");
				// compilation.warnings.push(new Error("[Circular Dependency check] end right now"))
			}
		})
	]
};
