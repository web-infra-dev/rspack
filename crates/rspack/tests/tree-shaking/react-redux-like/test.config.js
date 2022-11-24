/**
 * @type {import('webpack').Configuration}
 */
module.exports = {
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
