
/**var fs = require("fs");
var path = require("path");

module.exports = function(config) {
	return fs.existsSync(path.join(__dirname, "TEST.FILTER.JS"));
};
*/
module.exports = () => {return "https://github.com/web-infra-dev/rspack/issues/4347"}
							