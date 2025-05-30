const fs = require("fs");

module.exports = async function (code) {
	const exports = await this.importModule(this.resourcePath, {});
	return `export default ${exports.add_one(1)}`;
};
