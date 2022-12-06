/**
 * @type {import('webpack').Configuration}
 */
module.exports = {
	mode: "development",
	builtins: {
		treeShaking: true
	},
	context: __dirname,
	entry: {
		main: {
			import: ["./index.js"]
		}
	},
	define: {
		"process.env.NODE_ENV": "'development'"
	},
	infrastructureLogging: {
		debug: false
	}
};
