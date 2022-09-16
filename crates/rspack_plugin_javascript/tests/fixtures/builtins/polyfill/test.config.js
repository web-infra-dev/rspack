module.exports = {
	entry: {
		main: "./index.js"
	},
	target: ["browserslist", "es5"],
	builtins: {
		browserslist: ["chrome 60"],
		polyfill: false
	}
};
