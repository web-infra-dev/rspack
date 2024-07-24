module.exports = {
	entry: {
		a: "./a",
		b: "./b",
		c: "./c",
	},
	module: {
		rules: [
			{
				test: /a\.js/,
				type: "javascript/auto"
			},
			{
				test: /b\.js/,
				type: "javascript/esm"
			},
			{
				test: /c\.js/,
				type: "javascript/dynamic"
			}
		]
	}
}

