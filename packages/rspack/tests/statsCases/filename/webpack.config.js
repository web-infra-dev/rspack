const path = require("path");

module.exports = {
	output: {
		path: path.join(__dirname, './dist'),
		filename: "[id].xxxx.js",
		chunkFilename: "[id].xxxx.js"
	}
};
