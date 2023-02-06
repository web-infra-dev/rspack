const CopyPlugin = require("copy-webpack-plugin");
module.exports = {
	plugins: [
		new CopyPlugin([
			{
				from: "*.txt",
				to: "[name].[ext]",
				transform(content) {
					return content + "to";
				}
			}
		])
	]
};
