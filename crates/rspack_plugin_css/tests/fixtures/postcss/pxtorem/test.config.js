module.exports = {
	entry: {
		main: ["./index.js"]
	},
	builtins: {
		postcss: {
			pxtorem: {
				propList: ["*"],
				rootValue: 50
			}
		}
	}
};
