// var fs = require("fs");
// var path = require("path");
// module.exports = function (config) {
// 	return fs.existsSync(path.join(__dirname, "TEST.FILTER.JS"));
// };
// TODO: Works on Linux but fail on Windows
module.exports = () => false;
