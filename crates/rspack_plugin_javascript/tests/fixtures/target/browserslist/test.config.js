module.exports = {
	entry: {
		main: "./index.js"
	},
	target: ["browserslist", "es2022"],
	builtins: {
		browserslist: ["chrome 60"]
	},
	external: {
		"core-js": "core-js"
	}
};
