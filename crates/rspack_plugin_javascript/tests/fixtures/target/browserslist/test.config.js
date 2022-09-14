module.exports = {
	entry: {
		main: "./index.js"
	},
	target: ["browserslist", "es5"],
	builtins: {
		browserslist: ["chrome 60"]
	},
	external: {
		"core-js": "core-js"
	}
};
