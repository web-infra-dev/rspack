/**
 * @type {import('webpack').Configuration}
 */
module.exports = {
	mode: "development",
	entry: {
		main: "./index.js",
	},
	define: {
		"process.env.NODE_ENV": "development",
	},
	infrastructureLogging: {
		debug: true,
	},
	builtins: {
		tree_shaking: true,
	},
};
