const fs = require('fs')

module.exports = async function (code) {
	const resource = this.resource;
	if (fs.existsSync(resource + '.macro')) {
		const exports = await this.importModule(`dummy.webpack[javascript/auto]!=!!!${resource}.macro`, {});
		return `export default ${JSON.stringify(exports.default)}`
	}

	return code
}
