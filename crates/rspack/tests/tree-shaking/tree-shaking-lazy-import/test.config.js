/**
 * @type {import('webpack').Configuration}
 */
module.exports = {
	mode: "development",
	entry: {
		main: {
			import: ["./index.js"]
		}
	},
	infrastructureLogging: {
		debug: true
	},
	builtins: {
		treeShaking: true
	}
};
