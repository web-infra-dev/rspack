module.exports = {
	entry: {
		main: ["./index.js"]
	},
	builtins: {
		provide: {
			aaa: "./aaa"
		},
		treeShaking: true
	},
	experiments: {
		rspackFuture: {
			disableTransformByDefault: true
		}
	}
};
