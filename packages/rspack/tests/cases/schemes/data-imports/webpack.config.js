/** @type {import('webpack').Configuration} */
module.exports = {
	module: {
		rules: [
			{
				dependency: "url",
				scheme: /^data$/,
				type: "asset/resource"
			},
			{
				issuer: /\.js/,
				mimetype: /^image\/svg/,
				type: "asset/inline"
			}
		]
	},
	experiments: {
		css: true
	}
};
