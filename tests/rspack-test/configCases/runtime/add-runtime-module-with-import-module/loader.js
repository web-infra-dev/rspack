const path = require('path')

module.exports = async function loader() {
	const callback = this.async()
	const result = await this.importModule(path.resolve(__dirname, './execute-module.js'))
	callback(null, `export default ${result.default}`)
}
