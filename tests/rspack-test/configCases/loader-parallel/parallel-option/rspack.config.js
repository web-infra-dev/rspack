module.exports = {
	context: __dirname,
	module: {
		rules: [
			{
				test: /lib\.js/,
				use: [
					{
						loader: "./unclonable.js",
						options: {
							notclonable() { }
						}
					},
					{
						loader: "./loader-in-worker.js",
						parallel: true,
						options: {}
					}
				]
			}
		]
	},
};
