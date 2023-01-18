/**
 * @type {import('webpack').Configuration}
 */
module.exports = {
	mode: "development",
	entry: {
		main: {
			import: ["./index.ts"]
		}
	},
	define: {
		"process.env.NODE_ENV": "'development'"
	},
	infrastructureLogging: {
		debug: true
	},
	builtins: {
		treeShaking: true,
		sideEffects: true
	}
};
