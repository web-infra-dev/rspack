const path = require("path");

module.exports = {
	builtins: {
		html: [
			{
				publicPath: "/assets/",
				favicon: path.resolve(__dirname, "favicon.ico")
			}
		]
	}
};
