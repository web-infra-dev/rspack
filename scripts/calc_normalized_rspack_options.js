// cacl normalized rspack options
// run: node calc_normalized_rspack_options /<your project path>/test.config.js
const path = require("path");
const { getNormalizedRspackOptions } = require(path.join(
	__dirname,
	"../packages/rspack"
));

const config_path = process.argv[2];
const config = getNormalizedRspackOptions(require(config_path));

console.log(JSON.stringify(config));
