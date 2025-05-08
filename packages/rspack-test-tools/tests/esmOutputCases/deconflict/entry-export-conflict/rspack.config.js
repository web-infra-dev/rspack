const path = require("path");

module.exports = {
	optimization: {
		splitChunks: {
			cacheGroups: {
				splitEvery: {
					chunks: "all",
					minSize: 0,
					test: /(foo|bar)/,
					name(m) {
						const name = m.nameForCondition();
						return path.relative(__dirname, name);
					}
				}
			}
		}
	}
};
