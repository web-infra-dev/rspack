module.exports = [
	{
		entry: { regex: "./index.js" },
		module: {
			noParse: /not-parsed/
		}
	},
	{
		entry: { func: "./index.js" },
		module: {
			noParse: function (content) {
				return /not-parsed/.test(content);
			}
		}
	}
];
