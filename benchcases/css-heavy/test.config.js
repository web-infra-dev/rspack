module.exports = {
	mode: "development",
	entry: {
		index: ["./index.js"]
	},
	enhanced: {},
	builtins: {
		postcss: {
			pxtorem: {
				propList: ["*"]
			}
		}
	}
};
