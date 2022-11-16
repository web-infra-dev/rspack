/**
 * @type {import('webpack').Configuration}
 */
module.exports = {
	mode: "development",
	entry: {
		main: ["./index.js"]
	},
	infrastructureLogging: {
		debug: true
	},
	builtins: {
		treeShaking: true
	}
};
