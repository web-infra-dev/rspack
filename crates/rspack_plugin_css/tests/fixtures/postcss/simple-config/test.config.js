module.exports = {
	entry: {
		main: {
			import: ["./index.js"]
		}
	},
	builtins: {
		css: {
			presetEnv: ["chrome >= 40", "firefox > 10"]
		}
	}
};
