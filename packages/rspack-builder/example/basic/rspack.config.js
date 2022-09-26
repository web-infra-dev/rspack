const path = require("path");
module.exports = {
	entry: {
		main: path.resolve(__dirname, "./index.js")
	},
	context: __dirname,
	mode: "development"
};
