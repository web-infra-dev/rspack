const path = require("path");

/** @type {import('@rspack/cli').Configuration} */
module.exports = {
	mode: "development", // will be override to "production" by "--mode"
	extends: ["./base.config.js"],
	output: {
		path: path.resolve(__dirname, "dist")
	},
	cache: true,
	experiments: {
		cache: {
			type: "persistent"
		}
	},
	plugins: [
		{
			apply(compiler) {
				console.log(
					"buildDependencies is ",
					compiler.options.experiments.cache.buildDependencies
				);
			}
		}
	]
};
