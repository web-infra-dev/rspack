module.exports = async function () {
	const exports = await this.importModule(this.resourcePath, {});
	return `export default ${exports.add_one(1)}`;
};
