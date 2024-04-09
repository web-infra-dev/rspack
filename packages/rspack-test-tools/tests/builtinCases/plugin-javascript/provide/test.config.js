module.exports = {
	builtins: {
		provide: {
			process: ["./process.js"],
			name: ["./name.js"]
		},
		treeShaking: true
	}
};
