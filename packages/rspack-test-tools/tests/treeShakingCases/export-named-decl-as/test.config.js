module.exports = {
	entry: {
		main: {
			import: ["./src/index.js"]
		}
	},
	optimization: {
		sideEffects: true
	},
	builtins: {
		treeShaking: true,
		define: {
			"process.env.NODE_ENV": "'development'"
		}
	}
};
