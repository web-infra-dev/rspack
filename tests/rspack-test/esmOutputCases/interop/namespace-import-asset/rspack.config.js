module.exports = {
	module: {
		rules: [
			{
				test: /foo\.mjs$/,
				type: 'asset/resource',
				generator: {
					importMode: "preserve",
				},
			}
		]
	}
}
