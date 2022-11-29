module.exports = {
	mode: "development",
	entry: {
		index: {
			import: ["./index.js"]
		}
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
