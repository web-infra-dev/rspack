/**
 * @type {import('webpack').Configuration}
 */
module.exports = {
	mode: "development",
	entry: {
		main: ["./main.js"],
	},
	define: {
		"process.env.NODE_ENV": "development",
	},
	infrastructureLogging: {
		debug: true,
	},
	builtins: {
		treeShaking: true,
	},
};
