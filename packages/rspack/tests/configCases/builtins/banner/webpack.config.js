module.exports = {
	entry: {
		main: "./index",
		a: "./a"
	},
	builtins: {
		banner: [
			"MMMMMMM",
			{
				banner: "/** MMMMMMM */",
				raw: true,
				footer: true,
				entryOnly: true,
				exclude: [/a\.js/]
			}
		]
	}
};
